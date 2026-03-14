use super::{builtin::builtin_theme, palette::Theme};

/// Resolve the active theme from the priority-ordered override chain.
///
/// Resolution order (highest priority first):
/// 1. `env_override` — value of the `VIDI_THEME` environment variable
/// 2. `cli_override` — value of the `--theme` CLI flag
/// 3. `config_theme` — value from the config file
/// 4. Default: `catppuccin-mocha`
///
/// For each step, `custom_themes` is searched first, then built-in themes.
/// If a name is given but not found anywhere, the resolution falls through to
/// the next step.  If no step yields a theme, `catppuccin-mocha` is returned.
pub fn resolve_theme(
    env_override: Option<String>,
    cli_override: Option<String>,
    config_theme: Option<String>,
    custom_themes: &[Theme],
) -> Theme {
    let sources = [env_override, cli_override, config_theme];

    for maybe_name in &sources {
        if let Some(name) = maybe_name {
            if let Some(theme) = find_theme(name, custom_themes) {
                return theme;
            }
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
        let theme = resolve_theme(None, None, None, &[]);
        assert_eq!(theme.name, "catppuccin-mocha");
    }

    #[test]
    fn env_wins_over_cli_and_config() {
        let theme = resolve_theme(
            Some("catppuccin-latte".into()),
            Some("catppuccin-frappe".into()),
            Some("catppuccin-macchiato".into()),
            &[],
        );
        assert_eq!(theme.name, "catppuccin-latte");
    }

    #[test]
    fn cli_wins_over_config_when_no_env() {
        let theme = resolve_theme(
            None,
            Some("catppuccin-frappe".into()),
            Some("catppuccin-macchiato".into()),
            &[],
        );
        assert_eq!(theme.name, "catppuccin-frappe");
    }

    #[test]
    fn config_used_when_no_env_or_cli() {
        let theme = resolve_theme(None, None, Some("catppuccin-macchiato".into()), &[]);
        assert_eq!(theme.name, "catppuccin-macchiato");
    }

    #[test]
    fn unknown_name_falls_back_to_next_source() {
        // env has an unknown name, cli has a valid name → cli wins
        let theme = resolve_theme(
            Some("no-such-theme".into()),
            Some("catppuccin-latte".into()),
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
        let theme = resolve_theme(None, None, Some("catppuccin-mocha".into()), &[custom]);
        // Should get the custom version (white bg), not the built-in dark one
        assert_eq!(theme.bg.to_hex(), "#FFFFFF");
    }

    #[test]
    fn custom_theme_found_by_name() {
        let custom = dummy_theme("my-special-theme");
        let theme = resolve_theme(None, Some("my-special-theme".into()), None, &[custom]);
        assert_eq!(theme.name, "my-special-theme");
    }
}
