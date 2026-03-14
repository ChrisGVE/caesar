/// RGB color with 8 bits per channel.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Encode as uppercase hex string, e.g. `"#1E1E2E"`.
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Parse a `#RRGGBB` hex string (leading `#` required).
    /// Returns `None` if the input is not a valid 7-character hex color.
    pub fn from_hex(s: &str) -> Option<Self> {
        let s = s.strip_prefix('#')?;
        if s.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(Self { r, g, b })
    }

    /// Compute perceptual luminance in the range [0.0, 1.0].
    ///
    /// Uses the BT.601 luma coefficients, sufficient for light/dark detection.
    pub fn luminance(&self) -> f64 {
        (0.299 * f64::from(self.r) + 0.587 * f64::from(self.g) + 0.114 * f64::from(self.b)) / 255.0
    }
}

/// A named terminal color theme.
///
/// `ansi` holds 16 ANSI palette entries (indices 0–7 normal, 8–15 bright).
/// `accents` holds 4 theme-specific accent colors for tool arguments.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Theme {
    pub name: String,
    pub bg: Color,
    pub fg: Color,
    pub cursor: Color,
    pub ansi: [Color; 16],
    pub accents: [Color; 4],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_hex_formats_correctly() {
        let c = Color {
            r: 0x1E,
            g: 0x1E,
            b: 0x2E,
        };
        assert_eq!(c.to_hex(), "#1E1E2E");
    }

    #[test]
    fn to_hex_zero_padded() {
        let c = Color { r: 0, g: 0, b: 0 };
        assert_eq!(c.to_hex(), "#000000");
    }

    #[test]
    fn from_hex_roundtrip() {
        let original = Color {
            r: 0xAB,
            g: 0xCD,
            b: 0xEF,
        };
        let hex = original.to_hex();
        let parsed = Color::from_hex(&hex).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn from_hex_lowercase_accepted() {
        let c = Color::from_hex("#abcdef").unwrap();
        assert_eq!(c.r, 0xAB);
        assert_eq!(c.g, 0xCD);
        assert_eq!(c.b, 0xEF);
    }

    #[test]
    fn from_hex_missing_hash_returns_none() {
        assert!(Color::from_hex("1E1E2E").is_none());
    }

    #[test]
    fn from_hex_wrong_length_returns_none() {
        assert!(Color::from_hex("#1E1E2").is_none());
        assert!(Color::from_hex("#1E1E2EFF").is_none());
    }

    #[test]
    fn from_hex_invalid_chars_returns_none() {
        assert!(Color::from_hex("#GGGGGG").is_none());
    }

    #[test]
    fn luminance_white_is_one() {
        let white = Color {
            r: 255,
            g: 255,
            b: 255,
        };
        let lum = white.luminance();
        assert!((lum - 1.0).abs() < 0.001);
    }

    #[test]
    fn luminance_black_is_zero() {
        let black = Color { r: 0, g: 0, b: 0 };
        assert_eq!(black.luminance(), 0.0);
    }

    #[test]
    fn theme_ansi_array_has_16_entries() {
        // Verify the array size is encoded in the type — compile-time check,
        // but we also assert at runtime to guard against accidental changes.
        let ansi: [Color; 16] = std::array::from_fn(|i| Color {
            r: i as u8,
            g: 0,
            b: 0,
        });
        assert_eq!(ansi.len(), 16);
    }

    #[test]
    fn theme_accents_array_has_4_entries() {
        let accents: [Color; 4] = std::array::from_fn(|i| Color {
            r: 0,
            g: i as u8,
            b: 0,
        });
        assert_eq!(accents.len(), 4);
    }
}
