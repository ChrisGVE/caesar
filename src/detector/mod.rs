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
    /// Plain text and source code (`.txt`, `.rs`, `.py`, `.sh`, …).
    Text,
    /// Markdown documents (`.md`, `.markdown`, `.mdx`, …).
    Markdown,
    /// Raster and vector images (`.jpg`, `.png`, `.gif`, `.svg`, `.webp`, …).
    Image,
    /// Video files (`.mp4`, `.mkv`, `.mov`, `.webm`, …).
    Video,
    /// Audio files (`.mp3`, `.flac`, `.ogg`, `.wav`, …).
    Audio,
    /// PDF documents (`.pdf`).
    Pdf,
    /// Ebook formats (`.epub`, `.mobi`, `.djvu`, …).
    Ebook,
    /// HTML documents (`.html`, `.htm`, `.xhtml`).
    Html,
    /// Office documents (`.docx`, `.odt`, `.pptx`, `.pages`, …).
    OfficeDocs,
    /// Spreadsheet files (`.xlsx`, `.ods`, `.numbers`, …).
    Spreadsheet,
    /// Delimited tabular data (`.csv`, `.tsv`, `.psv`).
    Csv,
    /// LaTeX source files (`.tex`, `.sty`, `.cls`, `.bib`, …).
    LaTeX,
    /// Typst source files (`.typ`).
    Typst,
    /// JSON and JSON-adjacent formats (`.json`, `.jsonl`, `.json5`, …).
    Json,
    /// YAML files (`.yaml`, `.yml`).
    Yaml,
    /// TOML files (`.toml`).
    Toml,
    /// Compressed archives and packages (`.tar.gz`, `.zip`, `.7z`, `.deb`, …).
    Archive,
    /// Unrecognised binary files; rendered as a hex dump.
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
            FileKind::Html => "HTML",
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

impl FileKind {
    /// Return the lowercase config key used in the `[viewer]` table.
    pub fn config_key(self) -> &'static str {
        match self {
            FileKind::Text => "text",
            FileKind::Markdown => "markdown",
            FileKind::Image => "image",
            FileKind::Video => "video",
            FileKind::Audio => "audio",
            FileKind::Pdf => "pdf",
            FileKind::Ebook => "ebook",
            FileKind::Html => "html",
            FileKind::OfficeDocs => "office",
            FileKind::Spreadsheet => "spreadsheet",
            FileKind::Csv => "csv",
            FileKind::LaTeX => "latex",
            FileKind::Typst => "typst",
            FileKind::Json => "json",
            FileKind::Yaml => "yaml",
            FileKind::Toml => "toml",
            FileKind::Archive => "archive",
            FileKind::Binary => "binary",
        }
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
    detect_by_content(path)
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
            FileKind::Html,
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

    #[test]
    fn html_display_is_uppercase() {
        assert_eq!(FileKind::Html.to_string(), "HTML");
    }

    #[test]
    fn config_keys_are_unique_and_non_empty() {
        use std::collections::HashSet;
        let variants = [
            FileKind::Text,
            FileKind::Markdown,
            FileKind::Image,
            FileKind::Video,
            FileKind::Audio,
            FileKind::Pdf,
            FileKind::Ebook,
            FileKind::Html,
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
        let mut seen = HashSet::new();
        for v in variants {
            let key = v.config_key();
            assert!(!key.is_empty(), "empty config_key for {v}");
            assert!(seen.insert(key), "duplicate config_key '{key}' for {v}");
        }
    }

    #[test]
    fn html_config_key() {
        assert_eq!(FileKind::Html.config_key(), "html");
    }
}
