mod epub;
mod html;
pub mod image;

use crate::detector::FileKind;
use crate::error::Result;
use crate::terminal::TerminalCaps;
use std::path::Path;

pub use html::render_with_resolver as render_html_with_resolver;

/// Returns `true` if an internal renderer exists for `kind`.
pub fn has_internal_renderer(kind: FileKind) -> bool {
    matches!(kind, FileKind::Html | FileKind::Ebook)
}

/// Attempt to render `path` using the internal renderer for `kind`.
///
/// Returns:
/// - `Some(Ok(bytes))` — ANSI bytes ready for stdout
/// - `Some(Err(_))` — renderer attempted but failed; fall through to external
/// - `None` — no internal renderer for this kind; fall through to external
///
/// `max_lines` is used for inline line-limit enforcement by the caller.
/// `inline` controls image sizing (compact vs. full-screen proportions).
pub fn internal_render(
    kind: FileKind,
    path: &Path,
    caps: &TerminalCaps,
    max_lines: u16,
    inline: bool,
) -> Option<Result<Vec<u8>>> {
    match kind {
        FileKind::Html => Some(html::render(path, caps, inline, max_lines)),
        FileKind::Ebook => epub::render(path, caps, inline, max_lines),
        _ => None,
    }
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
    fn has_renderer_for_html_and_ebook() {
        assert!(has_internal_renderer(FileKind::Html));
        assert!(has_internal_renderer(FileKind::Ebook));
    }

    #[test]
    fn no_renderer_for_text_pdf_image() {
        assert!(!has_internal_renderer(FileKind::Text));
        assert!(!has_internal_renderer(FileKind::Pdf));
        assert!(!has_internal_renderer(FileKind::Image));
        assert!(!has_internal_renderer(FileKind::Markdown));
    }

    #[test]
    fn internal_render_returns_none_for_text() {
        let result = internal_render(FileKind::Text, Path::new("dummy.txt"), &caps(), 50, true);
        assert!(result.is_none());
    }

    #[test]
    fn internal_render_html_missing_file_returns_some_err() {
        let result = internal_render(
            FileKind::Html,
            Path::new("/nonexistent/path/file.html"),
            &caps(),
            50,
            true,
        );
        assert!(result.is_some(), "expected Some for Html kind");
        assert!(result.unwrap().is_err(), "expected Err for missing file");
    }
}
