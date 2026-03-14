use std::path::Path;

use crate::{
    error::Result,
    registry::{is_available, TEXT_TOOLS},
    terminal::TerminalCaps,
    theme::ThemeMapper,
};

use super::fullscreen::launch_fullscreen;

/// Launch toggle mode for LaTeX or Typst files (rendered source view).
///
/// This is currently a stub that falls back to a `bat` source view.
/// Full compile-and-render toggle is planned for task 16/17.
pub fn launch_toggle(file: &Path, mapper: &ThemeMapper<'_>, caps: &TerminalCaps) -> Result<()> {
    // TODO(task-16/17): implement compile-and-render toggle loop.
    // For now, fall back to bat source view when available.
    if is_available("bat") {
        let bat = TEXT_TOOLS.iter().find(|s| s.binary == "bat").unwrap();
        launch_fullscreen(bat, file, mapper, caps)
    } else {
        let fallback = TEXT_TOOLS.last().unwrap();
        launch_fullscreen(fallback, file, mapper, caps)
    }
}
