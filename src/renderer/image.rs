use crate::registry::is_available;
use crate::terminal::TerminalCaps;
use std::io::Write as _;
use tempfile::NamedTempFile;

/// Render raw image bytes to ANSI terminal output via chafa, or emit a
/// placeholder `[image: {filename}]` line if chafa is not installed.
///
/// `max_lines` is the maximum number of terminal rows to use for the image.
pub fn render_image(bytes: &[u8], filename: &str, caps: &TerminalCaps, max_lines: u16) -> Vec<u8> {
    if !is_available("chafa") {
        return format!("[image: {filename}]\n").into_bytes();
    }

    match render_via_chafa(bytes, filename, caps, max_lines) {
        Ok(output) => output,
        Err(_) => format!("[image: {filename}]\n").into_bytes(),
    }
}

fn render_via_chafa(
    bytes: &[u8],
    filename: &str,
    caps: &TerminalCaps,
    max_lines: u16,
) -> std::io::Result<Vec<u8>> {
    let mut tmp = NamedTempFile::new()?;
    tmp.write_all(bytes)?;
    tmp.flush()?;

    let cols = if caps.columns > 0 { caps.columns } else { 80 };
    let size_arg = format!("{}x{}", cols, max_lines);

    let output = std::process::Command::new("chafa")
        .args(["--format=symbols", &format!("--size={size_arg}")])
        .arg(tmp.path())
        .output()?;

    if output.status.success() {
        Ok(output.stdout)
    } else {
        Ok(format!("[image: {filename}]\n").into_bytes())
    }
}

/// Calculate the maximum number of image lines for inline vs full-screen mode.
pub fn max_image_lines(caps: &TerminalCaps, inline: bool) -> u16 {
    let cols = if caps.columns > 0 { caps.columns } else { 80 };
    let rows = if caps.rows > 0 { caps.rows } else { 24 };
    if inline {
        (cols / 2).min(20)
    } else {
        (rows / 3).min(40)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::{GraphicsProtocol, TerminalCaps};

    fn dummy_caps() -> TerminalCaps {
        TerminalCaps {
            graphics: GraphicsProtocol::HalfBlock256,
            true_color: false,
            columns: 80,
            rows: 24,
        }
    }

    #[test]
    fn missing_chafa_returns_placeholder() {
        // chafa may or may not be installed; if absent the placeholder is used.
        let result = render_image(b"\x89PNG", "test.png", &dummy_caps(), 10);
        if !is_available("chafa") {
            assert_eq!(result, b"[image: test.png]\n");
        }
    }

    #[test]
    fn max_image_lines_inline_caps_at_20() {
        let caps = TerminalCaps {
            columns: 200,
            rows: 50,
            ..dummy_caps()
        };
        assert_eq!(max_image_lines(&caps, true), 20);
    }

    #[test]
    fn max_image_lines_fullscreen_caps_at_40() {
        let caps = TerminalCaps {
            columns: 80,
            rows: 200,
            ..dummy_caps()
        };
        assert_eq!(max_image_lines(&caps, false), 40);
    }

    #[test]
    fn max_image_lines_inline_uses_half_cols() {
        let caps = TerminalCaps {
            columns: 80,
            rows: 24,
            ..dummy_caps()
        };
        // 80/2 = 40, capped at 20
        assert_eq!(max_image_lines(&caps, true), 20);
    }

    #[test]
    fn max_image_lines_fullscreen_uses_third_rows() {
        let caps = TerminalCaps {
            columns: 80,
            rows: 60,
            ..dummy_caps()
        };
        // 60/3 = 20, within cap of 40
        assert_eq!(max_image_lines(&caps, false), 20);
    }

    #[test]
    fn max_image_lines_zero_dims_use_fallback() {
        let caps = TerminalCaps {
            columns: 0,
            rows: 0,
            ..dummy_caps()
        };
        // cols=80 fallback: 80/2=40, cap 20 → 20
        assert_eq!(max_image_lines(&caps, true), 20);
        // rows=24 fallback: 24/3=8, within cap 40 → 8
        assert_eq!(max_image_lines(&caps, false), 8);
    }
}
