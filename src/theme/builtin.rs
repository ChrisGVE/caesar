use super::palette::{Color, Theme};

/// Convenience macro to build a `Color` from a hex literal without `unwrap`
/// noise throughout the static data. Panics at runtime if the literal is
/// malformed — acceptable because these are compile-time constants authored
/// by us, not user input.
macro_rules! c {
    ($hex:expr) => {
        match Color::from_hex($hex) {
            Some(color) => color,
            None => panic!("invalid built-in color literal"),
        }
    };
}

fn make_mocha() -> Theme {
    Theme {
        name: "catppuccin-mocha".into(),
        bg: c!("#1E1E2E"),
        fg: c!("#CDD6F4"),
        cursor: c!("#F5E0DC"),
        ansi: [
            c!("#45475A"), // 0  Surface1
            c!("#F38BA8"), // 1  Red
            c!("#A6E3A1"), // 2  Green
            c!("#F9E2AF"), // 3  Yellow
            c!("#89B4FA"), // 4  Blue
            c!("#F5C2E7"), // 5  Pink
            c!("#94E2D5"), // 6  Teal
            c!("#BAC2DE"), // 7  SubText1
            c!("#585B70"), // 8  Surface2
            c!("#F38BA8"), // 9  Red (bright)
            c!("#A6E3A1"), // 10 Green (bright)
            c!("#F9E2AF"), // 11 Yellow (bright)
            c!("#89B4FA"), // 12 Blue (bright)
            c!("#F5C2E7"), // 13 Pink (bright)
            c!("#94E2D5"), // 14 Teal (bright)
            c!("#CDD6F4"), // 15 Text
        ],
        accents: [
            c!("#CBA6F7"), // Mauve
            c!("#FAB387"), // Peach
            c!("#89DCEB"), // Sky
            c!("#74C7EC"), // Sapphire
        ],
    }
}

fn make_latte() -> Theme {
    Theme {
        name: "catppuccin-latte".into(),
        bg: c!("#EFF1F5"),
        fg: c!("#4C4F69"),
        cursor: c!("#DC8A78"),
        ansi: [
            c!("#ACB0BE"), // 0  Surface1
            c!("#D20F39"), // 1  Red
            c!("#40A02B"), // 2  Green
            c!("#DF8E1D"), // 3  Yellow
            c!("#1E66F5"), // 4  Blue
            c!("#EA76CB"), // 5  Pink
            c!("#179299"), // 6  Teal
            c!("#6C6F85"), // 7  SubText1
            c!("#BCC0CC"), // 8  Surface2
            c!("#D20F39"), // 9  Red (bright)
            c!("#40A02B"), // 10 Green (bright)
            c!("#DF8E1D"), // 11 Yellow (bright)
            c!("#1E66F5"), // 12 Blue (bright)
            c!("#EA76CB"), // 13 Pink (bright)
            c!("#179299"), // 14 Teal (bright)
            c!("#4C4F69"), // 15 Text
        ],
        accents: [
            c!("#8839EF"), // Mauve
            c!("#FE640B"), // Peach
            c!("#04A5E5"), // Sky
            c!("#209FB5"), // Sapphire
        ],
    }
}

fn make_frappe() -> Theme {
    Theme {
        name: "catppuccin-frappe".into(),
        bg: c!("#303446"),
        fg: c!("#C6D0F5"),
        cursor: c!("#F2D5CF"),
        ansi: [
            c!("#51576D"), // 0  Surface1
            c!("#E78284"), // 1  Red
            c!("#A6D189"), // 2  Green
            c!("#E5C890"), // 3  Yellow
            c!("#8CAAEE"), // 4  Blue
            c!("#F4B8E4"), // 5  Pink
            c!("#81C8BE"), // 6  Teal
            c!("#B5BFE2"), // 7  SubText1
            c!("#626880"), // 8  Surface2
            c!("#E78284"), // 9  Red (bright)
            c!("#A6D189"), // 10 Green (bright)
            c!("#E5C890"), // 11 Yellow (bright)
            c!("#8CAAEE"), // 12 Blue (bright)
            c!("#F4B8E4"), // 13 Pink (bright)
            c!("#81C8BE"), // 14 Teal (bright)
            c!("#C6D0F5"), // 15 Text
        ],
        accents: [
            c!("#CA9EE6"), // Mauve
            c!("#EF9F76"), // Peach
            c!("#99D1DB"), // Sky
            c!("#85C1DC"), // Sapphire
        ],
    }
}

fn make_macchiato() -> Theme {
    Theme {
        name: "catppuccin-macchiato".into(),
        bg: c!("#24273A"),
        fg: c!("#CAD3F5"),
        cursor: c!("#F4DBD6"),
        ansi: [
            c!("#494D64"), // 0  Surface1
            c!("#ED8796"), // 1  Red
            c!("#A6DA95"), // 2  Green
            c!("#EED49F"), // 3  Yellow
            c!("#8AADF4"), // 4  Blue
            c!("#F5BDE6"), // 5  Pink
            c!("#8BD5CA"), // 6  Teal
            c!("#B8C0E0"), // 7  SubText1
            c!("#5B6078"), // 8  Surface2
            c!("#ED8796"), // 9  Red (bright)
            c!("#A6DA95"), // 10 Green (bright)
            c!("#EED49F"), // 11 Yellow (bright)
            c!("#8AADF4"), // 12 Blue (bright)
            c!("#F5BDE6"), // 13 Pink (bright)
            c!("#8BD5CA"), // 14 Teal (bright)
            c!("#CAD3F5"), // 15 Text
        ],
        accents: [
            c!("#C6A0F6"), // Mauve
            c!("#F5A97F"), // Peach
            c!("#91D7E3"), // Sky
            c!("#7DC4E4"), // Sapphire
        ],
    }
}

/// All built-in theme names in priority order.
pub const BUILTIN_NAMES: &[&str] = &[
    "catppuccin-mocha",
    "catppuccin-latte",
    "catppuccin-frappe",
    "catppuccin-macchiato",
];

/// Return the names of all built-in themes.
pub fn all_builtin_names() -> &'static [&'static str] {
    BUILTIN_NAMES
}

/// Look up a built-in theme by name.  Returns `None` for unknown names.
pub fn builtin_theme(name: &str) -> Option<Theme> {
    match name {
        "catppuccin-mocha" => Some(make_mocha()),
        "catppuccin-latte" => Some(make_latte()),
        "catppuccin-frappe" => Some(make_frappe()),
        "catppuccin-macchiato" => Some(make_macchiato()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mocha_has_correct_bg() {
        let theme = builtin_theme("catppuccin-mocha").unwrap();
        assert_eq!(theme.bg.to_hex(), "#1E1E2E");
    }

    #[test]
    fn mocha_has_correct_fg() {
        let theme = builtin_theme("catppuccin-mocha").unwrap();
        assert_eq!(theme.fg.to_hex(), "#CDD6F4");
    }

    #[test]
    fn unknown_name_returns_none() {
        assert!(builtin_theme("not-a-theme").is_none());
        assert!(builtin_theme("").is_none());
    }

    #[test]
    fn all_builtin_names_resolve() {
        for name in all_builtin_names() {
            assert!(
                builtin_theme(name).is_some(),
                "builtin_theme({name}) returned None"
            );
        }
    }

    #[test]
    fn all_themes_have_16_ansi_colors() {
        for name in all_builtin_names() {
            let theme = builtin_theme(name).unwrap();
            assert_eq!(theme.ansi.len(), 16, "theme {name} ansi len != 16");
        }
    }

    #[test]
    fn all_themes_have_4_accents() {
        for name in all_builtin_names() {
            let theme = builtin_theme(name).unwrap();
            assert_eq!(theme.accents.len(), 4, "theme {name} accents len != 4");
        }
    }

    #[test]
    fn latte_bg_is_light() {
        let theme = builtin_theme("catppuccin-latte").unwrap();
        assert_eq!(theme.bg.to_hex(), "#EFF1F5");
        // latte is a light theme — bg luminance must be high
        assert!(theme.bg.luminance() > 0.5);
    }

    #[test]
    fn mocha_bg_is_dark() {
        let theme = builtin_theme("catppuccin-mocha").unwrap();
        assert!(theme.bg.luminance() < 0.5);
    }
}
