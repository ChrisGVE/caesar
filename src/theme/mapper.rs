use super::palette::Theme;

/// Converts a [`Theme`] into per-tool arguments and style strings.
pub struct ThemeMapper<'a> {
    theme: &'a Theme,
}

impl<'a> ThemeMapper<'a> {
    /// Create a new mapper for the given theme.
    pub fn new(theme: &'a Theme) -> Self {
        Self { theme }
    }

    /// Return the `bat` `--theme` argument value for this theme.
    ///
    /// Maps `catppuccin-mocha` → `"Catppuccin Mocha"` etc.
    /// Falls back to `"base16"` for unknown names.
    pub fn bat_theme_name(&self) -> String {
        map_bat_theme(&self.theme.name)
    }

    /// Return `"dark"` or `"light"` based on background luminance.
    ///
    /// Used for tools like `glow` that accept a style word rather than a
    /// palette.  Threshold at 0.5 matches the standard perceptual midpoint.
    pub fn glow_style(&self) -> String {
        if self.theme.bg.luminance() >= 0.5 {
            "light".into()
        } else {
            "dark".into()
        }
    }

    /// Return the background color as a `#RRGGBB` hex string.
    pub fn chafa_bg(&self) -> String {
        self.theme.bg.to_hex()
    }

    /// Return the foreground color as a `#RRGGBB` hex string.
    pub fn chafa_fg(&self) -> String {
        self.theme.fg.to_hex()
    }
}

/// Map a vidi theme name to the corresponding `bat` theme name.
fn map_bat_theme(name: &str) -> String {
    match name {
        "catppuccin-mocha" => "Catppuccin Mocha",
        "catppuccin-latte" => "Catppuccin Latte",
        "catppuccin-frappe" => "Catppuccin Frappe",
        "catppuccin-macchiato" => "Catppuccin Macchiato",
        other => {
            // For user-named themes, title-case the hyphenated name as a
            // best-effort mapping; users can always override via tool_overrides.
            return title_case_hyphenated(other);
        }
    }
    .into()
}

/// Convert a hyphenated lowercase string to Title Case.
/// `"my-theme"` → `"My Theme"`.
fn title_case_hyphenated(s: &str) -> String {
    s.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let upper: String = first.to_uppercase().collect();
                    upper + chars.as_str()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Expand placeholder tokens in an args slice and return the final `Vec<String>`.
///
/// Supported tokens: `{theme}`, `{cols}`, `{rows}`, `{lines}`, `{bytes}`.
/// `{bytes}` is computed as `cols * lines * 16` (rough estimate for hex tools).
pub fn apply_to_args(
    args: &[&'static str],
    mapper: &ThemeMapper<'_>,
    cols: u16,
    rows: u16,
    lines: u16,
) -> Vec<String> {
    // Pre-compute the substitution values.
    let theme_name = mapper.bat_theme_name();
    let cols_s = cols.to_string();
    let rows_s = rows.to_string();
    let lines_s = lines.to_string();
    let bytes_s = (u32::from(cols) * u32::from(lines) * 16).to_string();

    args.iter()
        .map(|arg| {
            arg.replace("{theme}", &theme_name)
                .replace("{cols}", &cols_s)
                .replace("{rows}", &rows_s)
                .replace("{lines}", &lines_s)
                .replace("{bytes}", &bytes_s)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::builtin::builtin_theme;

    fn mocha_mapper() -> (Theme,) {
        (builtin_theme("catppuccin-mocha").unwrap(),)
    }

    #[test]
    fn bat_theme_name_for_mocha() {
        let (theme,) = mocha_mapper();
        let mapper = ThemeMapper::new(&theme);
        assert_eq!(mapper.bat_theme_name(), "Catppuccin Mocha");
    }

    #[test]
    fn bat_theme_name_for_latte() {
        let theme = builtin_theme("catppuccin-latte").unwrap();
        let mapper = ThemeMapper::new(&theme);
        assert_eq!(mapper.bat_theme_name(), "Catppuccin Latte");
    }

    #[test]
    fn bat_theme_name_unknown_falls_back_to_title_case() {
        use crate::theme::palette::Color;
        let dummy_color = Color { r: 0, g: 0, b: 0 };
        let theme = Theme {
            name: "my-custom-theme".into(),
            bg: dummy_color.clone(),
            fg: dummy_color.clone(),
            cursor: dummy_color.clone(),
            ansi: std::array::from_fn(|_| dummy_color.clone()),
            accents: std::array::from_fn(|_| dummy_color.clone()),
        };
        let mapper = ThemeMapper::new(&theme);
        assert_eq!(mapper.bat_theme_name(), "My Custom Theme");
    }

    #[test]
    fn glow_style_dark_for_mocha() {
        let (theme,) = mocha_mapper();
        let mapper = ThemeMapper::new(&theme);
        assert_eq!(mapper.glow_style(), "dark");
    }

    #[test]
    fn glow_style_light_for_latte() {
        let theme = builtin_theme("catppuccin-latte").unwrap();
        let mapper = ThemeMapper::new(&theme);
        assert_eq!(mapper.glow_style(), "light");
    }

    #[test]
    fn chafa_bg_returns_hex_string() {
        let (theme,) = mocha_mapper();
        let mapper = ThemeMapper::new(&theme);
        let bg = mapper.chafa_bg();
        assert!(bg.starts_with('#'), "chafa_bg should start with #");
        assert_eq!(bg.len(), 7, "chafa_bg should be 7 chars");
    }

    #[test]
    fn chafa_bg_mocha_correct_value() {
        let (theme,) = mocha_mapper();
        let mapper = ThemeMapper::new(&theme);
        assert_eq!(mapper.chafa_bg(), "#1E1E2E");
    }

    #[test]
    fn placeholder_theme_replaced() {
        let (theme,) = mocha_mapper();
        let mapper = ThemeMapper::new(&theme);
        let args = apply_to_args(&["--theme={theme}"], &mapper, 80, 24, 20);
        assert_eq!(args[0], "--theme=Catppuccin Mocha");
    }

    #[test]
    fn placeholder_cols_rows_lines_replaced() {
        let (theme,) = mocha_mapper();
        let mapper = ThemeMapper::new(&theme);
        let args = apply_to_args(&["{cols}", "{rows}", "{lines}"], &mapper, 80, 24, 10);
        assert_eq!(args, vec!["80", "24", "10"]);
    }

    #[test]
    fn placeholder_bytes_computed() {
        let (theme,) = mocha_mapper();
        let mapper = ThemeMapper::new(&theme);
        // cols=80, lines=10 → bytes = 80 * 10 * 16 = 12800
        let args = apply_to_args(&["{bytes}"], &mapper, 80, 24, 10);
        assert_eq!(args[0], "12800");
    }

    #[test]
    fn no_placeholders_args_unchanged() {
        let (theme,) = mocha_mapper();
        let mapper = ThemeMapper::new(&theme);
        let args = apply_to_args(&["--plain", "--paging=never"], &mapper, 80, 24, 20);
        assert_eq!(args, vec!["--plain", "--paging=never"]);
    }
}
