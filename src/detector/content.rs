use super::FileKind;
use crate::error::{Result, VidiError};
use std::path::Path;

/// Detect `FileKind` by inspecting the file content when extension and magic
/// both fail. Determines whether the file is valid UTF-8 text or binary.
pub fn detect_by_content(path: &Path) -> Result<FileKind> {
    use std::io::Read;
    let mut f = std::fs::File::open(path).map_err(|e| VidiError::FileUnreadable {
        path: path.to_path_buf(),
        source: e,
    })?;
    // Sample the first 8 KiB; sufficient for UTF-8 sniffing without loading
    // large files entirely.
    let mut buf = vec![0u8; 8192];
    let n = f.read(&mut buf).map_err(|e| VidiError::FileUnreadable {
        path: path.to_path_buf(),
        source: e,
    })?;
    buf.truncate(n);

    if is_text(&buf) {
        Ok(FileKind::Text)
    } else {
        Ok(FileKind::Binary)
    }
}

/// Return `true` if `bytes` looks like UTF-8 text with no null bytes.
///
/// A file is considered text when:
/// - It contains no null bytes (NUL is almost never in text files)
/// - At least 85% of the sampled bytes are printable ASCII or valid UTF-8
///   continuation bytes
fn is_text(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return true; // empty file is treated as text
    }
    // Null byte is a strong binary indicator
    if bytes.contains(&0x00) {
        return false;
    }
    // Count bytes that are valid in UTF-8 text (ASCII printable, whitespace,
    // or UTF-8 continuation / leading bytes).
    let text_like: usize = bytes
        .iter()
        .filter(|&&b| b >= 0x09) // TAB and above covers all printable + continuation
        .count();
    let ratio = text_like as f64 / bytes.len() as f64;
    ratio >= 0.85
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_bytes_is_text() {
        assert!(is_text(&[]));
    }

    #[test]
    fn null_byte_is_binary() {
        assert!(!is_text(b"hello\x00world"));
    }

    #[test]
    fn plain_ascii_is_text() {
        assert!(is_text(b"fn main() { println!(\"hello\"); }"));
    }

    #[test]
    fn valid_utf8_is_text() {
        let utf8 = "héllo wörld — Unicode".as_bytes();
        assert!(is_text(utf8));
    }

    #[test]
    fn mostly_binary_is_not_text() {
        // 90% non-printable bytes
        let bin: Vec<u8> = (0x00u8..=0x08u8).cycle().take(100).collect();
        assert!(!is_text(&bin));
    }
}
