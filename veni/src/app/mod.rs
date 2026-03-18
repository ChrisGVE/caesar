use caesar_common::terminal::TerminalCaps;

/// Input mode for the modal editing model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Navigation and file operations.
    Normal,
    /// Text input (rename, search, command palette).
    Insert,
    /// Multi-file selection.
    Visual,
    /// Ex-style command input.
    Command,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Normal => write!(f, "NORMAL"),
            Mode::Insert => write!(f, "INSERT"),
            Mode::Visual => write!(f, "VISUAL"),
            Mode::Command => write!(f, "COMMAND"),
        }
    }
}

/// Core application state.
pub struct App {
    pub mode: Mode,
    pub cwd: std::path::PathBuf,
    pub caps: TerminalCaps,
    pub should_quit: bool,
}

impl App {
    pub fn new(path: std::path::PathBuf, caps: TerminalCaps) -> Self {
        Self {
            mode: Mode::Normal,
            cwd: path,
            caps,
            should_quit: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mode_is_normal() {
        assert_eq!(Mode::default(), Mode::Normal);
    }

    #[test]
    fn mode_display() {
        assert_eq!(Mode::Normal.to_string(), "NORMAL");
        assert_eq!(Mode::Insert.to_string(), "INSERT");
        assert_eq!(Mode::Visual.to_string(), "VISUAL");
        assert_eq!(Mode::Command.to_string(), "COMMAND");
    }

    #[test]
    fn app_starts_in_normal_mode() {
        let app = App::new(std::path::PathBuf::from("/tmp"), TerminalCaps::default());
        assert_eq!(app.mode, Mode::Normal);
        assert!(!app.should_quit);
    }
}
