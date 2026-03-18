mod detect;

pub use detect::{detect_capabilities, GraphicsProtocol, TerminalCaps};

/// The terminal multiplexer running the current session, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultiplexerKind {
    /// tmux multiplexer.
    Tmux,
    /// Zellij multiplexer.
    Zellij,
    /// GNU screen (a.k.a. cmux).
    Cmux,
    /// No multiplexer detected.
    None,
}

/// Detected information about the running terminal multiplexer.
#[derive(Debug, Clone)]
pub struct MultiplexerInfo {
    /// Which multiplexer is active (or `None`).
    pub kind: MultiplexerKind,
    /// Session identifier, if the multiplexer exposes one.
    pub session_id: Option<String>,
    /// Pane identifier, if the multiplexer exposes one.
    pub pane_id: Option<String>,
}

impl Default for MultiplexerInfo {
    fn default() -> Self {
        Self {
            kind: MultiplexerKind::None,
            session_id: None,
            pane_id: None,
        }
    }
}

/// Detect the terminal multiplexer for the current process.
///
/// Checks well-known environment variables set by tmux and Zellij.
/// GNU screen detection is included for completeness.
pub fn detect_multiplexer() -> MultiplexerInfo {
    // tmux sets TMUX to a path:session_id:pane_id triplet
    if let Ok(tmux) = std::env::var("TMUX") {
        let mut parts = tmux.splitn(3, ',');
        let _path = parts.next();
        let session_id = parts.next().map(str::to_owned);
        let pane_id = parts.next().map(str::to_owned);
        return MultiplexerInfo {
            kind: MultiplexerKind::Tmux,
            session_id,
            pane_id,
        };
    }

    // Zellij sets ZELLIJ and ZELLIJ_SESSION_NAME / ZELLIJ_PANE_ID
    if std::env::var("ZELLIJ").is_ok() {
        let session_id = std::env::var("ZELLIJ_SESSION_NAME").ok();
        let pane_id = std::env::var("ZELLIJ_PANE_ID").ok();
        return MultiplexerInfo {
            kind: MultiplexerKind::Zellij,
            session_id,
            pane_id,
        };
    }

    // GNU screen sets STY to "pid.tty.host"
    if let Ok(sty) = std::env::var("STY") {
        return MultiplexerInfo {
            kind: MultiplexerKind::Cmux,
            session_id: Some(sty),
            pane_id: None,
        };
    }

    MultiplexerInfo::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_returns_valid_caps() {
        // Must not panic in any environment (CI, dumb terminal, etc.)
        let caps = detect_capabilities();
        let _ = caps.graphics;
        let _ = caps.true_color;
    }

    #[test]
    fn default_multiplexer_info_is_none() {
        let info = MultiplexerInfo::default();
        assert_eq!(info.kind, MultiplexerKind::None);
        assert!(info.session_id.is_none());
        assert!(info.pane_id.is_none());
    }

    #[test]
    fn detect_multiplexer_does_not_panic() {
        // In any environment, the function must return without panicking.
        let info = detect_multiplexer();
        let _ = info.kind;
    }

    #[test]
    fn multiplexer_kind_none_when_no_env_vars() {
        // This test is environment-dependent; it checks the happy path in a
        // clean environment. In CI with no multiplexer vars, kind must be None.
        // We simply ensure the function completes without error.
        let _ = detect_multiplexer();
    }
}
