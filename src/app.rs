use crate::github::{GitHubClient, PrInfo};
use crate::semantics::SemanticAnalyzer;
use git2::{DiffOptions, Repository, StatusOptions};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomLevel {
    Galaxy,
    Structure,
    Logic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub heat: u8,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct StructureItem {
    pub text: String,
    pub path: String,
    pub is_file: bool,
    pub status: String,
    pub line_no: Option<usize>,
    pub is_staged: bool,
}

#[derive(Debug, Clone, Default)]
pub struct DashboardInfo {
    pub repo_name: String,
    pub branch_name: String,
    pub description: String,
    pub stats: String,
}

pub enum DataSource {
    Local {
        repo: Repository,
        root: PathBuf,
    },
    GitHub {
        pr_info: Box<PrInfo>,
        raw_diff: String,
        file_diffs: HashMap<String, Vec<String>>,
    },
}

impl fmt::Debug for DataSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Local { root, .. } => f
                .debug_struct("Local")
                .field("root", root)
                .field("repo", &"Repository")
                .finish(),
            Self::GitHub { pr_info, .. } => f
                .debug_struct("GitHub")
                .field("pr_info", pr_info)
                .finish_non_exhaustive(),
        }
    }
}

pub struct App {
    pub zoom_level: ZoomLevel,
    pub modules: Vec<Module>,
    pub structures: Vec<StructureItem>,
    pub logic_view_content: Vec<String>,
    // Indices of structures that match the search query
    pub filtered_structure_indices: Vec<usize>,
    pub selected_index: usize,
    pub analyzer: SemanticAnalyzer,
    pub source: Option<DataSource>,
    pub error_msg: Option<String>,
    pub repo_root: PathBuf,
    pub dashboard_info: DashboardInfo,
    pub context_lines: u32,
    // Search State
    pub input_mode: InputMode,
    pub search_query: String,
}

impl fmt::Debug for App {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("App")
            .field(
                "mode",
                &if self.source.is_some() {
                    "Ready"
                } else {
                    "Empty/Error"
                },
            )
            .finish_non_exhaustive()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}

impl App {
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        let mut app = Self {
            zoom_level: ZoomLevel::Galaxy,
            modules: vec![],
            structures: vec![],
            logic_view_content: vec![],
            filtered_structure_indices: vec![],
            selected_index: 0,
            analyzer: SemanticAnalyzer::new(),
            source: None,
            error_msg: None,
            repo_root: path.clone(),
            dashboard_info: DashboardInfo::default(),
            context_lines: 3,
            input_mode: InputMode::Normal,
            search_query: String::new(),
        };

        // Determine mode
        let path_str = path.to_string_lossy();
        if path_str.starts_with("http")
            || path_str.contains("github.com")
            || path_str.contains("/pull/")
        {
            match app.load_github(&path_str) {
                Ok(()) => {}
                Err(e) => app.error_msg = Some(format!("GitHub Error: {e}")),
            }
        } else {
            match app.load_local(path) {
                Ok(()) => {}
                Err(e) => app.error_msg = Some(format!("Local Error: {e}")),
            }
        }

        if !app.structures.is_empty() {
            app.update_search(); // Initialize filtered list
            app.load_diff();
        }

        app
    }

    // --- Search Logic ---
    pub fn update_search(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_structure_indices = (0..self.structures.len()).collect();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_structure_indices = self
                .structures
                .iter()
                .enumerate()
                .filter(|(_, item)| item.text.to_lowercase().contains(&query))
                .map(|(i, _)| i)
                .collect();
        }
        // Reset selection if out of bounds or empty
        if self.filtered_structure_indices.is_empty()
            || self.selected_index >= self.filtered_structure_indices.len()
        {
            self.selected_index = 0;
        }
        // Load diff for the new selection if applicable
        self.load_diff();
    }

    pub fn enter_search(&mut self) {
        self.input_mode = InputMode::Editing;
        self.search_query.clear();
        self.update_search();
    }

    pub const fn exit_search(&mut self) {
        self.input_mode = InputMode::Normal;
        // Keep the filter? No, standard behavior is usually reset or keep.
        // Let's reset for now if Esc is pressed, but if Enter was used we might keep it.
        // Actually, let's clearer: Esc cancels search (clears query), Enter commits it (keeps query).
    }

    pub fn cancel_search(&mut self) {
        self.input_mode = InputMode::Normal;
        self.search_query.clear();
        self.update_search();
    }

    // --- Loading Logic ---

    fn load_local(&mut self, path: PathBuf) -> anyhow::Result<()> {
        let repo = Repository::open(&path)?;

        let repo_name = path.file_name().map_or_else(
            || "Unknown Repo".to_string(),
            |n| n.to_string_lossy().to_string(),
        );

        let branch_name = repo.head().map_or_else(
            |_| "Empty Repo".to_string(),
            |head| head.shorthand().unwrap_or("DETACHED HEAD").to_string(),
        );

        let (modules, structures) = Self::scan_local_repo(&repo, &path, &mut self.analyzer);

        self.dashboard_info = DashboardInfo {
            repo_name,
            branch_name,
            description: "Local Working Tree Changes".to_string(),
            stats: format!(
                "{} files changed",
                structures.iter().filter(|i| i.is_file).count()
            ),
        };

        self.modules = modules;
        self.structures = structures;
        self.source = Some(DataSource::Local { repo, root: path });
        Ok(())
    }

    fn load_github(&mut self, pr_ref: &str) -> anyhow::Result<()> {
        GitHubClient::check_auth()?;
        let info = GitHubClient::fetch_pr_info(pr_ref)?;
        let raw_diff = GitHubClient::fetch_pr_diff(pr_ref)?;

        let file_diffs = Self::split_diff(&raw_diff);

        let mut dir_counts: HashMap<String, usize> = HashMap::new();
        let mut structures = Vec::new();

        for file in &info.files {
            let path = Path::new(&file.path);
            if let Some(parent) = path.parent() {
                let parent_str = parent.to_string_lossy().to_string();
                *dir_counts
                    .entry(if parent_str.is_empty() {
                        "root".to_string()
                    } else {
                        parent_str
                    })
                    .or_insert(0) += 1;
            }

            structures.push(StructureItem {
                text: file.path.clone(),
                path: file.path.clone(),
                is_file: true,
                status: format!("+{} -{}", file.additions, file.deletions),
                line_no: None,
                is_staged: false,
            });
        }

        let mut modules: Vec<Module> = dir_counts
            .into_iter()
            .map(|(name, count)| {
                let heat = (count * 10).min(100) as u8;
                Module {
                    name,
                    heat,
                    description: format!("{count} changed files"),
                }
            })
            .collect();
        modules.sort_by(|a, b| b.heat.cmp(&a.heat));

        self.dashboard_info = DashboardInfo {
            repo_name: info.head_repository.name_with_owner.clone(),
            branch_name: format!("#{}", info.number),
            description: info.title.clone(),
            stats: format!(
                "+{} -{} ({} files)",
                info.additions, info.deletions, info.changed_files
            ),
        };

        self.modules = modules;
        self.structures = structures;
        self.source = Some(DataSource::GitHub {
            pr_info: Box::new(info),
            raw_diff,
            file_diffs,
        });

        Ok(())
    }

    fn split_diff(raw: &str) -> HashMap<String, Vec<String>> {
        let mut map = HashMap::new();
        let mut current_file = String::new();
        let mut current_lines = Vec::new();

        for line in raw.lines() {
            if line.starts_with("diff --git") {
                if !current_file.is_empty() {
                    map.insert(current_file.clone(), current_lines.clone());
                    current_lines.clear();
                }
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(b_path) = parts.last() {
                    current_file = b_path.trim_start_matches("b/").to_string();
                }
            }
            current_lines.push(line.to_string());
        }
        if !current_file.is_empty() {
            map.insert(current_file, current_lines);
        }
        map
    }

    fn scan_local_repo(
        repo: &Repository,
        root: &Path,
        analyzer: &mut SemanticAnalyzer,
    ) -> (Vec<Module>, Vec<StructureItem>) {
        let mut status_opts = StatusOptions::new();
        status_opts.include_untracked(true);

        let statuses = repo
            .statuses(Some(&mut status_opts))
            .unwrap_or_else(|_| repo.statuses(None).expect("Failed to get statuses"));

        let mut dir_counts: HashMap<String, usize> = HashMap::new();
        let mut structures = Vec::new();

        for entry in statuses.iter() {
            let path_str = entry.path().unwrap_or("unknown").to_string();
            let status_char = format!("{:?}", entry.status());

            let is_staged = entry.status().contains(git2::Status::INDEX_NEW)
                || entry.status().contains(git2::Status::INDEX_MODIFIED)
                || entry.status().contains(git2::Status::INDEX_DELETED);

            structures.push(StructureItem {
                text: path_str.clone(),
                path: path_str.clone(),
                is_file: true,
                status: status_char,
                line_no: None,
                is_staged,
            });

            if let Some(parent) = Path::new(&path_str).parent() {
                let parent_str = parent.to_string_lossy().to_string();
                *dir_counts
                    .entry(if parent_str.is_empty() {
                        "root".to_string()
                    } else {
                        parent_str
                    })
                    .or_insert(0) += 1;
            }

            let full_path = root.join(&path_str);
            if full_path.exists()
                && let Ok(content) = fs::read_to_string(&full_path)
            {
                let symbols = analyzer.analyze(&path_str, &content);
                for sym in symbols {
                    structures.push(StructureItem {
                        text: format!("  {} {}", sym.kind, sym.name),
                        path: path_str.clone(),
                        is_file: false,
                        status: sym.kind,
                        line_no: Some(sym.start_line),
                        is_staged: false,
                    });
                }
            }
        }

        let mut modules: Vec<Module> = dir_counts
            .into_iter()
            .map(|(name, count)| {
                let heat = (count * 10).min(100) as u8;
                Module {
                    name,
                    heat,
                    description: format!("{count} changed files"),
                }
            })
            .collect();
        modules.sort_by(|a, b| b.heat.cmp(&a.heat));

        (modules, structures)
    }

    fn load_diff(&mut self) {
        if self.structures.is_empty() || self.source.is_none() {
            return;
        }

        // Logic View content clearing logic
        self.logic_view_content.clear();

        // Get the REAL index from the filtered list
        if self.selected_index >= self.filtered_structure_indices.len() {
            self.selected_index = 0; // Fallback
        }

        if self.filtered_structure_indices.is_empty() {
            return;
        }

        let real_index = self.filtered_structure_indices[self.selected_index];
        let item = &self.structures[real_index];

        let path = &item.path;
        if path.is_empty() {
            return;
        }

        match self.source.as_ref().expect("Source must be loaded") {
            DataSource::Local { repo, .. } => {
                let mut diff_opts = DiffOptions::new();
                diff_opts.pathspec(path);
                diff_opts.context_lines(self.context_lines);

                let diff = if let Ok(tree) = repo.head().and_then(|h| h.peel_to_tree()) {
                    repo.diff_tree_to_workdir_with_index(Some(&tree), Some(&mut diff_opts))
                        .ok()
                } else {
                    repo.diff_tree_to_workdir_with_index(None, Some(&mut diff_opts))
                        .ok()
                };

                if let Some(diff) = diff {
                    let _ = diff.print(git2::DiffFormat::Patch, |_, _, line| {
                        let content = String::from_utf8_lossy(line.content())
                            .trim_end()
                            .to_string();
                        let prefix = match line.origin() {
                            '+' => "+",
                            '-' => "-",
                            _ => " ",
                        };
                        self.logic_view_content.push(format!("{prefix}{content}"));
                        true
                    });
                }
            }
            DataSource::GitHub { file_diffs, .. } => {
                if let Some(lines) = file_diffs.get(path) {
                    self.logic_view_content = lines.clone();
                } else {
                    self.logic_view_content
                        .push("No diff available for this file.".to_string());
                }
            }
        }

        if !item.is_file
            && let Some(line) = item.line_no
        {
            self.logic_view_content
                .push(format!("--- Focused on Line {line} ---"));
        }
    }

    pub fn increase_context(&mut self) {
        if matches!(self.zoom_level, ZoomLevel::Logic)
            && matches!(self.source, Some(DataSource::Local { .. }))
        {
            self.context_lines = self.context_lines.saturating_add(3);
            self.load_diff();
        }
    }

    pub fn decrease_context(&mut self) {
        if matches!(self.zoom_level, ZoomLevel::Logic)
            && matches!(self.source, Some(DataSource::Local { .. }))
        {
            self.context_lines = self.context_lines.saturating_sub(3).max(1);
            self.load_diff();
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn toggle_stage(&mut self) {
        if let Some(DataSource::Local { repo, root: path }) = &self.source
            && !self.filtered_structure_indices.is_empty()
        {
            // Use real index
            let real_index = self.filtered_structure_indices[self.selected_index];
            let item = &self.structures[real_index];

            if !item.is_file {
                return;
            }

            let file_path = Path::new(&item.path);
            let mut index = repo.index().expect("Failed to get index");

            if item.is_staged {
                if let Ok(head) = repo.head() {
                    let obj = head
                        .peel(git2::ObjectType::Any)
                        .expect("Failed to peel HEAD");
                    repo.reset_default(Some(&obj), vec![file_path])
                        .expect("Failed to unstage");
                } else {
                    index
                        .remove_path(file_path)
                        .expect("Failed to remove from index");
                }
            } else {
                index.add_path(file_path).expect("Failed to add path");
            }
            index.write().expect("Failed to write index");

            let (modules, structures) = Self::scan_local_repo(repo, path, &mut self.analyzer);
            self.modules = modules;
            self.structures = structures;
            self.update_search(); // Re-apply filter to update indices
        }
    }

    pub fn next(&mut self) {
        let max = match self.zoom_level {
            ZoomLevel::Galaxy => self.modules.len(),
            ZoomLevel::Structure => self.filtered_structure_indices.len(), // Use filtered len
            ZoomLevel::Logic => self.logic_view_content.len().max(1),
        };

        if max > 0 && self.selected_index < max - 1 {
            self.selected_index += 1;
            if matches!(self.zoom_level, ZoomLevel::Structure) {
                self.load_diff();
            }
        }
    }

    pub fn previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            if matches!(self.zoom_level, ZoomLevel::Structure) {
                self.load_diff();
            }
        }
    }

    pub fn zoom_in(&mut self) {
        match self.zoom_level {
            ZoomLevel::Galaxy => {
                self.zoom_level = ZoomLevel::Structure;
                self.selected_index = 0;
                self.load_diff();
            }
            ZoomLevel::Structure => {
                // Prevent zooming if list is empty
                if !self.filtered_structure_indices.is_empty() {
                    self.zoom_level = ZoomLevel::Logic;
                    self.selected_index = 0;
                }
            }
            ZoomLevel::Logic => {}
        }
    }

    pub const fn zoom_out(&mut self) {
        match self.zoom_level {
            ZoomLevel::Galaxy => {}
            ZoomLevel::Structure => {
                self.zoom_level = ZoomLevel::Galaxy;
                self.selected_index = 0;
            }
            ZoomLevel::Logic => {
                self.zoom_level = ZoomLevel::Structure;
                self.selected_index = 0;
            }
        }
    }
}
