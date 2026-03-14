//! Crossterm raw-mode event loop for the toggle viewer.
//!
//! Drives the state machine: Source ↔ Rendered.  The loop handles keypresses
//! and delegates rendering to the caller-supplied callbacks.

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::error::Result;

/// The two views the toggle mode can display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Show the file source with bat syntax highlighting.
    Source,
    /// Show the compiled+rendered view (PDF page as PNG).
    Rendered,
}

/// Run the interactive keypress loop.
///
/// `show_source` and `show_rendered` are called each time the view changes.
///
/// Key bindings:
/// - `s` / `S` → switch to source view
/// - `r` / `R` → switch to rendered view
/// - `q` / `Q` / `Ctrl-C` / `Escape` → quit
///
/// # Errors
///
/// Returns any I/O error from crossterm or the view callbacks.
pub fn run_event_loop<S, R>(
    initial_view: View,
    mut show_source: S,
    mut show_rendered: R,
) -> Result<()>
where
    S: FnMut() -> Result<()>,
    R: FnMut() -> Result<()>,
{
    let mut stdout = std::io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;

    let mut current = initial_view;

    // Show the initial view.
    let show_result = match current {
        View::Source => show_source(),
        View::Rendered => show_rendered(),
    };
    if let Err(e) = show_result {
        // Clean up before propagating.
        let _ = disable_raw_mode();
        let _ = execute!(stdout, LeaveAlternateScreen);
        return Err(e);
    }

    loop {
        let ev = event::read()?;
        let requested = handle_event(&ev);

        match requested {
            EventAction::Quit => break,
            EventAction::Show(next) if next != current => {
                current = next;
                let result = match current {
                    View::Source => show_source(),
                    View::Rendered => show_rendered(),
                };
                if let Err(e) = result {
                    let _ = disable_raw_mode();
                    let _ = execute!(stdout, LeaveAlternateScreen);
                    return Err(e);
                }
            }
            // Same view requested, or unrecognised key — no-op.
            EventAction::Show(_) | EventAction::None => {}
        }
    }

    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;
    Ok(())
}

/// Internal action produced by a single key event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventAction {
    Quit,
    Show(View),
    None,
}

/// Map a crossterm [`Event`] to an [`EventAction`].
///
/// This is a pure function with no side effects, making it straightforward
/// to unit-test the state machine transitions.
pub fn handle_event(ev: &Event) -> EventAction {
    match ev {
        Event::Key(KeyEvent {
            code: KeyCode::Char('q' | 'Q') | KeyCode::Esc,
            ..
        }) => EventAction::Quit,

        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers,
            ..
        }) if modifiers.contains(KeyModifiers::CONTROL) => EventAction::Quit,

        Event::Key(KeyEvent {
            code: KeyCode::Char('s' | 'S'),
            ..
        }) => EventAction::Show(View::Source),

        Event::Key(KeyEvent {
            code: KeyCode::Char('r' | 'R'),
            ..
        }) => EventAction::Show(View::Rendered),

        _ => EventAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    fn ctrl(c: char) -> Event {
        Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    #[test]
    fn q_key_produces_quit() {
        assert_eq!(handle_event(&key(KeyCode::Char('q'))), EventAction::Quit);
    }

    #[test]
    fn uppercase_q_produces_quit() {
        assert_eq!(handle_event(&key(KeyCode::Char('Q'))), EventAction::Quit);
    }

    #[test]
    fn escape_produces_quit() {
        assert_eq!(handle_event(&key(KeyCode::Esc)), EventAction::Quit);
    }

    #[test]
    fn ctrl_c_produces_quit() {
        assert_eq!(handle_event(&ctrl('c')), EventAction::Quit);
    }

    #[test]
    fn s_key_shows_source() {
        assert_eq!(
            handle_event(&key(KeyCode::Char('s'))),
            EventAction::Show(View::Source)
        );
    }

    #[test]
    fn uppercase_s_shows_source() {
        assert_eq!(
            handle_event(&key(KeyCode::Char('S'))),
            EventAction::Show(View::Source)
        );
    }

    #[test]
    fn r_key_shows_rendered() {
        assert_eq!(
            handle_event(&key(KeyCode::Char('r'))),
            EventAction::Show(View::Rendered)
        );
    }

    #[test]
    fn uppercase_r_shows_rendered() {
        assert_eq!(
            handle_event(&key(KeyCode::Char('R'))),
            EventAction::Show(View::Rendered)
        );
    }

    #[test]
    fn unknown_key_produces_none() {
        assert_eq!(handle_event(&key(KeyCode::Char('x'))), EventAction::None);
    }

    #[test]
    fn transition_source_to_rendered() {
        let mut state = View::Source;
        let action = handle_event(&key(KeyCode::Char('r')));
        if let EventAction::Show(next) = action {
            if next != state {
                state = next;
            }
        }
        assert_eq!(state, View::Rendered);
    }

    #[test]
    fn transition_rendered_to_source() {
        let mut state = View::Rendered;
        let action = handle_event(&key(KeyCode::Char('s')));
        if let EventAction::Show(next) = action {
            if next != state {
                state = next;
            }
        }
        assert_eq!(state, View::Source);
    }

    #[test]
    fn same_view_request_no_transition() {
        let state = View::Source;
        let action = handle_event(&key(KeyCode::Char('s')));
        if let EventAction::Show(next) = action {
            // next == state, so no transition needed
            assert_eq!(next, state);
        }
    }
}
