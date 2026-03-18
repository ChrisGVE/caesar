use std::path::PathBuf;
use thiserror::Error;

pub use caesar_common::error::ConfigError;

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

impl From<caesar_common::error::CommonError> for VidiError {
    fn from(e: caesar_common::error::CommonError) -> Self {
        use caesar_common::error::CommonError;
        match e {
            CommonError::FileNotFound(p) => VidiError::FileNotFound(p),
            CommonError::FileUnreadable { path, source } => {
                VidiError::FileUnreadable { path, source }
            }
            CommonError::Config(c) => VidiError::Config(c),
            CommonError::Io(io) => VidiError::Io(io),
            CommonError::Theme(msg) => VidiError::Theme(msg),
            CommonError::Detection(msg) => VidiError::Theme(msg),
        }
    }
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

    #[test]
    fn from_common_error_file_not_found() {
        use caesar_common::error::CommonError;
        let common = CommonError::FileNotFound(PathBuf::from("/tmp/x"));
        let vidi: VidiError = common.into();
        assert!(vidi.to_string().contains("/tmp/x"));
    }

    #[test]
    fn from_common_error_theme() {
        use caesar_common::error::CommonError;
        let common = CommonError::Theme("bad theme".to_string());
        let vidi: VidiError = common.into();
        assert!(vidi.to_string().contains("bad theme"));
    }
}
