use crate::app::{App, DirEntry, Mode};
use crate::pane::Pane;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Main draw routine.  Splits the terminal into three rows:
///   1. Directory listing area (fills remaining space, split 50/50 into two panes)
///   2. Status bar / command line (1 line)
pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(f.area());

    draw_panes(f, app, chunks[0]);
    draw_status(f, app, chunks[1]);
}

// ---------------------------------------------------------------------------
// Two-pane layout
// ---------------------------------------------------------------------------

fn draw_panes(f: &mut Frame, app: &mut App, area: Rect) {
    let pane_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Render both panes, passing information about which is active.
    render_pane(f, app, 0, pane_areas[0]);
    render_pane(f, app, 1, pane_areas[1]);
}

/// Render a single pane at `pane_idx` into the given `area`.
fn render_pane(f: &mut Frame, app: &mut App, pane_idx: usize, area: Rect) {
    let is_active = pane_idx == app.active_pane;

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
    let entries: &[DirEntry] = &pane.entries;
    let selected = pane.selected;
    let selection = if is_active {
        &app.selection
    } else {
        // Return an empty set view for inactive pane.
        &std::collections::HashSet::new()
    };
    let search_matches: &[usize] = if is_active { &app.search_matches } else { &[] };

    let items: Vec<ListItem> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let line = format_entry(e);
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

    let mut state = ListState::default();
    state.select(Some(selected));
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

fn format_entry(entry: &DirEntry) -> Line<'static> {
    let name = if entry.is_dir {
        format!("{}/", entry.name)
    } else {
        entry.name.clone()
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
            Span::styled(format!("{:<35}", name), style),
            Span::raw(size_str),
        ])
    }
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
        _ => draw_normal_status(f, app, area),
    }
}

fn draw_normal_status(f: &mut Frame, app: &App, area: Rect) {
    let path_str = app.active().cwd.to_string_lossy().into_owned();
    let mode_str = app.mode.to_string();

    let inner_width = area.width as usize;
    let mode_len = mode_str.len();
    let path_display = if path_str.len() + mode_len + 1 > inner_width {
        let keep = inner_width.saturating_sub(mode_len + 4);
        let start = path_str.len().saturating_sub(keep);
        format!("...{}", &path_str[start..])
    } else {
        path_str.clone()
    };

    let padding = inner_width.saturating_sub(path_display.len() + mode_len);
    let status_line = format!("{}{}{}", path_display, " ".repeat(padding), mode_str);

    let status = Paragraph::new(status_line).style(
        Style::default()
            .fg(Color::Black)
            .bg(Color::Blue)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(status, area);
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
}
