use crate::detector::FileKind;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Contextual information about a file/directory for action discovery.
#[derive(Debug, Clone)]
pub struct Context {
    pub path: PathBuf,
    pub file_kind: Option<FileKind>,
    pub project_type: Option<String>,
    pub git_state: Option<GitState>,
    pub environment: HashMap<String, String>,
}

/// Git repository state.
#[derive(Debug, Clone)]
pub struct GitState {
    pub branch: Option<String>,
    pub is_dirty: bool,
    pub has_staged: bool,
}

/// An action that can be performed on a context.
#[derive(Debug, Clone)]
pub struct Action {
    pub name: String,
    pub description: String,
    pub provider: String,
    pub metadata: ActionMetadata,
}

/// Metadata about an action.
#[derive(Debug, Clone)]
pub struct ActionMetadata {
    pub category: String,
    pub confidence: f32,
    pub requires: Vec<String>,
}

/// Analyze a path/environment to produce a Context.
pub trait ContextAnalyzer {
    fn analyze(&self, path: &Path) -> crate::error::Result<Context>;
}

/// Discover available actions for a given context.
pub trait ActionDiscovery {
    fn discover(&self, context: &Context) -> Vec<Action>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_can_be_created() {
        let ctx = Context {
            path: PathBuf::from("/tmp/test"),
            file_kind: Some(FileKind::Text),
            project_type: None,
            git_state: None,
            environment: HashMap::new(),
        };
        assert_eq!(ctx.path, PathBuf::from("/tmp/test"));
        assert_eq!(ctx.file_kind, Some(FileKind::Text));
    }

    #[test]
    fn action_can_be_created() {
        let action = Action {
            name: "cargo.build".into(),
            description: "Build the project".into(),
            provider: "rust".into(),
            metadata: ActionMetadata {
                category: "compile".into(),
                confidence: 0.95,
                requires: vec!["cargo".into()],
            },
        };
        assert_eq!(action.name, "cargo.build");
        assert_eq!(action.metadata.confidence, 0.95);
    }

    #[test]
    fn git_state_defaults() {
        let state = GitState {
            branch: Some("main".into()),
            is_dirty: false,
            has_staged: false,
        };
        assert_eq!(state.branch.as_deref(), Some("main"));
        assert!(!state.is_dirty);
    }
}
