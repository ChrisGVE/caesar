use crate::config::Config;
use crate::detector::FileKind;

/// Resolved viewer choice for a file kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewerChoice {
    /// Hard override from `--tool` CLI flag; fails if tool not installed.
    ToolHard(String),
    /// Soft preference from `config.viewer[kind]`; falls back to registry if unavailable.
    ToolSoft(String),
    /// Force the internal renderer; falls back to registry only if no internal renderer exists
    /// for the sub-format (e.g. mobi within Ebook).
    Internal,
    /// Default: try internal renderer first (if available), then registry.
    Default,
}

/// Resolve the viewer choice from CLI flag and config.
///
/// Priority (highest to lowest):
/// 1. `cli_tool` → `ToolHard`
/// 2. `config.viewer[kind] = "internal"` → `Internal`
/// 3. `config.viewer[kind] = "<tool>"` → `ToolSoft`
/// 4. Nothing set → `Default`
pub fn resolve_viewer_choice(
    kind: FileKind,
    cli_tool: Option<&str>,
    config: &Config,
) -> ViewerChoice {
    if let Some(tool) = cli_tool {
        return ViewerChoice::ToolHard(tool.to_string());
    }
    match config.viewer.get(kind.config_key()).map(|s| s.as_str()) {
        Some("internal") => ViewerChoice::Internal,
        Some(tool) => ViewerChoice::ToolSoft(tool.to_string()),
        None => ViewerChoice::Default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn config_with_viewer(key: &str, value: &str) -> Config {
        let mut cfg = Config::default();
        cfg.viewer.insert(key.to_string(), value.to_string());
        cfg
    }

    #[test]
    fn cli_tool_produces_tool_hard() {
        let cfg = Config::default();
        let choice = resolve_viewer_choice(FileKind::Pdf, Some("zathura"), &cfg);
        assert_eq!(choice, ViewerChoice::ToolHard("zathura".to_string()));
    }

    #[test]
    fn config_internal_produces_internal() {
        let cfg = config_with_viewer("ebook", "internal");
        let choice = resolve_viewer_choice(FileKind::Ebook, None, &cfg);
        assert_eq!(choice, ViewerChoice::Internal);
    }

    #[test]
    fn config_tool_name_produces_tool_soft() {
        let cfg = config_with_viewer("pdf", "zathura");
        let choice = resolve_viewer_choice(FileKind::Pdf, None, &cfg);
        assert_eq!(choice, ViewerChoice::ToolSoft("zathura".to_string()));
    }

    #[test]
    fn no_cli_no_config_produces_default() {
        let cfg = Config::default();
        let choice = resolve_viewer_choice(FileKind::Pdf, None, &cfg);
        assert_eq!(choice, ViewerChoice::Default);
    }

    #[test]
    fn cli_takes_precedence_over_config() {
        let cfg = config_with_viewer("pdf", "zathura");
        let choice = resolve_viewer_choice(FileKind::Pdf, Some("evince"), &cfg);
        assert_eq!(choice, ViewerChoice::ToolHard("evince".to_string()));
    }
}
