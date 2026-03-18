use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VeniError {
    #[error("not a directory: {0}")]
    NotADirectory(PathBuf),

    #[error("cannot read directory: {path}: {source}")]
    ReadDir {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("configuration error: {0}")]
    Config(#[from] caesar_common::error::ConfigError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("terminal error: {0}")]
    Terminal(String),
}

pub type Result<T> = std::result::Result<T, VeniError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_a_directory_displays_path() {
        let err = VeniError::NotADirectory(PathBuf::from("/tmp/file.txt"));
        assert!(err.to_string().contains("/tmp/file.txt"));
    }

    #[test]
    fn io_error_converts() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err = VeniError::from(io_err);
        assert!(err.to_string().contains("denied"));
    }
}
