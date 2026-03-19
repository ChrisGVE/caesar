use crate::app::{App, DirEntry, LayoutMode, Mode};
use crate::pane::Pane;
use caesar_common::terminal::MultiplexerKind;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Main draw routine.  Splits the terminal into three rows:
///   1. Directory listing area (fills remaining space, split equally across visible panes)
///   2. Status bar / command line (1 line)
pub fn draw(f: &mut Frame, app: &mut App) {
    if app.mode == Mode::Help {
        draw_help(f, app);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(f.area());

    draw_panes(f, app, chunks[0]);
    draw_status(f, app, chunks[1]);
}

// ---------------------------------------------------------------------------
// Multi-pane layout
// ---------------------------------------------------------------------------

fn draw_panes(f: &mut Frame, app: &mut App, area: Rect) {
    match app.layout_mode {
        LayoutMode::Single => {
            // Full-width single pane — only the active pane.
            render_pane(f, app, app.active_pane, area);
        }
        LayoutMode::DualBalanced => {
            // Niri-style: show as many panes as fit.
            draw_panes_niri(f, app, area);
        }
        LayoutMode::SinglePreview => {
            // Left: active directory pane. Right: preview pane.
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);
            render_pane(f, app, app.active_pane, chunks[0]);
            draw_preview_pane(f, app, chunks[1]);
        }
    }
}

/// Niri-style multi-pane layout (used for DualBalanced and general niri scrolling).
fn draw_panes_niri(f: &mut Frame, app: &mut App, area: Rect) {
    const MIN_PANE_WIDTH: u16 = 20;
    let total_panes = app.panes.len();
    let max_visible = if area.width >= MIN_PANE_WIDTH {
        (area.width / MIN_PANE_WIDTH) as usize
    } else {
        1
    };
    let visible_count = max_visible.min(total_panes.saturating_sub(app.viewport_start));
    let visible_count = visible_count.max(1);

    let constraints: Vec<Constraint> = (0..visible_count)
        .map(|_| Constraint::Ratio(1, visible_count as u32))
        .collect();

    let pane_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    for (slot, area) in pane_areas.iter().enumerate() {
        let pane_idx = app.viewport_start + slot;
        if pane_idx < app.panes.len() {
            render_pane(f, app, pane_idx, *area);
        }
    }
}

/// Preview pane — shows a placeholder until vidi library API is integrated.
fn draw_preview_pane(f: &mut Frame, app: &App, area: Rect) {
    let preview_text = if let Some(entry) = app.active().current_entry() {
        if entry.is_dir {
            format!("[directory: {}]", entry.name)
        } else {
            format!("[preview: {}]", entry.name)
        }
    } else {
        "[no selection]".to_string()
    };

    let block = Block::default()
        .title(" Preview ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(preview_text)
        .block(block)
        .style(Style::default().fg(Color::DarkGray));

    f.render_widget(paragraph, area);
}

/// Render a single pane at `pane_idx` into the given `area`.
fn render_pane(f: &mut Frame, app: &mut App, pane_idx: usize, area: Rect) {
    let is_active = pane_idx == app.active_pane;
    let is_renaming = is_active && app.mode == Mode::Insert;

    // Build block with path title.
    let title = truncate_path(
        &app.panes[pane_idx].cwd.to_string_lossy(),
        area.width as usize,
    );
    let block = if is_active {
        Block::default()
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .title(Span::styled(
                title,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
    } else {
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(title, Style::default().fg(Color::DarkGray)))
    };

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Only show visual/selection highlights in the active pane.
    let visual_range = if is_active && app.mode == Mode::Visual {
        Some(app.visual_range())
    } else {
        None
    };

    let pane: &Pane = &app.panes[pane_idx];
    let all_entries: &[DirEntry] = &pane.entries;
    let selected = pane.selected;
    let scroll_offset = pane.scroll_offset;

    // Virtual scrolling: only render the visible slice.
    let visible_height = inner.height as usize;
    let end = (scroll_offset + visible_height).min(all_entries.len());
    let entries = &all_entries[scroll_offset..end];

    let selection = if is_active {
        &app.selection
    } else {
        // Return an empty set view for inactive pane.
        &std::collections::HashSet::new()
    };
    let search_matches: &[usize] = if is_active { &app.search_matches } else { &[] };

    // When renaming, snapshot the rename buffer to use in formatting.
    let rename_buffer = if is_renaming {
        Some(app.rename_buffer.clone())
    } else {
        None
    };

    let items: Vec<ListItem> = entries
        .iter()
        .enumerate()
        .map(|(slot, e)| {
            // Absolute index into all_entries.
            let i = scroll_offset + slot;
            let line = if is_renaming && i == selected {
                // Show rename buffer for the currently selected entry.
                format_rename_entry(e, rename_buffer.as_deref().unwrap_or(""))
            } else {
                format_entry(e)
            };
            let in_visual = visual_range
                .as_ref()
                .map(|r| r.contains(&i))
                .unwrap_or(false);
            let in_selection = selection.contains(&i);
            let is_search_match = search_matches.contains(&i);

            if in_visual || in_selection {
                ListItem::new(line).style(Style::default().bg(Color::DarkGray).fg(Color::Yellow))
            } else if is_search_match {
                ListItem::new(line).style(Style::default().bg(Color::DarkGray).fg(Color::Green))
            } else {
                ListItem::new(line)
            }
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    // ListState selection is relative to the visible slice.
    let visible_selected = selected.saturating_sub(scroll_offset);
    let mut state = ListState::default();
    state.select(Some(visible_selected));
    f.render_stateful_widget(list, inner, &mut state);
}

/// Truncate a path string to fit within `max_width`, using `...` prefix.
fn truncate_path(path: &str, max_width: usize) -> String {
    // Reserve space for borders (2) and some padding (2).
    let available = max_width.saturating_sub(4);
    if path.len() <= available {
        path.to_string()
    } else {
        let keep = available.saturating_sub(3);
        let start = path.len().saturating_sub(keep);
        format!("...{}", &path[start..])
    }
}

// ---------------------------------------------------------------------------
// Directory listing helpers
// ---------------------------------------------------------------------------

/// Return a nerd-font icon for a directory entry.
/// Extension is checked first; file kind is the fallback for files.
fn icon_for_entry(entry: &DirEntry) -> &'static str {
    if entry.is_dir {
        return "\u{f07b}"; //
    }
    let ext = entry.name.rsplit_once('.').map(|(_, e)| e).unwrap_or("");
    match ext.to_lowercase().as_str() {
        // Rust
        "rs" => "\u{e7a8}", //
        // Python
        "py" => "\u{e606}", //
        // JavaScript / TypeScript
        "js" | "mjs" | "cjs" => "\u{e74e}", //
        "ts" | "mts" | "cts" => "\u{e628}", //
        // Web
        "html" | "htm" => "\u{e736}",          //
        "css" | "scss" | "sass" => "\u{e749}", //
        // Config / data
        "json" => "\u{e60b}",                  //
        "toml" | "yaml" | "yml" => "\u{e615}", //
        "xml" => "\u{e619}",                   //
        // Text / docs
        "md" | "markdown" => "\u{e73e}", //
        "txt" => "\u{f15c}",             //
        "pdf" => "\u{f1c1}",             //
        // Images
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" => "\u{f1c5}", //
        // Audio / video
        "mp3" | "flac" | "ogg" | "wav" => "\u{f1c7}", //
        "mp4" | "mkv" | "avi" | "mov" | "webm" => "\u{f1c8}", //
        // Archives
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" => "\u{f1c6}", //
        // Shell / scripts
        "sh" | "bash" | "zsh" | "fish" => "\u{f489}", //
        // Go
        "go" => "\u{e626}", //
        // C / C++
        "c" | "h" => "\u{e61e}",                    //
        "cpp" | "cc" | "cxx" | "hpp" => "\u{e61d}", //
        // Java / Kotlin
        "java" => "\u{e738}",       //
        "kt" | "kts" => "\u{e634}", //
        // Lua
        "lua" => "\u{e620}", //
        // Binary / executable
        "exe" | "bin" | "out" => "\u{f489}", //
        // Fallback
        _ => "\u{f15b}", //
    }
}

fn format_entry(entry: &DirEntry) -> Line<'static> {
    let icon = icon_for_entry(entry);

    if entry.is_symlink {
        let target_str = entry
            .symlink_target
            .as_ref()
            .map(|t| t.to_string_lossy().into_owned())
            .unwrap_or_else(|| "?".to_string());
        let name = format!("{} {} \u{2192} {}", icon, entry.name, target_str);
        let style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);
        return Line::from(Span::styled(name, style));
    }

    let name = if entry.is_dir {
        format!("{} {}/", icon, entry.name)
    } else {
        format!("{} {}", icon, entry.name)
    };

    let size_str = if entry.is_dir {
        String::new()
    } else {
        format_size(entry.size)
    };

    let style = if entry.is_dir {
        Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    if size_str.is_empty() {
        Line::from(Span::styled(name, style))
    } else {
        Line::from(vec![
            Span::styled(format!("{:<37}", name), style),
            Span::raw(size_str),
        ])
    }
}

/// Format an entry while it is being renamed: replace the name with the
/// current rename buffer and show a cursor indicator.
fn format_rename_entry(entry: &DirEntry, buffer: &str) -> Line<'static> {
    let icon = icon_for_entry(entry);
    let display = format!("{} {}|", icon, buffer);
    Line::from(Span::styled(
        display,
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ))
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1_024;
    const MB: u64 = 1_024 * KB;
    const GB: u64 = 1_024 * MB;
    if bytes >= GB {
        format!("{:.1}G", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}M", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}K", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

// ---------------------------------------------------------------------------
// Status bar / command line
// ---------------------------------------------------------------------------

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    match app.mode {
        Mode::Command => {
            let prompt = format!(":{}", app.command_input);
            let para = Paragraph::new(prompt).style(
                Style::default()
                    .fg(Color::White)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );
            f.render_widget(para, area);
        }
        Mode::Search => {
            let prompt = format!("/{}", app.search_query);
            let para = Paragraph::new(prompt).style(
                Style::default()
                    .fg(Color::White)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );
            f.render_widget(para, area);
        }
        Mode::Insert => {
            let prompt = format!("rename: {}|", app.rename_buffer);
            let para = Paragraph::new(prompt).style(
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );
            f.render_widget(para, area);
        }
        _ => draw_normal_status(f, app, area),
    }
}

fn draw_normal_status(f: &mut Frame, app: &App, area: Rect) {
    let path_str = app.active().cwd.to_string_lossy().into_owned();
    let mode_str = app.mode.to_string();

    // Build a multiplexer suffix, e.g. " [tmux]".
    let mux_suffix = match app.multiplexer.kind {
        MultiplexerKind::Tmux => " [tmux]".to_string(),
        MultiplexerKind::Zellij => " [zellij]".to_string(),
        MultiplexerKind::Cmux => " [screen]".to_string(),
        MultiplexerKind::None => String::new(),
    };
    let layout_str = format!(" [{}]", app.layout_mode);
    let right_str = format!("{}{}{}", layout_str, mux_suffix, mode_str);

    let inner_width = area.width as usize;
    let right_len = right_str.len();
    let path_display = if path_str.len() + right_len + 1 > inner_width {
        let keep = inner_width.saturating_sub(right_len + 4);
        let start = path_str.len().saturating_sub(keep);
        format!("...{}", &path_str[start..])
    } else {
        path_str.clone()
    };

    let padding = inner_width.saturating_sub(path_display.len() + right_len);
    let status_line = format!("{}{}{}", path_display, " ".repeat(padding), right_str);

    let status = Paragraph::new(status_line).style(
        Style::default()
            .fg(Color::Black)
            .bg(Color::Blue)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(status, area);
}

// ---------------------------------------------------------------------------
// Help overlay
// ---------------------------------------------------------------------------

fn draw_help(f: &mut Frame, app: &mut App) {
    let help_lines = help_content();
    let total = help_lines.len();

    // Clamp scroll offset.
    let visible = f.area().height.saturating_sub(2) as usize; // borders
    if app.help_scroll_offset + visible > total {
        app.help_scroll_offset = total.saturating_sub(visible);
    }

    let items: Vec<ListItem> = help_lines
        .into_iter()
        .skip(app.help_scroll_offset)
        .map(|line| ListItem::new(line))
        .collect();

    let block = Block::default()
        .title(" Help — press q/Esc to close, j/k to scroll ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let list = List::new(items).block(block);
    f.render_widget(list, f.area());
}

fn help_content() -> Vec<Line<'static>> {
    let section = |title: &'static str| {
        Line::from(Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
    };
    let key = |k: &'static str, desc: &'static str| {
        Line::from(vec![
            Span::styled(
                format!("  {:<16}", k),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(desc),
        ])
    };
    let blank = || Line::from("");

    vec![
        section("Navigation"),
        key("j / Down", "Move cursor down"),
        key("k / Up", "Move cursor up"),
        key("l / Right / Enter", "Enter directory / follow symlink"),
        key("h / Left / Bksp", "Go to parent directory"),
        key("gg", "Go to first entry"),
        key("G", "Go to last entry"),
        blank(),
        section("Pane Management"),
        key("Tab", "Switch active pane"),
        key("H (shift)", "Scroll workspace left (niri)"),
        key("L (shift)", "Scroll workspace right (niri)"),
        key("Ctrl-h", "Add new pane to the left"),
        key("Ctrl-l", "Add new pane to the right"),
        key("Ctrl-w q", "Close active pane"),
        blank(),
        section("File Operations"),
        key("yy", "Yank (copy) file to clipboard"),
        key("dd", "Cut file to clipboard"),
        key("p", "Paste from clipboard"),
        key("u", "Undo last operation"),
        key("Ctrl-r", "Redo"),
        key(".", "Repeat last file operation"),
        blank(),
        section("Rename"),
        key("cw / ciw", "Rename file (enter Insert mode)"),
        key("Enter (insert)", "Confirm rename"),
        key("Esc (insert)", "Cancel rename"),
        blank(),
        section("Selection"),
        key("v", "Enter Visual mode (range select)"),
        key("V", "Toggle selection on current entry"),
        key("Esc (visual)", "Exit Visual mode"),
        blank(),
        section("Search"),
        key("/", "Search forward"),
        key("?", "Search backward"),
        key("n", "Next search match"),
        key("N", "Previous search match"),
        key("Esc (search)", "Cancel search"),
        blank(),
        section("Command Mode"),
        key(":", "Enter command mode"),
        key(":q", "Quit veni"),
        key(":cd <path>", "Change directory"),
        key(":set hidden", "Show hidden files"),
        key(":set nohidden", "Hide hidden files"),
        key(":help", "Show this help"),
        blank(),
        section("Toggles"),
        key("gh", "Toggle hidden files"),
        blank(),
        section("General"),
        key("q", "Quit"),
        key("Ctrl-c", "Force quit"),
        key("F1", "Show this help"),
        key(":help", "Show this help"),
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_size_bytes() {
        assert_eq!(format_size(0), "0B");
        assert_eq!(format_size(512), "512B");
    }

    #[test]
    fn format_size_kilobytes() {
        assert_eq!(format_size(1_024), "1.0K");
        assert_eq!(format_size(2_048), "2.0K");
    }

    #[test]
    fn format_size_megabytes() {
        assert_eq!(format_size(1_048_576), "1.0M");
    }

    #[test]
    fn format_size_gigabytes() {
        assert_eq!(format_size(1_073_741_824), "1.0G");
    }

    #[test]
    fn format_entry_dir_appends_slash() {
        let entry = DirEntry {
            name: "docs".to_string(),
            path: "/tmp/docs".into(),
            is_dir: true,
            is_symlink: false,
            symlink_target: None,
            size: 0,
            modified: None,
        };
        let line = format_entry(&entry);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("docs/"), "directory entry must end with /");
    }

    #[test]
    fn format_entry_file_no_slash() {
        let entry = DirEntry {
            name: "readme.txt".to_string(),
            path: "/tmp/readme.txt".into(),
            is_dir: false,
            is_symlink: false,
            symlink_target: None,
            size: 1_024,
            modified: None,
        };
        let line = format_entry(&entry);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(
            !text.contains("readme.txt/"),
            "file entry must not end with /"
        );
        assert!(text.contains("1.0K"), "file entry must show size");
    }

    #[test]
    fn format_entry_dir_has_no_size() {
        let entry = DirEntry {
            name: "bin".to_string(),
            path: "/usr/bin".into(),
            is_dir: true,
            is_symlink: false,
            symlink_target: None,
            size: 4096,
            modified: None,
        };
        let line = format_entry(&entry);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(
            !text.contains("4.0K"),
            "directory must not display its size"
        );
    }

    // ------------------------------------------------------------------
    // truncate_path tests
    // ------------------------------------------------------------------

    #[test]
    fn truncate_path_short_string_unchanged() {
        let path = "/home/user";
        assert_eq!(truncate_path(path, 80), path);
    }

    #[test]
    fn truncate_path_long_string_has_ellipsis() {
        let path = "/very/long/path/that/exceeds/the/available/width/by/a/lot";
        let result = truncate_path(path, 20);
        assert!(result.starts_with("..."));
    }

    // ------------------------------------------------------------------
    // icon_for_entry tests
    // ------------------------------------------------------------------

    #[test]
    fn icon_for_dir() {
        let entry = DirEntry {
            name: "mydir".to_string(),
            path: "/tmp/mydir".into(),
            is_dir: true,
            is_symlink: false,
            symlink_target: None,
            size: 0,
            modified: None,
        };
        let icon = icon_for_entry(&entry);
        assert_eq!(icon, "\u{f07b}");
    }

    #[test]
    fn icon_for_rust_file() {
        let entry = DirEntry {
            name: "main.rs".to_string(),
            path: "/tmp/main.rs".into(),
            is_dir: false,
            is_symlink: false,
            symlink_target: None,
            size: 0,
            modified: None,
        };
        let icon = icon_for_entry(&entry);
        assert_eq!(icon, "\u{e7a8}");
    }

    #[test]
    fn icon_for_unknown_extension() {
        let entry = DirEntry {
            name: "weird.xyz".to_string(),
            path: "/tmp/weird.xyz".into(),
            is_dir: false,
            is_symlink: false,
            symlink_target: None,
            size: 0,
            modified: None,
        };
        let icon = icon_for_entry(&entry);
        assert_eq!(icon, "\u{f15b}"); // generic file
    }

    #[test]
    fn icon_for_no_extension() {
        let entry = DirEntry {
            name: "Makefile".to_string(),
            path: "/tmp/Makefile".into(),
            is_dir: false,
            is_symlink: false,
            symlink_target: None,
            size: 0,
            modified: None,
        };
        let icon = icon_for_entry(&entry);
        assert_eq!(icon, "\u{f15b}"); // generic file
    }

    #[test]
    fn format_entry_includes_icon() {
        let entry = DirEntry {
            name: "main.rs".to_string(),
            path: "/src/main.rs".into(),
            is_dir: false,
            is_symlink: false,
            symlink_target: None,
            size: 0,
            modified: None,
        };
        let line = format_entry(&entry);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        // The icon character must appear in the formatted text.
        assert!(
            text.contains('\u{e7a8}'),
            "Rust icon must be in formatted entry"
        );
    }

    #[test]
    fn format_rename_entry_shows_buffer_and_cursor() {
        let entry = DirEntry {
            name: "old.txt".to_string(),
            path: "/tmp/old.txt".into(),
            is_dir: false,
            is_symlink: false,
            symlink_target: None,
            size: 0,
            modified: None,
        };
        let line = format_rename_entry(&entry, "new_name");
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("new_name"), "rename buffer must appear");
        assert!(text.contains('|'), "cursor indicator must appear");
    }

    // ------------------------------------------------------------------
    // Symlink display tests (task 35)
    // ------------------------------------------------------------------

    #[test]
    fn format_entry_symlink_shows_arrow_and_target() {
        let entry = DirEntry {
            name: "mylink".to_string(),
            path: "/tmp/mylink".into(),
            is_dir: false,
            is_symlink: true,
            symlink_target: Some("/real/target".into()),
            size: 0,
            modified: None,
        };
        let line = format_entry(&entry);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(
            text.contains("mylink"),
            "symlink entry must contain the link name"
        );
        // Unicode right arrow U+2192.
        assert!(
            text.contains('\u{2192}'),
            "symlink entry must contain the arrow character"
        );
        assert!(
            text.contains("/real/target"),
            "symlink entry must contain the target path"
        );
    }

    #[test]
    fn format_entry_symlink_style_is_cyan() {
        let entry = DirEntry {
            name: "link".to_string(),
            path: "/tmp/link".into(),
            is_dir: false,
            is_symlink: true,
            symlink_target: Some("/somewhere".into()),
            size: 0,
            modified: None,
        };
        let line = format_entry(&entry);
        // All spans for symlinks should use Cyan foreground.
        for span in &line.spans {
            assert_eq!(
                span.style.fg,
                Some(Color::Cyan),
                "symlink spans must use Cyan color"
            );
        }
    }
}
