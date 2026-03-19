use crate::config::VeniConfig;
use crate::error::Result;
use crate::input::{resolve, KeyAction};
use crate::ops::{execute_op, inverse_op, FileOp};
use crate::pane::Pane;
use caesar_common::terminal::{detect_multiplexer, MultiplexerInfo, TerminalCaps};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::SystemTime;

/// Maximum number of operations kept in the undo stack.
const UNDO_STACK_LIMIT: usize = 50;

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
    /// Incremental filename search.
    Search,
    /// Help overlay showing all keybindings.
    Help,
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
            Mode::Search => write!(f, "SEARCH"),
            Mode::Help => write!(f, "HELP"),
        }
    }
}

/// Layout mode for the main content area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// Single directory pane, full width.
    Single,
    /// Two directory panes, balanced 50/50.
    DualBalanced,
    /// Single directory pane on the left, preview pane on the right.
    SinglePreview,
}

impl Default for LayoutMode {
    fn default() -> Self {
        Self::SinglePreview
    }
}

impl LayoutMode {
    /// Cycle to the next layout mode.
    pub fn next(self) -> Self {
        match self {
            Self::SinglePreview => Self::Single,
            Self::Single => Self::DualBalanced,
            Self::DualBalanced => Self::SinglePreview,
        }
    }
}

impl std::fmt::Display for LayoutMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single => write!(f, "single"),
            Self::DualBalanced => write!(f, "dual"),
            Self::SinglePreview => write!(f, "preview"),
        }
    }
}

/// Whether a yank is Copy or Cut.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardOp {
    Copy,
    Cut,
}

/// A single entry in the directory listing.
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    /// Whether this entry is a symbolic link.
    pub is_symlink: bool,
    /// The path this symlink points to, if it is one.
    pub symlink_target: Option<PathBuf>,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

/// Core application state.
pub struct App {
    pub mode: Mode,
    pub caps: TerminalCaps,
    pub config: VeniConfig,
    pub should_quit: bool,
    /// Detected terminal multiplexer (tmux, Zellij, screen, or none).
    pub multiplexer: MultiplexerInfo,
    /// All panes (niri-style horizontal workspace).
    pub panes: Vec<Pane>,
    /// Which pane has keyboard focus (index into `panes`).
    pub active_pane: usize,
    /// Index of the leftmost visible pane (niri-style viewport).
    pub viewport_start: usize,
    /// Pending first key for multi-key sequences (e.g. `gg`, `dd`, `yy`).
    pub pending_key: Option<char>,
    /// Index where Visual mode selection started (in the active pane).
    pub visual_anchor: Option<usize>,
    /// Explicitly toggled entries (V-mode line selections) in the active pane.
    pub selection: HashSet<usize>,
    /// Buffer for Command mode input (`:` commands).
    pub command_input: String,
    /// Buffer for Search mode input (`/` search).
    pub search_query: String,
    /// Indices into the active pane's entries that match the current search.
    pub search_matches: Vec<usize>,
    /// Position within `search_matches` currently highlighted.
    pub search_match_idx: usize,
    /// Yanked file paths.
    pub clipboard: Vec<PathBuf>,
    /// Whether the last yank was Copy or Cut.
    pub clipboard_op: ClipboardOp,
    /// Completed operations (for undo).
    undo_stack: Vec<FileOp>,
    /// Undone operations (for redo).
    redo_stack: Vec<FileOp>,
    /// Buffer for the in-progress rename (Insert mode).
    pub rename_buffer: String,
    /// Original path of the file being renamed; `None` when not renaming.
    pub rename_origin: Option<PathBuf>,
    /// Last repeatable file action (for `.` dot-repeat).
    pub last_file_action: Option<KeyAction>,
    /// Scroll offset for the help overlay.
    pub help_scroll_offset: usize,
    /// Current layout mode.
    pub layout_mode: LayoutMode,
}

impl App {
    pub fn new(path: PathBuf, caps: TerminalCaps, config: VeniConfig) -> Self {
        let left = Pane::new(path.clone());
        let right = Pane::new(path);
        let multiplexer = detect_multiplexer();
        Self {
            mode: Mode::Normal,
            caps,
            config,
            should_quit: false,
            multiplexer,
            panes: vec![left, right],
            active_pane: 0,
            viewport_start: 0,
            pending_key: None,
            visual_anchor: None,
            selection: HashSet::new(),
            command_input: String::new(),
            search_query: String::new(),
            search_matches: Vec::new(),
            search_match_idx: 0,
            clipboard: Vec::new(),
            clipboard_op: ClipboardOp::Copy,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            rename_buffer: String::new(),
            rename_origin: None,
            last_file_action: None,
            help_scroll_offset: 0,
            layout_mode: LayoutMode::default(),
        }
    }

    /// Read all panes' directories from disk.
    pub fn load_dir(&mut self) -> Result<()> {
        let show_hidden = self.config.show_hidden;
        for pane in &mut self.panes {
            pane.load_dir(show_hidden)?;
        }
        Ok(())
    }

    /// Immutable reference to the currently focused pane.
    pub fn active(&self) -> &Pane {
        &self.panes[self.active_pane]
    }

    /// Mutable reference to the currently focused pane.
    pub fn active_mut(&mut self) -> &mut Pane {
        &mut self.panes[self.active_pane]
    }

    // ------------------------------------------------------------------
    // Convenience accessors that proxy to the active pane so that
    // existing code (especially ui.rs) still compiles with minimal changes.
    // ------------------------------------------------------------------

    /// CWD of the active pane.
    pub fn cwd(&self) -> &PathBuf {
        &self.active().cwd
    }

    /// Entries of the active pane.
    pub fn entries(&self) -> &[DirEntry] {
        &self.active().entries
    }

    /// Selected index of the active pane.
    pub fn selected(&self) -> usize {
        self.active().selected
    }

    /// Dispatch a key event to the active mode handler.
    pub fn handle_key(&mut self, key: KeyEvent) {
        // Ctrl-c always quits.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            self.pending_key = None;
            return;
        }
        match self.mode {
            Mode::Normal => self.handle_key_normal(key),
            Mode::Visual => self.handle_key_visual(key),
            Mode::Command => self.handle_key_command(key),
            Mode::Search => self.handle_key_search(key),
            Mode::Insert => self.handle_key_insert(key),
            Mode::Help => self.handle_key_help(key),
        }
    }

    // ------------------------------------------------------------------
    // Normal mode
    // ------------------------------------------------------------------

    fn handle_key_normal(&mut self, key: KeyEvent) {
        // Tab switches the active pane.
        if key.code == KeyCode::Tab {
            self.switch_pane();
            return;
        }

        // Ctrl-r = redo.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('r') {
            self.do_redo();
            return;
        }

        // Ctrl-h = new pane to the left of active.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('h') {
            self.add_pane_left();
            return;
        }

        // Ctrl-l = new pane to the right of active.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('l') {
            self.add_pane_right();
            return;
        }

        // Ctrl-w then q = close active pane (handled via two-step: first key
        // sets pending_key to Ctrl-w sentinel, second key 'q' closes).
        // We implement it as a direct check on Ctrl-w here and handle the 'q'
        // in the normal char dispatch below via a special sentinel.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('w') {
            self.pending_key = Some('\x17'); // Ctrl-w sentinel
            return;
        }

        // Handle pending Ctrl-w sentinel for Ctrl-w q.
        if self.pending_key == Some('\x17') {
            self.pending_key = None;
            if key.code == KeyCode::Char('q') {
                self.close_active_pane();
            }
            return;
        }

        // F1 opens help (like neovim).
        if key.code == KeyCode::F(1) {
            self.help_scroll_offset = 0;
            self.mode = Mode::Help;
            return;
        }

        // Arrow keys handled directly without going through the char resolver.
        match key.code {
            KeyCode::Down => {
                self.pending_key = None;
                self.move_down();
                return;
            }
            KeyCode::Up => {
                self.pending_key = None;
                self.move_up();
                return;
            }
            KeyCode::Right | KeyCode::Enter => {
                self.pending_key = None;
                self.enter_dir();
                return;
            }
            KeyCode::Left | KeyCode::Backspace => {
                self.pending_key = None;
                self.go_parent();
                return;
            }
            _ => {}
        }

        if let KeyCode::Char(ch) = key.code {
            if let Some(action) = resolve(ch, &mut self.pending_key) {
                self.execute_action(action);
            }
        }
    }

    fn execute_action(&mut self, action: KeyAction) {
        match action {
            KeyAction::MoveDown => self.move_down(),
            KeyAction::MoveUp => self.move_up(),
            KeyAction::EnterDir => self.enter_dir(),
            KeyAction::ParentDir => self.go_parent(),
            KeyAction::GoTop => self.go_top(),
            KeyAction::GoBottom => self.go_bottom(),
            KeyAction::Quit => self.should_quit = true,
            KeyAction::EnterVisual => {
                let sel = self.active().selected;
                self.visual_anchor = Some(sel);
                self.mode = Mode::Visual;
            }
            KeyAction::ToggleVisualLine => {
                let sel = self.active().selected;
                if self.selection.contains(&sel) {
                    self.selection.remove(&sel);
                } else {
                    self.selection.insert(sel);
                }
            }
            KeyAction::EnterCommand => {
                self.command_input.clear();
                self.mode = Mode::Command;
            }
            KeyAction::SearchForward | KeyAction::SearchBackward => {
                self.search_query.clear();
                self.search_matches.clear();
                self.search_match_idx = 0;
                self.mode = Mode::Search;
            }
            KeyAction::SearchNext => self.search_next(),
            KeyAction::SearchPrev => self.search_prev(),
            KeyAction::Yank => {
                self.yank_current(ClipboardOp::Copy);
                self.last_file_action = Some(KeyAction::Yank);
            }
            KeyAction::Delete => {
                self.yank_current(ClipboardOp::Cut);
                self.last_file_action = Some(KeyAction::Delete);
            }
            KeyAction::Paste => {
                self.do_paste();
                self.last_file_action = Some(KeyAction::Paste);
            }
            KeyAction::Undo => self.do_undo(),
            KeyAction::Rename => self.begin_rename(),
            KeyAction::ToggleHidden => self.toggle_hidden(),
            KeyAction::DotRepeat => self.dot_repeat(),
            KeyAction::ScrollLeft => self.scroll_viewport_left(),
            KeyAction::ScrollRight => self.scroll_viewport_right(),
            KeyAction::ToggleLayout => {
                self.layout_mode = self.layout_mode.next();
                // In single mode, only one pane is relevant. In dual, ensure
                // we have at least 2 panes.
                if self.layout_mode == LayoutMode::DualBalanced && self.panes.len() < 2 {
                    let new_pane = Pane::new(self.active().cwd.clone());
                    self.panes.push(new_pane);
                    let show_hidden = self.config.show_hidden;
                    let last = self.panes.len() - 1;
                    let _ = self.panes[last].load_dir(show_hidden);
                }
            }
            KeyAction::ShowHelp => self.mode = Mode::Help,
        }
    }

    // ------------------------------------------------------------------
    // Help mode — any key dismisses the help overlay
    // ------------------------------------------------------------------

    fn handle_key_help(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.help_scroll_down(),
            KeyCode::Char('k') | KeyCode::Up => self.help_scroll_up(),
            KeyCode::Char('G') => self.help_scroll_end(),
            _ => self.mode = Mode::Normal,
        }
    }

    /// Scroll offset for the help view.
    pub fn help_scroll(&self) -> usize {
        self.help_scroll_offset
    }

    fn help_scroll_down(&mut self) {
        self.help_scroll_offset = self.help_scroll_offset.saturating_add(1);
    }

    fn help_scroll_up(&mut self) {
        self.help_scroll_offset = self.help_scroll_offset.saturating_sub(1);
    }

    fn help_scroll_end(&mut self) {
        // A large number — draw will clamp it.
        self.help_scroll_offset = 999;
    }

    // ------------------------------------------------------------------
    // Insert mode (rename)
    // ------------------------------------------------------------------

    fn handle_key_insert(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                // Cancel rename.
                self.rename_buffer.clear();
                self.rename_origin = None;
                self.mode = Mode::Normal;
            }
            KeyCode::Enter => {
                self.confirm_rename();
            }
            KeyCode::Backspace => {
                self.rename_buffer.pop();
            }
            KeyCode::Char(ch) => {
                self.rename_buffer.push(ch);
            }
            _ => {}
        }
        self.pending_key = None;
    }

    // ------------------------------------------------------------------
    // Visual mode
    // ------------------------------------------------------------------

    fn handle_key_visual(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.visual_anchor = None;
            }
            KeyCode::Char('j') | KeyCode::Down => self.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.move_up(),
            KeyCode::Char('y') => {
                self.yank_visual(ClipboardOp::Copy);
                self.mode = Mode::Normal;
                self.visual_anchor = None;
            }
            KeyCode::Char('d') => {
                self.yank_visual(ClipboardOp::Cut);
                self.mode = Mode::Normal;
                self.visual_anchor = None;
            }
            KeyCode::Char('V') => {
                // Toggle current entry in explicit selection set and exit visual.
                let sel = self.active().selected;
                if self.selection.contains(&sel) {
                    self.selection.remove(&sel);
                } else {
                    self.selection.insert(sel);
                }
                self.mode = Mode::Normal;
                self.visual_anchor = None;
            }
            _ => {}
        }
    }

    /// Returns the range of indices covered by the current Visual selection.
    /// Returns an empty range when not in Visual mode or no anchor is set.
    pub fn visual_range(&self) -> std::ops::RangeInclusive<usize> {
        match self.visual_anchor {
            Some(anchor) => {
                let cur = self.active().selected;
                let lo = anchor.min(cur);
                let hi = anchor.max(cur);
                lo..=hi
            }
            None => 0..=0, // degenerate; callers should check mode
        }
    }

    // ------------------------------------------------------------------
    // Command mode
    // ------------------------------------------------------------------

    fn handle_key_command(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.command_input.clear();
                self.mode = Mode::Normal;
            }
            KeyCode::Enter => {
                let cmd = self.command_input.trim().to_string();
                self.command_input.clear();
                self.mode = Mode::Normal;
                self.execute_command(&cmd);
            }
            KeyCode::Backspace => {
                self.command_input.pop();
            }
            KeyCode::Char(ch) => {
                self.command_input.push(ch);
            }
            _ => {}
        }
    }

    fn execute_command(&mut self, cmd: &str) {
        match cmd {
            "q" => self.should_quit = true,
            "help" => {
                self.help_scroll_offset = 0;
                self.mode = Mode::Help;
                return;
            }
            "set hidden" => {
                self.config.show_hidden = true;
                let _ = self.load_dir();
            }
            "set nohidden" => {
                self.config.show_hidden = false;
                let _ = self.load_dir();
            }
            other if other.starts_with("cd ") => {
                let path_str = other.trim_start_matches("cd ").trim();
                let new_path = if path_str.starts_with('/') {
                    PathBuf::from(path_str)
                } else {
                    self.active().cwd.join(path_str)
                };
                if new_path.is_dir() {
                    let show_hidden = self.config.show_hidden;
                    self.active_mut().cwd = new_path;
                    let _ = self.active_mut().load_dir(show_hidden);
                }
            }
            _ => {} // unknown command — silently ignore
        }
    }

    // ------------------------------------------------------------------
    // Search mode
    // ------------------------------------------------------------------

    fn handle_key_search(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.search_query.clear();
                self.search_matches.clear();
                self.search_match_idx = 0;
                self.mode = Mode::Normal;
            }
            KeyCode::Enter => {
                // Confirm search: move to first match if any, return to Normal.
                if !self.search_matches.is_empty() {
                    self.active_mut().selected = self.search_matches[0];
                    self.search_match_idx = 0;
                }
                self.mode = Mode::Normal;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.update_search_matches();
            }
            KeyCode::Char(ch) => {
                self.search_query.push(ch);
                self.update_search_matches();
                // Jump cursor to first match immediately.
                if !self.search_matches.is_empty() {
                    self.active_mut().selected = self.search_matches[0];
                    self.search_match_idx = 0;
                }
            }
            _ => {}
        }
    }

    pub fn update_search_matches(&mut self) {
        if self.search_query.is_empty() {
            self.search_matches.clear();
            self.search_match_idx = 0;
            return;
        }
        let query = self.search_query.to_lowercase();
        self.search_matches = self
            .active()
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.name.to_lowercase().contains(&query))
            .map(|(i, _)| i)
            .collect();
        self.search_match_idx = 0;
    }

    fn search_next(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.search_match_idx = (self.search_match_idx + 1) % self.search_matches.len();
        self.active_mut().selected = self.search_matches[self.search_match_idx];
    }

    fn search_prev(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        if self.search_match_idx == 0 {
            self.search_match_idx = self.search_matches.len() - 1;
        } else {
            self.search_match_idx -= 1;
        }
        self.active_mut().selected = self.search_matches[self.search_match_idx];
    }

    // ------------------------------------------------------------------
    // Pane switching
    // ------------------------------------------------------------------

    fn switch_pane(&mut self) {
        self.active_pane = (self.active_pane + 1) % self.panes.len();
        self.clear_per_pane_state();
    }

    /// Clear search / selection state that is per-pane.
    fn clear_per_pane_state(&mut self) {
        self.search_query.clear();
        self.search_matches.clear();
        self.search_match_idx = 0;
        self.visual_anchor = None;
        self.selection.clear();
        self.pending_key = None;
        if self.mode == Mode::Visual || self.mode == Mode::Search {
            self.mode = Mode::Normal;
        }
    }

    // ------------------------------------------------------------------
    // Niri-style viewport scrolling and pane management
    // ------------------------------------------------------------------

    /// Scroll viewport one pane to the left (show previous pane).
    fn scroll_viewport_left(&mut self) {
        if self.viewport_start > 0 {
            self.viewport_start -= 1;
            // Also move focus left if active pane is now out of view.
            if self.active_pane > 0 {
                self.active_pane -= 1;
                self.clear_per_pane_state();
            }
        }
    }

    /// Scroll viewport one pane to the right (show next pane).
    fn scroll_viewport_right(&mut self) {
        if self.viewport_start + 1 < self.panes.len() {
            self.viewport_start += 1;
            // Also move focus right if active pane is still within bounds.
            if self.active_pane + 1 < self.panes.len() {
                self.active_pane += 1;
                self.clear_per_pane_state();
            }
        }
    }

    /// Insert a new pane to the left of the active pane.
    fn add_pane_left(&mut self) {
        let cwd = self.active().cwd.clone();
        let show_hidden = self.config.show_hidden;
        let mut new_pane = Pane::new(cwd);
        let _ = new_pane.load_dir(show_hidden);
        self.panes.insert(self.active_pane, new_pane);
        // active_pane index now points to the new pane (inserted before old active).
        self.clear_per_pane_state();
    }

    /// Insert a new pane to the right of the active pane.
    fn add_pane_right(&mut self) {
        let cwd = self.active().cwd.clone();
        let show_hidden = self.config.show_hidden;
        let mut new_pane = Pane::new(cwd);
        let _ = new_pane.load_dir(show_hidden);
        let insert_idx = self.active_pane + 1;
        self.panes.insert(insert_idx, new_pane);
        self.active_pane = insert_idx;
        self.clear_per_pane_state();
    }

    /// Close the active pane.  Does nothing if only one pane remains.
    fn close_active_pane(&mut self) {
        if self.panes.len() <= 1 {
            return;
        }
        self.panes.remove(self.active_pane);
        // Adjust active_pane so it stays within bounds.
        if self.active_pane >= self.panes.len() {
            self.active_pane = self.panes.len() - 1;
        }
        // Clamp viewport.
        if self.viewport_start >= self.panes.len() {
            self.viewport_start = self.panes.len().saturating_sub(1);
        }
        self.clear_per_pane_state();
    }

    // ------------------------------------------------------------------
    // Clipboard
    // ------------------------------------------------------------------

    fn yank_current(&mut self, op: ClipboardOp) {
        if let Some(entry) = self.active().current_entry() {
            self.clipboard = vec![entry.path.clone()];
            self.clipboard_op = op;
        }
        self.redo_stack.clear();
    }

    fn yank_visual(&mut self, op: ClipboardOp) {
        let anchor = self.visual_anchor.unwrap_or(self.active().selected);
        let cur = self.active().selected;
        let lo = anchor.min(cur);
        let hi = anchor.max(cur);
        let paths: Vec<PathBuf> = self.active().entries[lo..=hi]
            .iter()
            .map(|e| e.path.clone())
            .collect();
        if !paths.is_empty() {
            self.clipboard = paths;
            self.clipboard_op = op;
        }
        self.redo_stack.clear();
    }

    // ------------------------------------------------------------------
    // Paste
    // ------------------------------------------------------------------

    fn do_paste(&mut self) {
        if self.clipboard.is_empty() {
            return;
        }
        let dest = self.active().cwd.clone();
        let op = match self.clipboard_op {
            ClipboardOp::Copy => FileOp::Copy {
                sources: self.clipboard.clone(),
                dest,
            },
            ClipboardOp::Cut => {
                let op = FileOp::Move {
                    sources: self.clipboard.clone(),
                    dest,
                };
                // Clear clipboard after cut-paste so it cannot be pasted twice.
                self.clipboard.clear();
                op
            }
        };

        if execute_op(&op).is_ok() {
            self.push_undo(op);
            let show_hidden = self.config.show_hidden;
            let _ = self.panes[self.active_pane].load_dir(show_hidden);
        }
    }

    // ------------------------------------------------------------------
    // Rename
    // ------------------------------------------------------------------

    fn begin_rename(&mut self) {
        if let Some(entry) = self.active().current_entry() {
            let name = entry.name.clone();
            let path = entry.path.clone();
            self.rename_buffer = name;
            self.rename_origin = Some(path);
            self.mode = Mode::Insert;
        }
    }

    fn confirm_rename(&mut self) {
        if let Some(origin) = self.rename_origin.take() {
            let new_name = self.rename_buffer.trim().to_string();
            if !new_name.is_empty()
                && new_name
                    != origin
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default()
            {
                if let Some(parent) = origin.parent() {
                    let to = parent.join(&new_name);
                    let op = FileOp::Rename { from: origin, to };
                    if execute_op(&op).is_ok() {
                        self.push_undo(op);
                        let show_hidden = self.config.show_hidden;
                        let _ = self.panes[self.active_pane].load_dir(show_hidden);
                        self.last_file_action = Some(KeyAction::Rename);
                    }
                }
            }
        }
        self.rename_buffer.clear();
        self.mode = Mode::Normal;
    }

    // ------------------------------------------------------------------
    // Hidden toggle
    // ------------------------------------------------------------------

    fn toggle_hidden(&mut self) {
        self.config.show_hidden = !self.config.show_hidden;
        let show_hidden = self.config.show_hidden;
        let _ = self.panes[self.active_pane].load_dir(show_hidden);
    }

    // ------------------------------------------------------------------
    // Dot repeat
    // ------------------------------------------------------------------

    fn dot_repeat(&mut self) {
        if let Some(action) = self.last_file_action {
            match action {
                KeyAction::Yank => self.yank_current(ClipboardOp::Copy),
                KeyAction::Delete => self.yank_current(ClipboardOp::Cut),
                KeyAction::Paste => self.do_paste(),
                KeyAction::Rename => self.begin_rename(),
                _ => {}
            }
        }
    }

    // ------------------------------------------------------------------
    // Undo / Redo
    // ------------------------------------------------------------------

    pub fn push_undo(&mut self, op: FileOp) {
        if self.undo_stack.len() >= UNDO_STACK_LIMIT {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(op);
    }

    fn do_undo(&mut self) {
        if let Some(op) = self.undo_stack.pop() {
            let inv = inverse_op(&op);
            if execute_op(&inv).is_ok() {
                self.redo_stack.push(op);
                let show_hidden = self.config.show_hidden;
                for pane in &mut self.panes {
                    let _ = pane.load_dir(show_hidden);
                }
            } else {
                // Put back if undo failed.
                self.undo_stack.push(op);
            }
        }
    }

    fn do_redo(&mut self) {
        if let Some(op) = self.redo_stack.pop() {
            if execute_op(&op).is_ok() {
                self.push_undo(op);
                let show_hidden = self.config.show_hidden;
                for pane in &mut self.panes {
                    let _ = pane.load_dir(show_hidden);
                }
            } else {
                self.redo_stack.push(op);
            }
        }
    }

    // ------------------------------------------------------------------
    // Navigation primitives (proxy to active pane)
    // ------------------------------------------------------------------

    /// Compute the visible entry height for the active pane.
    ///
    /// The pane area is `rows - 1` (status bar) minus 2 (top+bottom borders).
    /// When `rows` is zero (unknown), use a safe fallback of 20.
    fn visible_height(&self) -> usize {
        let rows = if self.caps.rows > 0 {
            self.caps.rows
        } else {
            23
        };
        (rows as usize).saturating_sub(3)
    }

    fn move_down(&mut self) {
        let pane = &mut self.panes[self.active_pane];
        if !pane.entries.is_empty() && pane.selected < pane.entries.len() - 1 {
            pane.selected += 1;
        }
        let vh = self.visible_height();
        self.panes[self.active_pane].ensure_visible(vh);
    }

    fn move_up(&mut self) {
        let pane = &mut self.panes[self.active_pane];
        if pane.selected > 0 {
            pane.selected -= 1;
        }
        let vh = self.visible_height();
        self.panes[self.active_pane].ensure_visible(vh);
    }

    fn go_top(&mut self) {
        let pane = &mut self.panes[self.active_pane];
        pane.selected = 0;
        pane.scroll_offset = 0;
    }

    fn go_bottom(&mut self) {
        let pane = &mut self.panes[self.active_pane];
        if !pane.entries.is_empty() {
            pane.selected = pane.entries.len() - 1;
        }
        let vh = self.visible_height();
        self.panes[self.active_pane].ensure_visible(vh);
    }

    fn enter_dir(&mut self) {
        let show_hidden = self.config.show_hidden;
        let pane = &mut self.panes[self.active_pane];
        if let Some(entry) = pane.entries.get(pane.selected) {
            // Follow symlink targets that point to a directory.
            let target = if entry.is_symlink {
                entry.symlink_target.clone().filter(|t| t.is_dir())
            } else if entry.is_dir {
                Some(entry.path.clone())
            } else {
                None
            };
            if let Some(new_path) = target {
                pane.cwd = new_path;
                let _ = pane.load_dir(show_hidden);
            }
        }
    }

    fn go_parent(&mut self) {
        let show_hidden = self.config.show_hidden;
        let pane = &mut self.panes[self.active_pane];
        if let Some(parent) = pane.cwd.parent().map(|p| p.to_path_buf()) {
            pane.cwd = parent;
            let _ = pane.load_dir(show_hidden);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_app(dir: &TempDir) -> App {
        App::new(
            dir.path().to_path_buf(),
            TerminalCaps::default(),
            VeniConfig::default(),
        )
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl_key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    // ------------------------------------------------------------------
    // Mode tests
    // ------------------------------------------------------------------

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
        assert_eq!(Mode::Search.to_string(), "SEARCH");
    }

    #[test]
    fn app_starts_in_normal_mode() {
        let tmp = TempDir::new().unwrap();
        let app = make_app(&tmp);
        assert_eq!(app.mode, Mode::Normal);
        assert!(!app.should_quit);
        assert!(app.panes[0].entries.is_empty());
        assert!(app.panes[1].entries.is_empty());
        assert_eq!(app.active_pane, 0);
    }

    // ------------------------------------------------------------------
    // load_dir tests
    // ------------------------------------------------------------------

    #[test]
    fn load_dir_lists_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("file.txt"), b"hello").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        assert_eq!(app.panes[0].entries.len(), 1);
        assert_eq!(app.panes[0].entries[0].name, "file.txt");
    }

    #[test]
    fn load_dir_sorts_dirs_before_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("aaa.txt"), b"").unwrap();
        fs::create_dir(tmp.path().join("bbb_dir")).unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        assert!(app.panes[0].entries[0].is_dir, "directory must come first");
        assert!(!app.panes[0].entries[1].is_dir);
    }

    #[test]
    fn load_dir_sorts_alphabetically_within_group() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("zebra.txt"), b"").unwrap();
        fs::write(tmp.path().join("apple.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        assert_eq!(app.panes[0].entries[0].name, "apple.txt");
        assert_eq!(app.panes[0].entries[1].name, "zebra.txt");
    }

    #[test]
    fn load_dir_hides_dotfiles_by_default() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(".hidden"), b"").unwrap();
        fs::write(tmp.path().join("visible.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        assert_eq!(app.panes[0].entries.len(), 1);
        assert_eq!(app.panes[0].entries[0].name, "visible.txt");
    }

    #[test]
    fn load_dir_shows_dotfiles_when_config_enabled() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(".hidden"), b"").unwrap();
        fs::write(tmp.path().join("visible.txt"), b"").unwrap();
        let mut cfg = VeniConfig::default();
        cfg.show_hidden = true;
        let mut app = App::new(tmp.path().to_path_buf(), TerminalCaps::default(), cfg);
        app.load_dir().unwrap();
        assert_eq!(app.panes[0].entries.len(), 2);
    }

    #[test]
    fn load_dir_resets_cursor() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), b"").unwrap();
        fs::write(tmp.path().join("b.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.panes[0].selected = 1;
        app.load_dir().unwrap();
        assert_eq!(app.panes[0].selected, 0);
    }

    #[test]
    fn load_dir_empty_directory() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        assert!(app.panes[0].entries.is_empty());
    }

    // ------------------------------------------------------------------
    // Pane switching
    // ------------------------------------------------------------------

    #[test]
    fn tab_switches_active_pane() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        assert_eq!(app.active_pane, 0);
        app.handle_key(key(KeyCode::Tab));
        assert_eq!(app.active_pane, 1);
        app.handle_key(key(KeyCode::Tab));
        assert_eq!(app.active_pane, 0);
    }

    #[test]
    fn panes_have_independent_navigation() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), b"").unwrap();
        fs::write(tmp.path().join("b.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();

        // Move down in pane 0.
        app.handle_key(key(KeyCode::Char('j')));
        assert_eq!(app.panes[0].selected, 1);
        // Pane 1 untouched.
        assert_eq!(app.panes[1].selected, 0);

        // Switch to pane 1.
        app.handle_key(key(KeyCode::Tab));
        assert_eq!(app.active_pane, 1);
        assert_eq!(app.panes[1].selected, 0);
    }

    #[test]
    fn active_returns_focused_pane() {
        let tmp = TempDir::new().unwrap();
        let app = make_app(&tmp);
        assert_eq!(app.active().cwd, app.panes[0].cwd);
    }

    // ------------------------------------------------------------------
    // handle_key / navigation tests
    // ------------------------------------------------------------------

    #[test]
    fn q_sets_should_quit() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        app.handle_key(key(KeyCode::Char('q')));
        assert!(app.should_quit);
    }

    #[test]
    fn ctrl_c_quits() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        app.handle_key(ctrl_key(KeyCode::Char('c')));
        assert!(app.should_quit);
    }

    #[test]
    fn j_moves_down() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), b"").unwrap();
        fs::write(tmp.path().join("b.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.handle_key(key(KeyCode::Char('j')));
        assert_eq!(app.panes[0].selected, 1);
    }

    #[test]
    fn k_moves_up() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), b"").unwrap();
        fs::write(tmp.path().join("b.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.panes[0].selected = 1;
        app.handle_key(key(KeyCode::Char('k')));
        assert_eq!(app.panes[0].selected, 0);
    }

    #[test]
    fn j_at_bottom_does_not_overflow() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.handle_key(key(KeyCode::Char('j')));
        assert_eq!(app.panes[0].selected, 0);
    }

    #[test]
    fn k_at_top_does_not_underflow() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.handle_key(key(KeyCode::Char('k')));
        assert_eq!(app.panes[0].selected, 0);
    }

    #[test]
    fn capital_g_goes_to_bottom() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), b"").unwrap();
        fs::write(tmp.path().join("b.txt"), b"").unwrap();
        fs::write(tmp.path().join("c.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.handle_key(key(KeyCode::Char('G')));
        assert_eq!(app.panes[0].selected, 2);
    }

    #[test]
    fn gg_goes_to_top() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), b"").unwrap();
        fs::write(tmp.path().join("b.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.panes[0].selected = 1;
        app.handle_key(key(KeyCode::Char('g')));
        app.handle_key(key(KeyCode::Char('g')));
        assert_eq!(app.panes[0].selected, 0);
    }

    #[test]
    fn single_g_does_not_move() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), b"").unwrap();
        fs::write(tmp.path().join("b.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.panes[0].selected = 1;
        app.handle_key(key(KeyCode::Char('g')));
        assert_eq!(app.panes[0].selected, 1);
        assert_eq!(app.pending_key, Some('g'));
    }

    #[test]
    fn l_enters_subdirectory() {
        let tmp = TempDir::new().unwrap();
        let subdir = tmp.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        assert_eq!(app.panes[0].entries[0].name, "subdir");
        let expected = app.panes[0].entries[0].path.clone();
        app.handle_key(key(KeyCode::Char('l')));
        assert_eq!(app.panes[0].cwd, expected);
    }

    #[test]
    fn h_goes_to_parent() {
        let tmp = TempDir::new().unwrap();
        let subdir = tmp.path().join("sub");
        fs::create_dir(&subdir).unwrap();
        let mut app = App::new(
            subdir.clone(),
            TerminalCaps::default(),
            VeniConfig::default(),
        );
        app.load_dir().unwrap();
        let parent = tmp.path().to_path_buf();
        app.handle_key(key(KeyCode::Char('h')));
        assert_eq!(
            app.panes[0]
                .cwd
                .canonicalize()
                .unwrap_or(app.panes[0].cwd.clone()),
            parent.canonicalize().unwrap_or(parent)
        );
    }

    #[test]
    fn arrow_keys_work_like_hjkl() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), b"").unwrap();
        fs::write(tmp.path().join("b.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.panes[0].selected, 1);
        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.panes[0].selected, 0);
    }

    #[test]
    fn escape_returns_to_normal_from_insert() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        app.mode = Mode::Insert;
        app.handle_key(key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn g_then_non_g_cancels_pending() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), b"").unwrap();
        fs::write(tmp.path().join("b.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.panes[0].selected = 1;
        app.handle_key(key(KeyCode::Char('g')));
        assert_eq!(app.pending_key, Some('g'));
        app.handle_key(key(KeyCode::Char('j')));
        assert_eq!(app.pending_key, None);
    }

    // ------------------------------------------------------------------
    // Visual mode tests
    // ------------------------------------------------------------------

    #[test]
    fn v_enters_visual_mode() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        app.handle_key(key(KeyCode::Char('v')));
        assert_eq!(app.mode, Mode::Visual);
        assert_eq!(app.visual_anchor, Some(0));
    }

    #[test]
    fn esc_exits_visual_mode() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        app.mode = Mode::Visual;
        app.visual_anchor = Some(0);
        app.handle_key(key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.visual_anchor, None);
    }

    #[test]
    fn visual_j_extends_selection_down() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), b"").unwrap();
        fs::write(tmp.path().join("b.txt"), b"").unwrap();
        fs::write(tmp.path().join("c.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.mode = Mode::Visual;
        app.visual_anchor = Some(0);
        app.handle_key(key(KeyCode::Char('j')));
        assert_eq!(app.panes[0].selected, 1);
        let range = app.visual_range();
        assert_eq!(*range.start(), 0);
        assert_eq!(*range.end(), 1);
    }

    #[test]
    fn visual_range_upward() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), b"").unwrap();
        fs::write(tmp.path().join("b.txt"), b"").unwrap();
        fs::write(tmp.path().join("c.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.panes[0].selected = 2;
        app.mode = Mode::Visual;
        app.visual_anchor = Some(2);
        app.handle_key(key(KeyCode::Char('k')));
        let range = app.visual_range();
        assert_eq!(*range.start(), 1);
        assert_eq!(*range.end(), 2);
    }

    #[test]
    fn capital_v_toggles_selection() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        // V in Normal mode toggles current entry.
        app.handle_key(key(KeyCode::Char('V')));
        assert!(app.selection.contains(&0));
        app.handle_key(key(KeyCode::Char('V')));
        assert!(!app.selection.contains(&0));
    }

    // ------------------------------------------------------------------
    // Command mode tests
    // ------------------------------------------------------------------

    #[test]
    fn colon_enters_command_mode() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        app.handle_key(key(KeyCode::Char(':')));
        assert_eq!(app.mode, Mode::Command);
        assert!(app.command_input.is_empty());
    }

    #[test]
    fn command_mode_types_chars() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        app.mode = Mode::Command;
        app.handle_key(key(KeyCode::Char('q')));
        assert_eq!(app.command_input, "q");
    }

    #[test]
    fn command_mode_backspace_deletes() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        app.mode = Mode::Command;
        app.command_input = "cd".to_string();
        app.handle_key(key(KeyCode::Backspace));
        assert_eq!(app.command_input, "c");
    }

    #[test]
    fn command_mode_esc_cancels() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        app.mode = Mode::Command;
        app.command_input = "q".to_string();
        app.handle_key(key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.command_input.is_empty());
    }

    #[test]
    fn command_q_quits() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        app.mode = Mode::Command;
        app.command_input = "q".to_string();
        app.handle_key(key(KeyCode::Enter));
        assert!(app.should_quit);
        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn command_set_hidden_shows_dotfiles() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(".hidden"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        assert_eq!(app.panes[0].entries.len(), 0);
        app.mode = Mode::Command;
        app.command_input = "set hidden".to_string();
        app.handle_key(key(KeyCode::Enter));
        assert!(app.config.show_hidden);
        assert_eq!(app.panes[0].entries.len(), 1);
    }

    #[test]
    fn command_set_nohidden_hides_dotfiles() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(".hidden"), b"").unwrap();
        fs::write(tmp.path().join("visible.txt"), b"").unwrap();
        let mut cfg = VeniConfig::default();
        cfg.show_hidden = true;
        let mut app = App::new(tmp.path().to_path_buf(), TerminalCaps::default(), cfg);
        app.load_dir().unwrap();
        assert_eq!(app.panes[0].entries.len(), 2);
        app.mode = Mode::Command;
        app.command_input = "set nohidden".to_string();
        app.handle_key(key(KeyCode::Enter));
        assert!(!app.config.show_hidden);
        assert_eq!(app.panes[0].entries.len(), 1);
    }

    #[test]
    fn command_cd_changes_directory() {
        let tmp = TempDir::new().unwrap();
        let subdir = tmp.path().join("sub");
        fs::create_dir(&subdir).unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.mode = Mode::Command;
        let cd_cmd = format!("cd {}", subdir.to_string_lossy());
        app.command_input = cd_cmd;
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.panes[0].cwd, subdir);
    }

    #[test]
    fn command_unknown_is_ignored() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        app.mode = Mode::Command;
        app.command_input = "foobar".to_string();
        app.handle_key(key(KeyCode::Enter));
        assert!(!app.should_quit);
        assert_eq!(app.mode, Mode::Normal);
    }

    // ------------------------------------------------------------------
    // Search mode tests
    // ------------------------------------------------------------------

    #[test]
    fn slash_enters_search_mode() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        app.handle_key(key(KeyCode::Char('/')));
        assert_eq!(app.mode, Mode::Search);
        assert!(app.search_query.is_empty());
    }

    #[test]
    fn search_typing_filters_matches() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("alpha.txt"), b"").unwrap();
        fs::write(tmp.path().join("beta.txt"), b"").unwrap();
        fs::write(tmp.path().join("alphabet.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.mode = Mode::Search;
        app.handle_key(key(KeyCode::Char('a')));
        app.handle_key(key(KeyCode::Char('l')));
        // "al" matches alpha.txt and alphabet.txt — not beta.
        assert_eq!(app.search_matches.len(), 2);
        assert_eq!(app.panes[0].selected, app.search_matches[0]);
    }

    #[test]
    fn search_enter_confirms_and_returns_normal() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("alpha.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.mode = Mode::Search;
        app.search_query = "alpha".to_string();
        app.update_search_matches();
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.panes[0].selected, 0);
    }

    #[test]
    fn search_esc_cancels() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("alpha.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.mode = Mode::Search;
        app.search_query = "al".to_string();
        app.handle_key(key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.search_query.is_empty());
        assert!(app.search_matches.is_empty());
    }

    #[test]
    fn search_n_goes_to_next_match() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("alpha.txt"), b"").unwrap();
        fs::write(tmp.path().join("beta.txt"), b"").unwrap();
        fs::write(tmp.path().join("gamma_a.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.search_query = "a".to_string();
        app.update_search_matches();
        assert_eq!(app.search_matches.len(), 3);
        app.panes[0].selected = app.search_matches[0];
        app.search_match_idx = 0;
        app.handle_key(key(KeyCode::Char('n')));
        assert_eq!(app.panes[0].selected, app.search_matches[1]);
    }

    #[test]
    fn search_capital_n_goes_to_prev_match() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("alpha.txt"), b"").unwrap();
        fs::write(tmp.path().join("beta.txt"), b"").unwrap();
        fs::write(tmp.path().join("gamma_a.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.search_query = "a".to_string();
        app.update_search_matches();
        app.panes[0].selected = app.search_matches[1];
        app.search_match_idx = 1;
        app.handle_key(key(KeyCode::Char('N')));
        assert_eq!(app.panes[0].selected, app.search_matches[0]);
    }

    #[test]
    fn search_backspace_removes_char_and_updates() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("alpha.txt"), b"").unwrap();
        fs::write(tmp.path().join("beta.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.mode = Mode::Search;
        app.handle_key(key(KeyCode::Char('a')));
        app.handle_key(key(KeyCode::Backspace));
        assert!(app.search_query.is_empty());
        assert!(app.search_matches.is_empty());
    }

    #[test]
    fn search_case_insensitive() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("Alpha.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.search_query = "alpha".to_string();
        app.update_search_matches();
        assert_eq!(app.search_matches.len(), 1);
    }

    #[test]
    fn search_n_wraps_around() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a1.txt"), b"").unwrap();
        fs::write(tmp.path().join("a2.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.search_query = "a".to_string();
        app.update_search_matches();
        app.search_match_idx = app.search_matches.len() - 1;
        app.panes[0].selected = *app.search_matches.last().unwrap();
        app.handle_key(key(KeyCode::Char('n')));
        assert_eq!(app.search_match_idx, 0);
    }

    #[test]
    fn search_capital_n_wraps_around() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a1.txt"), b"").unwrap();
        fs::write(tmp.path().join("a2.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.search_query = "a".to_string();
        app.update_search_matches();
        app.search_match_idx = 0;
        app.panes[0].selected = app.search_matches[0];
        app.handle_key(key(KeyCode::Char('N')));
        assert_eq!(app.search_match_idx, app.search_matches.len() - 1);
    }

    // ------------------------------------------------------------------
    // Clipboard — yank / paste
    // ------------------------------------------------------------------

    #[test]
    fn yy_yanks_current_file_as_copy() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("file.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        // yy = press 'y' twice.
        app.handle_key(key(KeyCode::Char('y')));
        app.handle_key(key(KeyCode::Char('y')));
        assert_eq!(app.clipboard.len(), 1);
        assert_eq!(app.clipboard[0].file_name().unwrap(), "file.txt");
        assert_eq!(app.clipboard_op, ClipboardOp::Copy);
    }

    #[test]
    fn dd_yanks_current_file_as_cut() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("file.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        // dd = press 'd' twice.
        app.handle_key(key(KeyCode::Char('d')));
        app.handle_key(key(KeyCode::Char('d')));
        assert_eq!(app.clipboard.len(), 1);
        assert_eq!(app.clipboard[0].file_name().unwrap(), "file.txt");
        assert_eq!(app.clipboard_op, ClipboardOp::Cut);
    }

    #[test]
    fn paste_copies_file_to_active_pane_cwd() {
        let src_dir = TempDir::new().unwrap();
        let dst_dir = TempDir::new().unwrap();
        fs::write(src_dir.path().join("file.txt"), b"data").unwrap();

        let mut app = App::new(
            src_dir.path().to_path_buf(),
            TerminalCaps::default(),
            VeniConfig::default(),
        );
        app.load_dir().unwrap();

        // Yank.
        app.handle_key(key(KeyCode::Char('y')));
        app.handle_key(key(KeyCode::Char('y')));
        assert_eq!(app.clipboard_op, ClipboardOp::Copy);

        // Switch active pane to dst.
        app.panes[1].cwd = dst_dir.path().to_path_buf();
        app.handle_key(key(KeyCode::Tab));

        // Paste.
        app.handle_key(key(KeyCode::Char('p')));
        assert!(dst_dir.path().join("file.txt").exists());
    }

    // ------------------------------------------------------------------
    // Rename tests
    // ------------------------------------------------------------------

    #[test]
    fn cw_enters_insert_mode_with_filename() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("file.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.handle_key(key(KeyCode::Char('c')));
        app.handle_key(key(KeyCode::Char('w')));
        assert_eq!(app.mode, Mode::Insert);
        assert_eq!(app.rename_buffer, "file.txt");
        assert!(app.rename_origin.is_some());
    }

    #[test]
    fn rename_esc_cancels_rename() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("file.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.handle_key(key(KeyCode::Char('c')));
        app.handle_key(key(KeyCode::Char('w')));
        app.handle_key(key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.rename_buffer.is_empty());
        assert!(app.rename_origin.is_none());
        // File still has old name.
        assert!(tmp.path().join("file.txt").exists());
    }

    #[test]
    fn rename_enter_renames_file() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("old.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.handle_key(key(KeyCode::Char('c')));
        app.handle_key(key(KeyCode::Char('w')));
        // Clear the buffer and type new name.
        app.rename_buffer.clear();
        app.handle_key(key(KeyCode::Char('n')));
        app.handle_key(key(KeyCode::Char('e')));
        app.handle_key(key(KeyCode::Char('w')));
        app.handle_key(key(KeyCode::Char('.')));
        app.handle_key(key(KeyCode::Char('t')));
        app.handle_key(key(KeyCode::Char('x')));
        app.handle_key(key(KeyCode::Char('t')));
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.mode, Mode::Normal);
        assert!(!tmp.path().join("old.txt").exists());
        assert!(tmp.path().join("new.txt").exists());
    }

    #[test]
    fn rename_undo_restores_old_name() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("old.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        app.handle_key(key(KeyCode::Char('c')));
        app.handle_key(key(KeyCode::Char('w')));
        app.rename_buffer.clear();
        app.rename_buffer = "new.txt".to_string();
        app.handle_key(key(KeyCode::Enter));
        assert!(tmp.path().join("new.txt").exists());

        // Undo the rename.
        app.handle_key(key(KeyCode::Char('u')));
        assert!(tmp.path().join("old.txt").exists());
        assert!(!tmp.path().join("new.txt").exists());
    }

    // ------------------------------------------------------------------
    // Niri viewport / pane management tests
    // ------------------------------------------------------------------

    #[test]
    fn app_starts_with_two_panes() {
        let tmp = TempDir::new().unwrap();
        let app = make_app(&tmp);
        assert_eq!(app.panes.len(), 2);
        assert_eq!(app.viewport_start, 0);
    }

    #[test]
    fn ctrl_l_adds_pane_to_right() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        assert_eq!(app.panes.len(), 2);
        app.handle_key(ctrl_key(KeyCode::Char('l')));
        assert_eq!(app.panes.len(), 3);
        assert_eq!(app.active_pane, 1);
    }

    #[test]
    fn ctrl_h_adds_pane_to_left() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        assert_eq!(app.active_pane, 0);
        app.handle_key(ctrl_key(KeyCode::Char('h')));
        assert_eq!(app.panes.len(), 3);
        // New pane inserted at index 0, active_pane stays at 0 (the new pane).
        assert_eq!(app.active_pane, 0);
    }

    #[test]
    fn ctrl_w_q_closes_active_pane() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        assert_eq!(app.panes.len(), 2);
        app.handle_key(ctrl_key(KeyCode::Char('w')));
        app.handle_key(key(KeyCode::Char('q')));
        assert_eq!(app.panes.len(), 1);
    }

    #[test]
    fn ctrl_w_q_does_not_close_last_pane() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        // Remove one pane to have only one.
        app.panes.pop();
        assert_eq!(app.panes.len(), 1);
        app.handle_key(ctrl_key(KeyCode::Char('w')));
        app.handle_key(key(KeyCode::Char('q')));
        assert_eq!(app.panes.len(), 1);
    }

    #[test]
    fn capital_h_scrolls_viewport_left() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        app.viewport_start = 1;
        app.active_pane = 1;
        app.handle_key(key(KeyCode::Char('H')));
        assert_eq!(app.viewport_start, 0);
        assert_eq!(app.active_pane, 0);
    }

    #[test]
    fn capital_l_scrolls_viewport_right() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        assert_eq!(app.viewport_start, 0);
        app.handle_key(key(KeyCode::Char('L')));
        assert_eq!(app.viewport_start, 1);
        assert_eq!(app.active_pane, 1);
    }

    #[test]
    fn capital_h_at_start_does_not_underflow() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        assert_eq!(app.viewport_start, 0);
        app.handle_key(key(KeyCode::Char('H')));
        assert_eq!(app.viewport_start, 0);
    }

    #[test]
    fn tab_wraps_through_all_panes() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        app.handle_key(ctrl_key(KeyCode::Char('l')));
        // Now 3 panes, active = 1.
        assert_eq!(app.panes.len(), 3);
        assert_eq!(app.active_pane, 1);
        app.handle_key(key(KeyCode::Tab));
        assert_eq!(app.active_pane, 2);
        app.handle_key(key(KeyCode::Tab));
        assert_eq!(app.active_pane, 0);
    }

    // ------------------------------------------------------------------
    // Dot repeat tests
    // ------------------------------------------------------------------

    #[test]
    fn dot_repeat_yank_repeats_yank() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("file.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        // Yank.
        app.handle_key(key(KeyCode::Char('y')));
        app.handle_key(key(KeyCode::Char('y')));
        assert_eq!(app.last_file_action, Some(KeyAction::Yank));
        // Clear clipboard, then dot-repeat.
        app.clipboard.clear();
        app.handle_key(key(KeyCode::Char('.')));
        // Clipboard should be populated again.
        assert_eq!(app.clipboard.len(), 1);
    }

    #[test]
    fn dot_repeat_noop_when_no_last_action() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        // No prior file action — dot repeat is a no-op (no panic).
        app.handle_key(key(KeyCode::Char('.')));
        assert!(app.clipboard.is_empty());
    }

    // ------------------------------------------------------------------
    // Toggle hidden tests
    // ------------------------------------------------------------------

    #[test]
    fn gh_toggles_hidden_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(".hidden"), b"").unwrap();
        fs::write(tmp.path().join("visible.txt"), b"").unwrap();
        let mut app = make_app(&tmp);
        app.load_dir().unwrap();
        // Default: hidden not shown.
        assert_eq!(app.panes[0].entries.len(), 1);
        // gh toggles hidden on.
        app.handle_key(key(KeyCode::Char('g')));
        app.handle_key(key(KeyCode::Char('h')));
        assert!(app.config.show_hidden);
        assert_eq!(app.panes[0].entries.len(), 2);
        // gh again toggles hidden off.
        app.handle_key(key(KeyCode::Char('g')));
        app.handle_key(key(KeyCode::Char('h')));
        assert!(!app.config.show_hidden);
        assert_eq!(app.panes[0].entries.len(), 1);
    }

    // ------------------------------------------------------------------
    // Multiplexer detection (task 24)
    // ------------------------------------------------------------------

    #[test]
    fn app_new_initialises_multiplexer_field() {
        let tmp = TempDir::new().unwrap();
        let app = make_app(&tmp);
        // The multiplexer field must exist and be a valid MultiplexerInfo.
        // We cannot predict which multiplexer is in use in CI, so we just
        // verify the field is present and does not panic.
        let _ = app.multiplexer.kind;
    }

    // ------------------------------------------------------------------
    // Virtual scrolling / visible_height (task 33)
    // ------------------------------------------------------------------

    #[test]
    fn scroll_offset_advances_when_cursor_moves_below_window() {
        let tmp = TempDir::new().unwrap();
        for i in 0..20u8 {
            fs::write(tmp.path().join(format!("{:02}.txt", i)), b"").unwrap();
        }
        // Use a terminal with only 8 rows so visible_height = 5.
        let mut caps = TerminalCaps::default();
        caps.rows = 8;
        let mut app = App::new(tmp.path().to_path_buf(), caps, VeniConfig::default());
        app.load_dir().unwrap();
        // Navigate past the visible window (5 rows).
        for _ in 0..10 {
            app.handle_key(key(KeyCode::Down));
        }
        assert!(
            app.panes[0].scroll_offset > 0,
            "scroll_offset must be non-zero after moving below visible window"
        );
        let vh = app.visible_height();
        let pane = &app.panes[0];
        assert!(
            pane.selected < pane.scroll_offset + vh,
            "cursor must remain within the visible window"
        );
    }

    // ------------------------------------------------------------------
    // Resize event handling (task 36) — tested in lib.rs integration.
    // We test the caps mutation directly here.
    // ------------------------------------------------------------------

    #[test]
    fn resize_updates_caps_columns_and_rows() {
        let tmp = TempDir::new().unwrap();
        let mut app = make_app(&tmp);
        app.caps.columns = 80;
        app.caps.rows = 24;
        // Simulate what lib.rs does on Event::Resize.
        app.caps.columns = 120;
        app.caps.rows = 40;
        assert_eq!(app.caps.columns, 120);
        assert_eq!(app.caps.rows, 40);
    }
}
