mod detect;

pub use detect::{detect_capabilities, GraphicsProtocol, TerminalCaps};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_returns_valid_caps() {
        // Must not panic in any environment (CI, dumb terminal, etc.)
        let caps = detect_capabilities();
        // columns and rows may be 0 in non-interactive environments
        let _ = caps.graphics;
        let _ = caps.true_color;
    }
}
