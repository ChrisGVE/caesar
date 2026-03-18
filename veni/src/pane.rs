use crate::app::DirEntry;
use crate::error::{Result, VeniError};
use std::path::PathBuf;

/// Action dispatched to a pane for navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationAction {
    Down,
    Up,
    Top,
    Bottom,
    Enter,
    Parent,
}

/// State for one file-manager pane.
#[derive(Debug, Clone)]
pub struct Pane {
    pub cwd: PathBuf,
    pub entries: Vec<DirEntry>,
    pub selected: usize,
    pub scroll_offset: usize,
}

impl Pane {
    pub fn new(path: PathBuf) -> Self {
        Self {
            cwd: path,
            entries: Vec::new(),
            selected: 0,
            scroll_offset: 0,
        }
    }

    /// Read `cwd` and populate `entries`.
    ///
    /// Sort order: directories first, then files; alphabetical within each
    /// group (case-insensitive).  Dotfiles are included only when
    /// `show_hidden` is true.
    pub fn load_dir(&mut self, show_hidden: bool) -> Result<()> {
        let read_dir = std::fs::read_dir(&self.cwd).map_err(|source| VeniError::ReadDir {
            path: self.cwd.clone(),
            source,
        })?;

        let mut entries: Vec<DirEntry> = Vec::new();
        for entry_result in read_dir {
            let entry = match entry_result {
                Ok(e) => e,
                Err(_) => continue,
            };

            let name = entry.file_name().to_string_lossy().into_owned();

            if !show_hidden && name.starts_with('.') {
                continue;
            }

            let meta = entry.metadata().ok();
            let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
            let modified = meta.and_then(|m| m.modified().ok());

            entries.push(DirEntry {
                name,
                path: entry.path(),
                is_dir,
                size,
                modified,
            });
        }

        entries.sort_by(|a, b| {
            b.is_dir
                .cmp(&a.is_dir)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        self.entries = entries;
        self.selected = 0;
        self.scroll_offset = 0;
        Ok(())
    }

    /// Apply a navigation action to this pane.
    pub fn handle_navigation(&mut self, action: NavigationAction, show_hidden: bool) {
        match action {
            NavigationAction::Down => self.move_down(),
            NavigationAction::Up => self.move_up(),
            NavigationAction::Top => self.go_top(),
            NavigationAction::Bottom => self.go_bottom(),
            NavigationAction::Enter => self.enter_dir(show_hidden),
            NavigationAction::Parent => self.go_parent(show_hidden),
        }
    }

    /// Currently highlighted entry, if any.
    pub fn current_entry(&self) -> Option<&DirEntry> {
        self.entries.get(self.selected)
    }

    // ------------------------------------------------------------------
    // Navigation primitives
    // ------------------------------------------------------------------

    fn move_down(&mut self) {
        if !self.entries.is_empty() && self.selected < self.entries.len() - 1 {
            self.selected += 1;
        }
    }

    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn go_top(&mut self) {
        self.selected = 0;
        self.scroll_offset = 0;
    }

    fn go_bottom(&mut self) {
        if !self.entries.is_empty() {
            self.selected = self.entries.len() - 1;
        }
    }

    fn enter_dir(&mut self, show_hidden: bool) {
        if let Some(entry) = self.entries.get(self.selected) {
            if entry.is_dir {
                let new_path = entry.path.clone();
                self.cwd = new_path;
                let _ = self.load_dir(show_hidden);
            }
        }
    }

    fn go_parent(&mut self, show_hidden: bool) {
        if let Some(parent) = self.cwd.parent().map(|p| p.to_path_buf()) {
            self.cwd = parent;
            let _ = self.load_dir(show_hidden);
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

    fn make_pane(dir: &TempDir) -> Pane {
        Pane::new(dir.path().to_path_buf())
    }

    #[test]
    fn load_dir_lists_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("file.txt"), b"hello").unwrap();
        let mut pane = make_pane(&tmp);
        pane.load_dir(false).unwrap();
        assert_eq!(pane.entries.len(), 1);
        assert_eq!(pane.entries[0].name, "file.txt");
    }

    #[test]
    fn load_dir_sorts_dirs_before_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("aaa.txt"), b"").unwrap();
        fs::create_dir(tmp.path().join("bbb_dir")).unwrap();
        let mut pane = make_pane(&tmp);
        pane.load_dir(false).unwrap();
        assert!(pane.entries[0].is_dir, "directory must come first");
        assert!(!pane.entries[1].is_dir);
    }

    #[test]
    fn load_dir_hides_dotfiles_by_default() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(".hidden"), b"").unwrap();
        fs::write(tmp.path().join("visible.txt"), b"").unwrap();
        let mut pane = make_pane(&tmp);
        pane.load_dir(false).unwrap();
        assert_eq!(pane.entries.len(), 1);
        assert_eq!(pane.entries[0].name, "visible.txt");
    }

    #[test]
    fn load_dir_shows_dotfiles_when_enabled() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(".hidden"), b"").unwrap();
        fs::write(tmp.path().join("visible.txt"), b"").unwrap();
        let mut pane = make_pane(&tmp);
        pane.load_dir(true).unwrap();
        assert_eq!(pane.entries.len(), 2);
    }

    #[test]
    fn load_dir_resets_cursor() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), b"").unwrap();
        fs::write(tmp.path().join("b.txt"), b"").unwrap();
        let mut pane = make_pane(&tmp);
        pane.load_dir(false).unwrap();
        pane.selected = 1;
        pane.load_dir(false).unwrap();
        assert_eq!(pane.selected, 0);
    }

    #[test]
    fn navigate_down_and_up() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), b"").unwrap();
        fs::write(tmp.path().join("b.txt"), b"").unwrap();
        let mut pane = make_pane(&tmp);
        pane.load_dir(false).unwrap();
        pane.handle_navigation(NavigationAction::Down, false);
        assert_eq!(pane.selected, 1);
        pane.handle_navigation(NavigationAction::Up, false);
        assert_eq!(pane.selected, 0);
    }

    #[test]
    fn navigate_bottom_and_top() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), b"").unwrap();
        fs::write(tmp.path().join("b.txt"), b"").unwrap();
        fs::write(tmp.path().join("c.txt"), b"").unwrap();
        let mut pane = make_pane(&tmp);
        pane.load_dir(false).unwrap();
        pane.handle_navigation(NavigationAction::Bottom, false);
        assert_eq!(pane.selected, 2);
        pane.handle_navigation(NavigationAction::Top, false);
        assert_eq!(pane.selected, 0);
        assert_eq!(pane.scroll_offset, 0);
    }

    #[test]
    fn navigate_down_at_bottom_does_not_overflow() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), b"").unwrap();
        let mut pane = make_pane(&tmp);
        pane.load_dir(false).unwrap();
        pane.handle_navigation(NavigationAction::Down, false);
        assert_eq!(pane.selected, 0);
    }

    #[test]
    fn navigate_up_at_top_does_not_underflow() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), b"").unwrap();
        let mut pane = make_pane(&tmp);
        pane.load_dir(false).unwrap();
        pane.handle_navigation(NavigationAction::Up, false);
        assert_eq!(pane.selected, 0);
    }

    #[test]
    fn navigate_enter_changes_cwd() {
        let tmp = TempDir::new().unwrap();
        let subdir = tmp.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        let mut pane = make_pane(&tmp);
        pane.load_dir(false).unwrap();
        assert_eq!(pane.entries[0].name, "subdir");
        let expected = pane.entries[0].path.clone();
        pane.handle_navigation(NavigationAction::Enter, false);
        assert_eq!(pane.cwd, expected);
    }

    #[test]
    fn navigate_parent_goes_up() {
        let tmp = TempDir::new().unwrap();
        let subdir = tmp.path().join("sub");
        fs::create_dir(&subdir).unwrap();
        let mut pane = Pane::new(subdir.clone());
        pane.load_dir(false).unwrap();
        let parent = tmp.path().to_path_buf();
        pane.handle_navigation(NavigationAction::Parent, false);
        assert_eq!(
            pane.cwd.canonicalize().unwrap_or(pane.cwd.clone()),
            parent.canonicalize().unwrap_or(parent)
        );
    }

    #[test]
    fn current_entry_returns_selected() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), b"").unwrap();
        let mut pane = make_pane(&tmp);
        pane.load_dir(false).unwrap();
        assert!(pane.current_entry().is_some());
        assert_eq!(pane.current_entry().unwrap().name, "a.txt");
    }

    #[test]
    fn current_entry_empty_pane() {
        let tmp = TempDir::new().unwrap();
        let mut pane = make_pane(&tmp);
        pane.load_dir(false).unwrap();
        assert!(pane.current_entry().is_none());
    }
}
