use anyhow::{Context, Result};
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize, Clone)]
pub struct PrInfo {
    pub number: u64,
    pub title: String,
    pub body: String,
    pub state: String,
    pub author: Author,
    #[serde(rename = "headRepository")]
    pub head_repository: RepoInfo,
    #[serde(rename = "changedFiles")]
    pub changed_files: u64,
    pub additions: u64,
    pub deletions: u64,
    pub files: Vec<PrFile>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RepoInfo {
    pub name: String,
    #[serde(rename = "nameWithOwner")]
    pub name_with_owner: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Author {
    pub login: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PrFile {
    pub path: String,
    pub additions: u64,
    pub deletions: u64,
}

#[derive(Debug)]
pub struct GitHubClient;

impl GitHubClient {
    /// Check if `gh` CLI is available and logged in
    ///
    /// # Errors
    /// Returns error if `gh` command is missing or not logged in.
    pub fn check_auth() -> Result<()> {
        let status = Command::new("gh")
            .arg("auth")
            .arg("status")
            .output()
            .context("Failed to execute 'gh' command. Is GitHub CLI installed?")?;

        if !status.status.success() {
            anyhow::bail!("GitHub CLI is not logged in. Please run 'gh auth login'.");
        }
        Ok(())
    }

    /// Fetch PR metadata using `gh pr view`
    ///
    /// # Errors
    /// Returns error if `gh` command fails or JSON parsing fails.
    pub fn fetch_pr_info(pr_ref: &str) -> Result<PrInfo> {
        // First get general info
        let output = Command::new("gh")
            .arg("pr")
            .arg("view")
            .arg(pr_ref)
            .arg("--json")
            .arg("number,title,body,state,author,headRepository,changedFiles,additions,deletions,files")
            .output()
            .context("Failed to fetch PR info")?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh command failed: {err}");
        }

        let info: PrInfo =
            serde_json::from_slice(&output.stdout).context("Failed to parse PR JSON")?;

        Ok(info)
    }

    /// Fetch PR diff content using `gh pr diff`
    ///
    /// # Errors
    /// Returns error if `gh` command fails.
    pub fn fetch_pr_diff(pr_ref: &str) -> Result<String> {
        let output = Command::new("gh")
            .arg("pr")
            .arg("diff")
            .arg(pr_ref)
            .output()
            .context("Failed to fetch PR diff")?;

        if !output.status.success() {
            anyhow::bail!("Failed to get diff");
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
