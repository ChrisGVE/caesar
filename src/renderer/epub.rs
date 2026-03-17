use super::html;
use crate::error::Result;
use crate::terminal::TerminalCaps;
use rbook::Epub;
use std::path::Path;

const CHAPTER_SEPARATOR: &[u8] = b"\n\x1b[2m---\x1b[0m\n";

/// Render an epub at `path` as ANSI text.
///
/// Returns:
/// - `None` if `path` is not a valid epub (mobi, djvu, etc.)
/// - `Some(Err(_))` if the epub structure is unreadable
/// - `Some(Ok(bytes))` on success
pub fn render(
    path: &Path,
    caps: &TerminalCaps,
    inline: bool,
    max_lines: u16,
) -> Option<Result<Vec<u8>>> {
    let epub = Epub::open(path).ok()?;
    Some(render_epub(&epub, caps, inline, max_lines))
}

fn render_epub(epub: &Epub, caps: &TerminalCaps, inline: bool, max_lines: u16) -> Result<Vec<u8>> {
    let mut out: Vec<u8> = Vec::new();
    let mut first = true;

    for content_result in epub.reader() {
        let content = match content_result {
            Ok(c) => c,
            Err(_) => continue,
        };

        let html_bytes = content.into_bytes();

        if !first {
            out.extend_from_slice(CHAPTER_SEPARATOR);
        }
        first = false;

        let resolver = |src: &str| epub.read_resource_bytes(src).ok();
        match html::render_with_resolver(&html_bytes, caps, inline, max_lines, &resolver) {
            Ok(rendered) => out.extend_from_slice(&rendered),
            Err(e) => return Err(e),
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::{GraphicsProtocol, TerminalCaps};

    fn caps() -> TerminalCaps {
        TerminalCaps {
            graphics: GraphicsProtocol::HalfBlock256,
            true_color: false,
            columns: 80,
            rows: 24,
        }
    }

    #[test]
    fn non_epub_file_returns_none() {
        let path = Path::new("Cargo.toml");
        let result = render(path, &caps(), true, 50);
        assert!(result.is_none(), "expected None for non-epub file");
    }
}
