mod content;
mod extension;
mod magic;

pub use content::detect_by_content;
pub use extension::detect_by_extension;
pub use magic::detect_by_magic;

use crate::error::Result;
use std::path::Path;

/// All file categories vidi knows how to dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileKind {
    Text,
    Markdown,
    Image,
    Video,
    Audio,
    Pdf,
    Ebook,
    OfficeDocs,
    Spreadsheet,
    Csv,
    LaTeX,
    Typst,
    Json,
    Yaml,
    Toml,
    Archive,
    Binary,
}

impl std::fmt::Display for FileKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            FileKind::Text => "text",
            FileKind::Markdown => "markdown",
            FileKind::Image => "image",
            FileKind::Video => "video",
            FileKind::Audio => "audio",
            FileKind::Pdf => "PDF",
            FileKind::Ebook => "ebook",
            FileKind::OfficeDocs => "office document",
            FileKind::Spreadsheet => "spreadsheet",
            FileKind::Csv => "CSV",
            FileKind::LaTeX => "LaTeX",
            FileKind::Typst => "Typst",
            FileKind::Json => "JSON",
            FileKind::Yaml => "YAML",
            FileKind::Toml => "TOML",
            FileKind::Archive => "archive",
            FileKind::Binary => "binary",
        };
        write!(f, "{name}")
    }
}

/// Detect the kind of file at `path` using extension → magic → content fallback.
pub fn detect(path: &Path) -> Result<FileKind> {
    if let Some(kind) = detect_by_extension(path) {
        return Ok(kind);
    }
    if let Some(kind) = detect_by_magic(path)? {
        return Ok(kind);
    }
    Ok(detect_by_content(path)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_kind_display_is_human_readable() {
        assert_eq!(FileKind::Pdf.to_string(), "PDF");
        assert_eq!(FileKind::OfficeDocs.to_string(), "office document");
        assert_eq!(FileKind::Binary.to_string(), "binary");
    }

    #[test]
    fn all_variants_display_without_panic() {
        let variants = [
            FileKind::Text,
            FileKind::Markdown,
            FileKind::Image,
            FileKind::Video,
            FileKind::Audio,
            FileKind::Pdf,
            FileKind::Ebook,
            FileKind::OfficeDocs,
            FileKind::Spreadsheet,
            FileKind::Csv,
            FileKind::LaTeX,
            FileKind::Typst,
            FileKind::Json,
            FileKind::Yaml,
            FileKind::Toml,
            FileKind::Archive,
            FileKind::Binary,
        ];
        for v in &variants {
            assert!(!v.to_string().is_empty());
        }
    }
}
