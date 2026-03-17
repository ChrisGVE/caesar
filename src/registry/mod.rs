mod probe;
mod tools;
mod tools_data;

pub use probe::is_available;
pub use probe::resolve_tool;
pub use tools::{ToolSpec, REGISTRY};
pub use tools_data::TEXT_TOOLS;

use crate::detector::FileKind;

/// Resolve the best available tool for `kind`, respecting any user override.
pub fn best_tool(kind: FileKind, override_name: Option<&str>) -> Option<&'static ToolSpec> {
    if let Some(name) = override_name {
        return REGISTRY
            .iter()
            .flat_map(|(_, specs)| specs.iter())
            .find(|s| s.name == name);
    }
    resolve_tool(kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_entry_for_every_file_kind() {
        let kinds = [
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
        for kind in &kinds {
            assert!(
                REGISTRY.iter().any(|(k, _)| k == kind),
                "No registry entry for {kind}"
            );
        }
    }

    #[test]
    fn every_registry_entry_has_at_least_one_spec() {
        for (kind, specs) in REGISTRY.iter() {
            assert!(!specs.is_empty(), "Empty spec list for {kind}");
        }
    }
}
