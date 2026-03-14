use crate::detector::FileKind;

use super::tools_data::{
    ARCHIVE_TOOLS, AUDIO_TOOLS, BINARY_TOOLS, CSV_TOOLS, EBOOK_TOOLS, IMAGE_TOOLS, JSON_TOOLS,
    LATEX_TOOLS, MARKDOWN_TOOLS, OFFICE_TOOLS, PDF_TOOLS, SPREADSHEET_TOOLS, TEXT_TOOLS,
    TOML_TOOLS, TYPST_TOOLS, VIDEO_TOOLS, YAML_TOOLS,
};

/// Specification for a single delegated viewer tool.
#[derive(Debug, Clone)]
pub struct ToolSpec {
    /// Short name used in config overrides and error messages.
    pub name: &'static str,
    /// Executable name searched on PATH.
    pub binary: &'static str,
    /// Arguments for full-screen invocation.
    /// `{theme}`, `{cols}`, `{rows}` are placeholder tokens replaced at runtime.
    pub fullscreen_args: &'static [&'static str],
    /// Arguments for inline (constrained stdout) invocation.
    pub inline_args: &'static [&'static str],
    pub supports_inline: bool,
    pub supports_fullscreen: bool,
    pub supports_theming: bool,
}

/// Static registry: maps each `FileKind` to an ordered list of candidate tools.
/// The first installed tool in each list is used.
pub static REGISTRY: &[(FileKind, &[ToolSpec])] = &[
    (FileKind::Text, TEXT_TOOLS),
    (FileKind::Markdown, MARKDOWN_TOOLS),
    (FileKind::Image, IMAGE_TOOLS),
    (FileKind::Video, VIDEO_TOOLS),
    (FileKind::Audio, AUDIO_TOOLS),
    (FileKind::Pdf, PDF_TOOLS),
    (FileKind::Ebook, EBOOK_TOOLS),
    (FileKind::OfficeDocs, OFFICE_TOOLS),
    (FileKind::Spreadsheet, SPREADSHEET_TOOLS),
    (FileKind::Csv, CSV_TOOLS),
    (FileKind::LaTeX, LATEX_TOOLS),
    (FileKind::Typst, TYPST_TOOLS),
    (FileKind::Json, JSON_TOOLS),
    (FileKind::Yaml, YAML_TOOLS),
    (FileKind::Toml, TOML_TOOLS),
    (FileKind::Archive, ARCHIVE_TOOLS),
    (FileKind::Binary, BINARY_TOOLS),
];

#[cfg(test)]
mod tests {
    use super::super::tools_data::BINARY_TOOLS;
    use super::*;

    #[test]
    fn all_tool_specs_have_non_empty_name_and_binary() {
        for (_, specs) in REGISTRY {
            for spec in *specs {
                assert!(!spec.name.is_empty(), "ToolSpec has empty name");
                assert!(
                    !spec.binary.is_empty(),
                    "ToolSpec has empty binary for {}",
                    spec.name
                );
            }
        }
    }

    #[test]
    fn text_tools_include_cat_fallback() {
        assert!(TEXT_TOOLS.iter().any(|s| s.binary == "cat"));
    }

    #[test]
    fn binary_tools_include_xxd_fallback() {
        assert!(BINARY_TOOLS.iter().any(|s| s.binary == "xxd"));
    }
}
