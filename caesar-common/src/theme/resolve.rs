use super::{builtin::builtin_theme, palette::Theme};

/// Resolve the active theme from the priority-ordered override chain.
///
/// Resolution order (highest priority first):
/// 1. `tool_env` — tool-specific env var (e.g. `VIDI_THEME`)
/// 2. `cli_override` — value of the `--theme` CLI flag
/// 3. `module_config` — value from the tool's `[vidi]` config section
/// 4. `workspace_env` — value of the `CAESAR_THEME` environment variable
/// 5. `workspace_config` — value from the `[caesar]` config section
/// 6. Default: `catppuccin-mocha`
///
/// Tool-specific settings always win over workspace-level defaults.
/// For each step, `custom_themes` is searched first, then built-in themes.
/// If a name is given but not found anywhere, the resolution falls through to
/// the next step.  If no step yields a theme, `catppuccin-mocha` is returned.
pub fn resolve_theme(
    tool_env: Option<String>,
    cli_override: Option<String>,
    module_config: Option<String>,
    workspace_env: Option<String>,
    workspace_config: Option<String>,
    custom_themes: &[Theme],
) -> Theme {
    let sources = [
        tool_env,
        cli_override,
        module_config,
        workspace_env,
        workspace_config,
    ];

    for maybe_name in sources.iter().flatten() {
        if let Some(theme) = find_theme(maybe_name, custom_themes) {
            return theme;
        }
    }

    // Guaranteed to exist — panicking here would be a programming error.
    builtin_theme("catppuccin-mocha").expect("catppuccin-mocha must always be present")
}

/// Search `custom_themes` first, then fall back to built-in themes.
fn find_theme(name: &str, custom_themes: &[Theme]) -> Option<Theme> {
    custom_themes
        .iter()
        .find(|t| t.name == name)
        .cloned()
        .or_else(|| builtin_theme(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::palette::Color;

    fn dummy_theme(name: &str) -> Theme {
        let c = Color { r: 0, g: 0, b: 0 };
        Theme {
            name: name.into(),
            bg: c.clone(),
            fg: c.clone(),
            cursor: c.clone(),
            ansi: std::array::from_fn(|_| c.clone()),
            accents: std::array::from_fn(|_| c.clone()),
        }
    }

    #[test]
    fn defaults_to_mocha_when_all_none() {
        let theme = resolve_theme(None, None, None, None, None, &[]);
        assert_eq!(theme.name, "catppuccin-mocha");
    }

    #[test]
    fn tool_env_wins_over_workspace_env() {
        let theme = resolve_theme(
            Some("catppuccin-frappe".into()), // VIDI_THEME — wins
            None,
            None,
            Some("catppuccin-latte".into()), // CAESAR_THEME — lower priority
            None,
            &[],
        );
        assert_eq!(theme.name, "catppuccin-frappe");
    }

    #[test]
    fn workspace_env_used_when_no_tool_env() {
        let theme = resolve_theme(
            None,
            None,
            None,
            Some("catppuccin-latte".into()), // CAESAR_THEME
            None,
            &[],
        );
        assert_eq!(theme.name, "catppuccin-latte");
    }

    #[test]
    fn tool_env_wins_over_cli_and_config() {
        let theme = resolve_theme(
            Some("catppuccin-latte".into()),     // VIDI_THEME
            Some("catppuccin-frappe".into()),    // --theme
            Some("catppuccin-macchiato".into()), // [vidi].theme
            None,
            None,
            &[],
        );
        assert_eq!(theme.name, "catppuccin-latte");
    }

    #[test]
    fn cli_wins_over_module_config() {
        let theme = resolve_theme(
            None,
            Some("catppuccin-frappe".into()),    // --theme
            Some("catppuccin-macchiato".into()), // [vidi].theme
            None,
            None,
            &[],
        );
        assert_eq!(theme.name, "catppuccin-frappe");
    }

    #[test]
    fn module_config_wins_over_workspace() {
        let theme = resolve_theme(
            None,
            None,
            Some("catppuccin-frappe".into()), // [vidi].theme
            None,
            Some("catppuccin-latte".into()), // [caesar].theme
            &[],
        );
        assert_eq!(theme.name, "catppuccin-frappe");
    }

    #[test]
    fn workspace_config_used_as_fallback() {
        let theme = resolve_theme(
            None,
            None,
            None,
            None,
            Some("catppuccin-macchiato".into()), // [caesar].theme
            &[],
        );
        assert_eq!(theme.name, "catppuccin-macchiato");
    }

    #[test]
    fn unknown_name_falls_through() {
        let theme = resolve_theme(
            Some("no-such-theme".into()),
            Some("catppuccin-latte".into()),
            None,
            None,
            None,
            &[],
        );
        assert_eq!(theme.name, "catppuccin-latte");
    }

    #[test]
    fn all_unknown_falls_back_to_mocha() {
        let theme = resolve_theme(
            Some("nope".into()),
            Some("nope2".into()),
            Some("nope3".into()),
            Some("nope4".into()),
            Some("nope5".into()),
            &[],
        );
        assert_eq!(theme.name, "catppuccin-mocha");
    }

    #[test]
    fn custom_theme_overrides_builtin_of_same_name() {
        let mut custom = dummy_theme("catppuccin-mocha");
        custom.bg = Color {
            r: 0xFF,
            g: 0xFF,
            b: 0xFF,
        };
        let theme = resolve_theme(
            None,
            None,
            Some("catppuccin-mocha".into()),
            None,
            None,
            &[custom],
        );
        assert_eq!(theme.bg.to_hex(), "#FFFFFF");
    }

    #[test]
    fn custom_theme_found_by_name() {
        let custom = dummy_theme("my-special-theme");
        let theme = resolve_theme(
            None,
            None,
            Some("my-special-theme".into()),
            None,
            None,
            &[custom],
        );
        assert_eq!(theme.name, "my-special-theme");
    }
}
