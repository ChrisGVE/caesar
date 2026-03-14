use std::io::Write as IoWrite;
use std::path::Path;
use std::process::Stdio;

use crate::{
    error::{Result, VidiError},
    registry::ToolSpec,
    terminal::TerminalCaps,
    theme::ThemeMapper,
};

use super::args::build_args;

/// Spawn the tool in inline mode, capture its stdout, truncate it to
/// `lines` lines (ANSI-safe), and write the result to our stdout.
pub fn launch_inline(
    spec: &ToolSpec,
    file: &Path,
    mapper: &ThemeMapper<'_>,
    caps: &TerminalCaps,
    lines: u16,
) -> Result<()> {
    // build_args already appends the file path as the last element.
    let final_args = build_args(spec, file, mapper, caps, lines, true);

    let output = std::process::Command::new(spec.binary)
        .args(&final_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map_err(|_| VidiError::ToolNotFound {
            tool: spec.binary.to_string(),
        })?;

    if !output.status.success() {
        let code = output.status.code().unwrap_or(1);
        return Err(VidiError::ToolFailed {
            tool: spec.binary.to_string(),
            code,
        });
    }

    let truncated = truncate_ansi_safe(&output.stdout, lines);

    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    lock.write_all(&truncated)?;
    Ok(())
}

/// Truncate `output` to at most `max_lines` lines without cutting ANSI escape
/// sequences. Emits `ESC[0m` (reset) after the last line if ANSI sequences
/// were present.
///
/// An "ANSI sequence" is detected by the presence of any ESC byte (`0x1b`).
pub fn truncate_ansi_safe(output: &[u8], max_lines: u16) -> Vec<u8> {
    if output.is_empty() || max_lines == 0 {
        return Vec::new();
    }

    let had_ansi = output.contains(&0x1b);
    let max = usize::from(max_lines);

    // Split on newline, keep the first `max` lines.
    // We re-insert the newline after each line except possibly the last.
    let mut result: Vec<u8> = Vec::with_capacity(output.len());
    let mut line_count = 0usize;
    let mut start = 0usize;

    for (i, &byte) in output.iter().enumerate() {
        if byte == b'\n' {
            line_count += 1;
            result.extend_from_slice(&output[start..=i]);
            start = i + 1;
            if line_count >= max {
                break;
            }
        }
    }

    // If we never hit max_lines lines, include any remaining partial line.
    if line_count < max && start < output.len() {
        result.extend_from_slice(&output[start..]);
    }

    // Append ANSI reset if the original contained escape sequences.
    if had_ansi && !result.is_empty() {
        result.extend_from_slice(b"\x1b[0m\n");
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_returns_empty() {
        assert_eq!(truncate_ansi_safe(&[], 10), Vec::<u8>::new());
    }

    #[test]
    fn max_lines_zero_returns_empty() {
        assert_eq!(truncate_ansi_safe(b"hello\nworld\n", 0), Vec::<u8>::new());
    }

    #[test]
    fn fewer_lines_than_max_unchanged() {
        let input = b"line1\nline2\n";
        let result = truncate_ansi_safe(input, 10);
        assert_eq!(result, input.to_vec());
    }

    #[test]
    fn exactly_max_lines_unchanged() {
        let input = b"a\nb\nc\n";
        let result = truncate_ansi_safe(input, 3);
        assert_eq!(result, input.to_vec());
    }

    #[test]
    fn over_max_lines_truncated() {
        let input = b"one\ntwo\nthree\nfour\n";
        let result = truncate_ansi_safe(input, 2);
        assert_eq!(result, b"one\ntwo\n".to_vec());
    }

    #[test]
    fn ansi_reset_appended_when_esc_present() {
        let input = b"\x1b[32mhello\x1b[0m\nworld\n";
        let result = truncate_ansi_safe(input, 10);
        assert!(result.ends_with(b"\x1b[0m\n"), "expected ANSI reset at end");
    }

    #[test]
    fn no_reset_when_no_esc_bytes() {
        let input = b"plain\ntext\n";
        let result = truncate_ansi_safe(input, 10);
        assert!(!result.contains(&0x1b), "no ESC expected in plain output");
        assert_eq!(result, input.to_vec());
    }

    #[test]
    fn ansi_reset_appended_only_once_when_over_limit() {
        let input = b"\x1b[31mred\x1b[0m\nline2\nline3\n";
        let result = truncate_ansi_safe(input, 1);
        // Only the first line + reset
        assert_eq!(result, b"\x1b[31mred\x1b[0m\n\x1b[0m\n".to_vec());
    }
}
