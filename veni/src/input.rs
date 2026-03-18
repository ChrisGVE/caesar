/// All actions that can be triggered by a key sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    MoveDown,
    MoveUp,
    EnterDir,
    ParentDir,
    GoTop,
    GoBottom,
    Quit,
    Yank,
    Delete,
    Paste,
    Undo,
    Rename,
    EnterVisual,
    ToggleVisualLine,
    EnterCommand,
    SearchForward,
    SearchNext,
    SearchPrev,
    ToggleHidden,
    DotRepeat,
    /// Shift-H: scroll viewport left (show previous pane).
    ScrollLeft,
    /// Shift-L: scroll viewport right (show next pane).
    ScrollRight,
}

/// Resolves a raw key character (and a pending first character) into a
/// [`KeyAction`].  Multi-key sequences (`gg`, `dd`, `yy`, `ciw`, `cw`, `gh`)
/// are handled by accumulating the first character in `pending`.
///
/// Returns `None` when a first key has been stored as pending (waiting for a
/// second key) or when the key has no known binding.
///
/// The caller is responsible for implementing any timeout logic; this function
/// only performs synchronous matching.
pub fn resolve(ch: char, pending: &mut Option<char>) -> Option<KeyAction> {
    match *pending {
        Some(first) => {
            *pending = None;
            match (first, ch) {
                ('g', 'g') => Some(KeyAction::GoTop),
                ('g', 'h') => Some(KeyAction::ToggleHidden),
                ('d', 'd') => Some(KeyAction::Delete),
                ('y', 'y') => Some(KeyAction::Yank),
                // `ciw` and `cw` both map to rename in a file manager context.
                ('c', 'i') => {
                    // `ciw` — store 'i' as new pending; caller needs a third
                    // key 'w'.  For simplicity we resolve immediately on 'i'
                    // followed by 'w'.
                    *pending = Some('i');
                    None
                }
                ('i', 'w') => Some(KeyAction::Rename),
                ('c', 'w') => Some(KeyAction::Rename),
                // Unrecognised second key: discard the sequence and try to
                // resolve the new key as a standalone.
                _ => resolve(ch, pending),
            }
        }
        None => match ch {
            // Single-key actions that fire immediately.
            'j' => Some(KeyAction::MoveDown),
            'k' => Some(KeyAction::MoveUp),
            'l' => Some(KeyAction::EnterDir),
            'h' => Some(KeyAction::ParentDir),
            'G' => Some(KeyAction::GoBottom),
            'q' => Some(KeyAction::Quit),
            'p' => Some(KeyAction::Paste),
            'u' => Some(KeyAction::Undo),
            'v' => Some(KeyAction::EnterVisual),
            'V' => Some(KeyAction::ToggleVisualLine),
            ':' => Some(KeyAction::EnterCommand),
            '/' => Some(KeyAction::SearchForward),
            'n' => Some(KeyAction::SearchNext),
            'N' => Some(KeyAction::SearchPrev),
            '.' => Some(KeyAction::DotRepeat),
            'H' => Some(KeyAction::ScrollLeft),
            'L' => Some(KeyAction::ScrollRight),
            // First key of a potential two-key sequence.
            'g' | 'd' | 'y' | 'c' => {
                *pending = Some(ch);
                None
            }
            _ => None,
        },
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn res(ch: char, pending: &mut Option<char>) -> Option<KeyAction> {
        resolve(ch, pending)
    }

    #[test]
    fn single_key_move_down() {
        let mut p = None;
        assert_eq!(res('j', &mut p), Some(KeyAction::MoveDown));
        assert_eq!(p, None);
    }

    #[test]
    fn single_key_move_up() {
        let mut p = None;
        assert_eq!(res('k', &mut p), Some(KeyAction::MoveUp));
    }

    #[test]
    fn single_key_enter_dir() {
        let mut p = None;
        assert_eq!(res('l', &mut p), Some(KeyAction::EnterDir));
    }

    #[test]
    fn single_key_parent_dir() {
        let mut p = None;
        assert_eq!(res('h', &mut p), Some(KeyAction::ParentDir));
    }

    #[test]
    fn single_key_go_bottom() {
        let mut p = None;
        assert_eq!(res('G', &mut p), Some(KeyAction::GoBottom));
    }

    #[test]
    fn single_key_quit() {
        let mut p = None;
        assert_eq!(res('q', &mut p), Some(KeyAction::Quit));
    }

    #[test]
    fn single_key_paste() {
        let mut p = None;
        assert_eq!(res('p', &mut p), Some(KeyAction::Paste));
    }

    #[test]
    fn single_key_undo() {
        let mut p = None;
        assert_eq!(res('u', &mut p), Some(KeyAction::Undo));
    }

    #[test]
    fn single_key_visual() {
        let mut p = None;
        assert_eq!(res('v', &mut p), Some(KeyAction::EnterVisual));
    }

    #[test]
    fn single_key_visual_line() {
        let mut p = None;
        assert_eq!(res('V', &mut p), Some(KeyAction::ToggleVisualLine));
    }

    #[test]
    fn single_key_command() {
        let mut p = None;
        assert_eq!(res(':', &mut p), Some(KeyAction::EnterCommand));
    }

    #[test]
    fn single_key_search_forward() {
        let mut p = None;
        assert_eq!(res('/', &mut p), Some(KeyAction::SearchForward));
    }

    #[test]
    fn single_key_search_next() {
        let mut p = None;
        assert_eq!(res('n', &mut p), Some(KeyAction::SearchNext));
    }

    #[test]
    fn single_key_search_prev() {
        let mut p = None;
        assert_eq!(res('N', &mut p), Some(KeyAction::SearchPrev));
    }

    #[test]
    fn single_key_dot_repeat() {
        let mut p = None;
        assert_eq!(res('.', &mut p), Some(KeyAction::DotRepeat));
    }

    #[test]
    fn single_key_scroll_left() {
        let mut p = None;
        assert_eq!(res('H', &mut p), Some(KeyAction::ScrollLeft));
    }

    #[test]
    fn single_key_scroll_right() {
        let mut p = None;
        assert_eq!(res('L', &mut p), Some(KeyAction::ScrollRight));
    }

    #[test]
    fn gg_goes_top() {
        let mut p = None;
        assert_eq!(res('g', &mut p), None);
        assert_eq!(p, Some('g'));
        assert_eq!(res('g', &mut p), Some(KeyAction::GoTop));
        assert_eq!(p, None);
    }

    #[test]
    fn gh_toggles_hidden() {
        let mut p = None;
        assert_eq!(res('g', &mut p), None);
        assert_eq!(p, Some('g'));
        assert_eq!(res('h', &mut p), Some(KeyAction::ToggleHidden));
        assert_eq!(p, None);
    }

    #[test]
    fn dd_delete() {
        let mut p = None;
        assert_eq!(res('d', &mut p), None);
        assert_eq!(res('d', &mut p), Some(KeyAction::Delete));
    }

    #[test]
    fn yy_yank() {
        let mut p = None;
        assert_eq!(res('y', &mut p), None);
        assert_eq!(res('y', &mut p), Some(KeyAction::Yank));
    }

    #[test]
    fn cw_rename() {
        let mut p = None;
        assert_eq!(res('c', &mut p), None);
        assert_eq!(res('w', &mut p), Some(KeyAction::Rename));
    }

    #[test]
    fn ciw_rename() {
        let mut p = None;
        // 'c' sets pending to 'c', returns None.
        assert_eq!(res('c', &mut p), None);
        assert_eq!(p, Some('c'));
        // 'i' matches (c,i) -> sets pending to 'i', returns None.
        assert_eq!(res('i', &mut p), None);
        assert_eq!(p, Some('i'));
        // 'w' matches (i,w) -> Rename.
        assert_eq!(res('w', &mut p), Some(KeyAction::Rename));
        assert_eq!(p, None);
    }

    #[test]
    fn g_then_non_g_non_h_cancels_and_processes_new_key() {
        let mut p = None;
        // First 'g' sets pending.
        assert_eq!(res('g', &mut p), None);
        // 'j' cancels pending 'g' and resolves 'j' as MoveDown.
        assert_eq!(res('j', &mut p), Some(KeyAction::MoveDown));
        assert_eq!(p, None);
    }

    #[test]
    fn unknown_key_returns_none() {
        let mut p = None;
        assert_eq!(res('z', &mut p), None);
        assert_eq!(p, None);
    }

    #[test]
    fn pending_cleared_after_sequence() {
        let mut p = None;
        res('g', &mut p);
        res('g', &mut p);
        assert_eq!(p, None);
    }
}
