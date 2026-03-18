pub mod app;
pub mod error;

pub use error::{Result, VeniError};

use std::path::{Path, PathBuf};

/// Entry point for the veni file manager.
pub fn run(
    path: PathBuf,
    _theme: Option<String>,
    _config: Option<&Path>,
) -> Result<()> {
    let path = std::fs::canonicalize(&path).unwrap_or(path);
    if !path.is_dir() {
        return Err(VeniError::NotADirectory(path));
    }
    // TODO: initialize app and enter event loop
    Ok(())
}
