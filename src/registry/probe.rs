use super::tools::{ToolSpec, REGISTRY};
use crate::detector::FileKind;
use std::collections::HashMap;
use std::sync::OnceLock;

// Lazily populated per-binary availability cache for this process.
static PROBE_CACHE: OnceLock<std::sync::Mutex<HashMap<&'static str, bool>>> = OnceLock::new();

/// Return `true` if `binary` is found on PATH.
///
/// Results are cached in-process so each binary is probed at most once per
/// vidi invocation.
pub fn is_available(binary: &'static str) -> bool {
    let cache = PROBE_CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()));
    let mut guard = cache.lock().unwrap();
    *guard
        .entry(binary)
        .or_insert_with(|| which::which(binary).is_ok())
}

/// Return the first available `ToolSpec` for the given `FileKind`, or `None`
/// if none of the candidates are installed.
pub fn resolve_tool(kind: FileKind) -> Option<&'static ToolSpec> {
    REGISTRY
        .iter()
        .find(|(k, _)| *k == kind)
        .and_then(|(_, specs)| specs.iter().find(|s| is_available(s.binary)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cat_is_always_available() {
        // `cat` is POSIX-mandated and must be present on all target platforms.
        assert!(is_available("cat"), "cat should be on PATH");
    }

    #[test]
    fn probe_cache_returns_same_result_on_repeated_calls() {
        let first = is_available("cat");
        let second = is_available("cat");
        assert_eq!(first, second);
    }

    #[test]
    fn resolve_text_returns_some_tool() {
        // cat is always present, so resolving Text must succeed.
        let tool = resolve_tool(FileKind::Text);
        assert!(tool.is_some(), "Expected a tool for Text, found none");
    }

    #[test]
    fn resolve_binary_returns_some_tool() {
        // xxd ships with vim and is universally available.
        // If xxd is absent, hexyl may also be absent; this test is
        // best-effort and documents the expectation.
        let tool = resolve_tool(FileKind::Binary);
        assert!(tool.is_some(), "Expected a tool for Binary, found none");
    }

    #[test]
    fn nonexistent_binary_is_not_available() {
        // A binary that cannot possibly exist.
        assert!(!which::which("__vidi_nonexistent_tool_xyz__").is_ok());
    }
}
