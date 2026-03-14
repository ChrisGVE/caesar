use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VidiError {
    #[error("file not found: {0}")]
    FileNotFound(PathBuf),

    #[error("cannot read file: {path}: {source}")]
    FileUnreadable {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("no viewer available for {kind}")]
    NoViewerAvailable { kind: String },

    #[error("tool '{tool}' failed with exit code {code}")]
    ToolFailed { tool: String, code: i32 },

    #[error("tool '{tool}' not found on PATH")]
    ToolNotFound { tool: String },

    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("theme error: {0}")]
    Theme(String),
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to parse config file {path}: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
}

pub type Result<T> = std::result::Result<T, VidiError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn file_not_found_displays_path() {
        let err = VidiError::FileNotFound(PathBuf::from("/tmp/missing.txt"));
        assert!(err.to_string().contains("/tmp/missing.txt"));
    }

    #[test]
    fn no_viewer_displays_kind() {
        let err = VidiError::NoViewerAvailable {
            kind: "Pdf".to_string(),
        };
        assert!(err.to_string().contains("Pdf"));
    }

    #[test]
    fn tool_failed_displays_tool_and_code() {
        let err = VidiError::ToolFailed {
            tool: "bat".to_string(),
            code: 1,
        };
        let msg = err.to_string();
        assert!(msg.contains("bat"));
        assert!(msg.contains('1'));
    }
}
