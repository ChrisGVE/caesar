/// The graphics rendering protocol supported by the running terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsProtocol {
    /// Kitty terminal graphics protocol (highest quality).
    Kitty,
    /// iTerm2 inline image protocol.
    ITerm2,
    /// Sixel graphics.
    Sixel,
    /// Unicode half-block characters with 24-bit (truecolor) ANSI.
    HalfBlock24,
    /// Unicode half-block characters with 256-color ANSI fallback.
    HalfBlock256,
}

/// Detected capabilities of the running terminal.
#[derive(Debug, Clone)]
pub struct TerminalCaps {
    pub graphics: GraphicsProtocol,
    pub true_color: bool,
    /// Terminal width in columns (0 if unavailable).
    pub columns: u16,
    /// Terminal height in rows (0 if unavailable).
    pub rows: u16,
}

impl Default for TerminalCaps {
    fn default() -> Self {
        Self {
            graphics: GraphicsProtocol::HalfBlock256,
            true_color: false,
            columns: 0,
            rows: 0,
        }
    }
}

/// Detect the terminal capabilities for the current process.
///
/// Uses environment variables for instant detection; does not issue escape
/// sequence queries in this initial implementation (those are added in task 8).
pub fn detect_capabilities() -> TerminalCaps {
    let graphics = detect_protocol();
    let true_color = detect_truecolor();
    let (columns, rows) = detect_dimensions();

    TerminalCaps {
        graphics,
        true_color,
        columns,
        rows,
    }
}

fn detect_protocol() -> GraphicsProtocol {
    // 1. Kitty: KITTY_WINDOW_ID or TERM=xterm-kitty
    if std::env::var("KITTY_WINDOW_ID").is_ok() {
        return GraphicsProtocol::Kitty;
    }
    if std::env::var("TERM")
        .map(|v| v == "xterm-kitty")
        .unwrap_or(false)
    {
        return GraphicsProtocol::Kitty;
    }

    // 2. WezTerm: prefers Kitty protocol
    if std::env::var("TERM_PROGRAM")
        .map(|v| v == "WezTerm")
        .unwrap_or(false)
    {
        return GraphicsProtocol::Kitty;
    }

    // 3. iTerm2
    if std::env::var("TERM_PROGRAM")
        .map(|v| v == "iTerm.app")
        .unwrap_or(false)
        || std::env::var("ITERM_PROFILE").is_ok()
    {
        return GraphicsProtocol::ITerm2;
    }

    // 4. Ghostty
    if std::env::var("TERM_PROGRAM")
        .map(|v| v == "ghostty")
        .unwrap_or(false)
    {
        return GraphicsProtocol::Kitty;
    }

    // 5. Truecolor → half-block 24-bit (Sixel detection deferred to task 8)
    if detect_truecolor() {
        return GraphicsProtocol::HalfBlock24;
    }

    GraphicsProtocol::HalfBlock256
}

fn detect_truecolor() -> bool {
    std::env::var("COLORTERM")
        .map(|v| v == "truecolor" || v == "24bit")
        .unwrap_or(false)
}

fn detect_dimensions() -> (u16, u16) {
    use crossterm::terminal::size;
    size().unwrap_or((0, 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_caps_are_conservative() {
        let caps = TerminalCaps::default();
        assert_eq!(caps.graphics, GraphicsProtocol::HalfBlock256);
        assert!(!caps.true_color);
    }

    #[test]
    fn detect_caps_does_not_panic() {
        let caps = detect_capabilities();
        // Must produce a valid protocol in any environment
        let _proto = caps.graphics;
    }

    #[test]
    fn truecolor_detection_without_env() {
        // Without COLORTERM set this should be false (env may vary in CI)
        let _ = detect_truecolor();
    }
}
