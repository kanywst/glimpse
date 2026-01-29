// Utility functions (Placeholder)
use directories::ProjectDirs;
use std::path::PathBuf;

#[must_use]
pub fn get_config_dir() -> PathBuf {
    ProjectDirs::from("io", "glim", "glim").map_or_else(
        || PathBuf::from(".").join(".glim"),
        |proj_dirs| proj_dirs.config_dir().to_path_buf(),
    )
}
