use crate::app::{App, InputMode, ZoomLevel};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, Wrap},
};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub fn render(app: &App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main Content
            Constraint::Length(3), // Footer / Search Bar
        ])
        .split(frame.area());

    render_header(app, frame, chunks[0]);
    render_main(app, frame, chunks[1]);
    render_footer(app, frame, chunks[2]);
}

fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let title = match app.zoom_level {
        ZoomLevel::Galaxy => "🌌 GALAXY VIEW - Dashboard",
        ZoomLevel::Structure => "🏗️  STRUCTURE VIEW - Hierarchy",
        ZoomLevel::Logic => "📝 LOGIC VIEW - Diff",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    // Right-aligned status info
    let status = format!(
        " {} - {} ",
        app.dashboard_info.repo_name, app.dashboard_info.branch_name
    );
    let title_span = Span::styled(
        title,
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::White),
    );

    // Calculate lengths before moving status
    let width = area.width as usize;
    let title_len = title.len();
    let status_len = status.len();

    let status_span = Span::styled(&status, Style::default().fg(Color::Gray));

    let spacer_len = width.saturating_sub(title_len + status_len + 4); // 4 for borders/padding
    let spacer = " ".repeat(spacer_len);

    let line = Line::from(vec![title_span, Span::raw(spacer), status_span]);

    let paragraph = Paragraph::new(line).block(block);
    frame.render_widget(paragraph, area);
}

fn render_main(app: &App, frame: &mut Frame, area: Rect) {
    match app.zoom_level {
        ZoomLevel::Galaxy => render_galaxy(app, frame, area),
        ZoomLevel::Structure => render_structure(app, frame, area),
        ZoomLevel::Logic => render_logic(app, frame, area),
    }
}

fn render_galaxy(app: &App, frame: &mut Frame, area: Rect) {
    // Split into Dashboard info (Top) and Heatmap (Bottom)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Dashboard Info
            Constraint::Min(0),    // Heatmap List
        ])
        .split(area);

    // 1. Dashboard Info
    let info_block = Block::default()
        .borders(Borders::ALL)
        .title(" Repository Overview ");

    let rows = vec![
        Row::new(vec![
            Cell::from("Repository").style(Style::default().fg(Color::Yellow)),
            Cell::from(app.dashboard_info.repo_name.as_str()),
        ]),
        Row::new(vec![
            Cell::from("Branch/PR").style(Style::default().fg(Color::Yellow)),
            Cell::from(app.dashboard_info.branch_name.as_str()),
        ]),
        Row::new(vec![
            Cell::from("Description").style(Style::default().fg(Color::Yellow)),
            Cell::from(app.dashboard_info.description.as_str()),
        ]),
        Row::new(vec![
            Cell::from("Stats").style(Style::default().fg(Color::Yellow)),
            Cell::from(app.dashboard_info.stats.as_str()),
        ]),
    ];

    let table = Table::new(rows, [Constraint::Length(15), Constraint::Min(0)])
        .block(info_block)
        .column_spacing(2);

    frame.render_widget(table, chunks[0]);

    // 2. Heatmap List (Existing Logic)
    let items: Vec<ListItem> = app
        .modules
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let color = if m.heat > 70 {
                Color::Red
            } else if m.heat > 30 {
                Color::Yellow
            } else {
                Color::Green
            };
            let prefix = if i == app.selected_index { "> " } else { "  " };
            let style = if i == app.selected_index {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{:<20}", m.name),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" Impact: {:>3}% ", m.heat),
                    Style::default().fg(color),
                ),
                Span::styled(
                    format!("| {}", m.description),
                    Style::default().fg(Color::Gray),
                ),
            ]))
            .style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Impact Zones (Select to Zoom) "),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(list, chunks[1]);
}

fn render_structure(app: &App, frame: &mut Frame, area: Rect) {
    // Use filtered indices to access structures
    let items: Vec<ListItem> = app
        .filtered_structure_indices
        .iter()
        .enumerate()
        .map(|(i, &real_index)| {
            let s = &app.structures[real_index]; // Map back to real structure
            let prefix = if i == app.selected_index { "> " } else { "  " };
            let style = if i == app.selected_index {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let (icon, color) = if s.is_file {
                // File icons
                if s.status.contains("New") {
                    ("+", Color::Green)
                } else if s.status.contains("Deleted") {
                    ("-", Color::Red)
                } else {
                    ("M", Color::Yellow)
                }
            } else {
                // Symbol icons
                match s.status.as_str() {
                    "fn" => ("ƒ", Color::Cyan),
                    "struct" => ("S", Color::Magenta),
                    "impl" => ("I", Color::Blue),
                    _ => ("•", Color::Gray),
                }
            };

            // Lazygit Style: Staged Indicator
            let staged_mark = if s.is_staged { "[x] " } else { "[ ] " };
            let staged_style = if s.is_staged {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                if s.is_file {
                    Span::styled(staged_mark, staged_style)
                } else {
                    Span::raw("    ")
                },
                Span::styled(format!("[{icon}] "), Style::default().fg(color)),
                Span::styled(
                    s.text.clone(),
                    Style::default().fg(if s.is_file { Color::White } else { Color::Gray }),
                ),
            ]))
            .style(style)
        })
        .collect();

    let title = if app.search_query.is_empty() {
        " Structure Map (Space to Stage, / to Search) ".to_string()
    } else {
        format!(" Search Results: '{}' ", app.search_query)
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(list, area);
}

fn render_logic(app: &App, frame: &mut Frame, area: Rect) {
    // Basic syntax highlighting setup (Load only once in real app)
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    // Use filtered indices logic to get current selection
    let syntax = if !app.filtered_structure_indices.is_empty()
        && app.selected_index < app.filtered_structure_indices.len()
    {
        let real_index = app.filtered_structure_indices[app.selected_index];
        let path = &app.structures[real_index].path;
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("rs");
        ps.find_syntax_by_extension(ext)
            .unwrap_or_else(|| ps.find_syntax_plain_text())
    } else {
        ps.find_syntax_plain_text()
    };

    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

    let lines: Vec<Line> = app
        .logic_view_content
        .iter()
        .map(|s| {
            // Determine Diff color background
            let (bg_color, prefix) = if s.starts_with('+') {
                (Some(Color::Rgb(20, 60, 20)), "+")
            } else if s.starts_with('-') {
                (Some(Color::Rgb(60, 20, 20)), "-")
            } else {
                (None, " ")
            };

            // Syntax Highlight the content (excluding prefix)
            let content = if s.len() > 1 { &s[1..] } else { "" };
            let ranges: Vec<(syntect::highlighting::Style, &str)> =
                h.highlight_line(content, &ps).unwrap_or_default();

            let mut spans = vec![Span::styled(prefix, Style::default().fg(Color::Gray))];

            for (style, text) in ranges {
                let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                spans.push(Span::styled(text.to_string(), Style::default().fg(fg)));
            }

            let mut line_style = Style::default();
            if let Some(bg) = bg_color {
                line_style = line_style.bg(bg);
            }

            Line::from(spans).style(line_style)
        })
        .collect();

    // Display context info in title
    let title = format!(
        " Code Diff (Context: {} lines) [+/- to expand] ",
        app.context_lines
    );

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_footer(app: &App, frame: &mut Frame, area: Rect) {
    if app.input_mode == InputMode::Editing {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Yellow));
        let paragraph = Paragraph::new(format!("/{}", app.search_query)).block(block);
        frame.render_widget(paragraph, area);
    } else {
        let info_text = match app.zoom_level {
            ZoomLevel::Galaxy => "Nav: [j/k] Select | [Enter] Zoom In | [q] Quit",
            ZoomLevel::Structure => {
                "Nav: [j/k] Select | [Enter] Zoom In | [Space] Stage | [/] Search | [Back] Out"
            }
            ZoomLevel::Logic => {
                "Nav: [j/k] Scroll | [+/-] Context | [Backspace] Zoom Out | [q] Quit"
            }
        };

        let paragraph = Paragraph::new(info_text)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::TOP));

        frame.render_widget(paragraph, area);
    }
}
