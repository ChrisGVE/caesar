use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{ConfigError, Result};

fn default_theme() -> String {
    "catppuccin-mocha".to_string()
}

/// Vidi configuration, loaded from `~/.config/vidi/config.toml`.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    /// Active theme name; defaults to `catppuccin-mocha`.
    #[serde(default = "default_theme")]
    pub theme: String,

    /// User-defined themes that extend or override built-in themes.
    #[serde(default)]
    pub custom_themes: Vec<crate::theme::Theme>,

    /// Per-tool argument overrides, keyed by tool name.
    #[serde(default)]
    pub tool_overrides: HashMap<String, Vec<String>>,

    /// Additional directories to search for viewer tools beyond PATH.
    #[serde(default)]
    pub extra_search_paths: Vec<PathBuf>,

    /// Per-kind viewer preferences.
    /// Keys are `FileKind::config_key()` values (e.g. `"ebook"`, `"html"`).
    /// Value `"internal"` selects the internal renderer;
    /// any other value is treated as a soft tool-name preference.
    #[serde(default)]
    pub viewer: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            custom_themes: Vec::new(),
            tool_overrides: HashMap::new(),
            extra_search_paths: Vec::new(),
            viewer: HashMap::new(),
        }
    }
}

/// Resolve the XDG config path for vidi.
///
/// Respects `XDG_CONFIG_HOME` and falls back to `~/.config`.
fn default_config_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("vidi").join("config.toml"))
}

/// Load configuration from `path_override` or the default XDG location.
///
/// Returns `Config::default()` when the file does not exist.
/// Returns `Err` only on read or parse failure.
pub fn load_config(path_override: Option<&Path>) -> Result<Config> {
    let path = match path_override {
        Some(p) => p.to_path_buf(),
        None => match default_config_path() {
            Some(p) => p,
            None => return Ok(Config::default()),
        },
    };

    if !path.exists() {
        return Ok(Config::default());
    }

    let raw = std::fs::read_to_string(&path).map_err(|source| {
        crate::error::VidiError::Config(ConfigError::Read {
            path: path.clone(),
            source,
        })
    })?;

    let config: Config = toml::from_str(&raw).map_err(|source| {
        crate::error::VidiError::Config(ConfigError::Parse {
            path: path.clone(),
            source,
        })
    })?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as IoWrite;

    #[test]
    fn missing_file_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.toml");
        let config = load_config(Some(&path)).unwrap();
        assert_eq!(config.theme, "catppuccin-mocha");
        assert!(config.custom_themes.is_empty());
        assert!(config.tool_overrides.is_empty());
    }

    #[test]
    fn valid_minimal_toml_parses() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, r#"theme = "catppuccin-latte""#).unwrap();
        drop(f);

        let config = load_config(Some(&path)).unwrap();
        assert_eq!(config.theme, "catppuccin-latte");
    }

    #[test]
    fn empty_toml_uses_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "").unwrap();

        let config = load_config(Some(&path)).unwrap();
        assert_eq!(config.theme, "catppuccin-mocha");
    }

    #[test]
    fn invalid_toml_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "this is not valid = [[toml").unwrap();

        let result = load_config(Some(&path));
        assert!(result.is_err());
    }

    #[test]
    fn tool_overrides_parsed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            r#"[tool_overrides]
bat = ["--paging=always"]
"#,
        )
        .unwrap();

        let config = load_config(Some(&path)).unwrap();
        assert!(config.tool_overrides.contains_key("bat"));
    }

    #[test]
    fn default_theme_string_is_mocha() {
        assert_eq!(default_theme(), "catppuccin-mocha");
    }

    #[test]
    fn empty_config_has_empty_viewer_map() {
        let config = Config::default();
        assert!(config.viewer.is_empty());
    }

    #[test]
    fn viewer_internal_parsed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[viewer]\nepub = \"internal\"\n").unwrap();

        let config = load_config(Some(&path)).unwrap();
        assert_eq!(
            config.viewer.get("epub").map(|s| s.as_str()),
            Some("internal")
        );
    }

    #[test]
    fn viewer_tool_name_parsed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[viewer]\npdf = \"zathura\"\n").unwrap();

        let config = load_config(Some(&path)).unwrap();
        assert_eq!(
            config.viewer.get("pdf").map(|s| s.as_str()),
            Some("zathura")
        );
    }
}
