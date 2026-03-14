use std::path::Path;

use crate::{
    registry::ToolSpec,
    terminal::TerminalCaps,
    theme::{mapper::apply_to_args, ThemeMapper},
};

/// Build the final argument list for invoking a tool.
///
/// Selects `inline_args` or `fullscreen_args` from `spec`, expands
/// placeholder tokens (`{theme}`, `{cols}`, `{rows}`, `{lines}`, `{bytes}`),
/// then appends the file path as the final argument.
pub fn build_args(
    spec: &ToolSpec,
    file: &Path,
    mapper: &ThemeMapper<'_>,
    caps: &TerminalCaps,
    lines: u16,
    inline: bool,
) -> Vec<String> {
    let template = if inline {
        spec.inline_args
    } else {
        spec.fullscreen_args
    };

    let mut args = apply_to_args(template, mapper, caps.columns, caps.rows, lines);

    // Append the file path as the final positional argument.
    args.push(file.to_string_lossy().into_owned());

    args
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        registry::TEXT_TOOLS,
        terminal::{GraphicsProtocol, TerminalCaps},
        theme::{builtin::builtin_theme, ThemeMapper},
    };
    use std::path::Path;

    fn default_caps() -> TerminalCaps {
        TerminalCaps {
            graphics: GraphicsProtocol::HalfBlock256,
            true_color: false,
            columns: 80,
            rows: 24,
        }
    }

    fn mocha_mapper() -> (crate::theme::Theme,) {
        (builtin_theme("catppuccin-mocha").unwrap(),)
    }

    #[test]
    fn file_path_is_last_argument() {
        let (theme,) = mocha_mapper();
        let mapper = ThemeMapper::new(&theme);
        let bat = TEXT_TOOLS.iter().find(|s| s.binary == "bat").unwrap();
        let caps = default_caps();
        let file = Path::new("/tmp/test.txt");
        let args = build_args(bat, file, &mapper, &caps, 20, false);
        assert_eq!(args.last().unwrap(), "/tmp/test.txt");
    }

    #[test]
    fn inline_selects_inline_args() {
        let (theme,) = mocha_mapper();
        let mapper = ThemeMapper::new(&theme);
        let bat = TEXT_TOOLS.iter().find(|s| s.binary == "bat").unwrap();
        let caps = default_caps();
        let file = Path::new("/tmp/test.txt");
        let args = build_args(bat, file, &mapper, &caps, 20, true);
        // inline bat args include --paging=never
        assert!(
            args.iter().any(|a| a.contains("never")),
            "inline args should include --paging=never; got: {args:?}"
        );
    }

    #[test]
    fn fullscreen_selects_fullscreen_args() {
        let (theme,) = mocha_mapper();
        let mapper = ThemeMapper::new(&theme);
        let bat = TEXT_TOOLS.iter().find(|s| s.binary == "bat").unwrap();
        let caps = default_caps();
        let file = Path::new("/tmp/test.txt");
        let args = build_args(bat, file, &mapper, &caps, 20, false);
        // fullscreen bat args include --paging=always
        assert!(
            args.iter().any(|a| a.contains("always")),
            "fullscreen args should include --paging=always; got: {args:?}"
        );
    }

    #[test]
    fn placeholders_expanded_in_result() {
        let (theme,) = mocha_mapper();
        let mapper = ThemeMapper::new(&theme);
        let bat = TEXT_TOOLS.iter().find(|s| s.binary == "bat").unwrap();
        let caps = default_caps();
        let file = Path::new("/tmp/test.txt");
        let args = build_args(bat, file, &mapper, &caps, 20, true);
        // No raw placeholders should remain
        for arg in &args {
            assert!(!arg.contains('{'), "unexpanded placeholder in arg: {arg}");
        }
    }

    #[test]
    fn cat_tool_appends_file_with_no_extra_args() {
        let (theme,) = mocha_mapper();
        let mapper = ThemeMapper::new(&theme);
        let cat = TEXT_TOOLS.iter().find(|s| s.binary == "cat").unwrap();
        let caps = default_caps();
        let file = Path::new("/etc/hosts");
        let args = build_args(cat, file, &mapper, &caps, 20, false);
        // cat has no fullscreen_args, so result is just [path]
        assert_eq!(args, vec!["/etc/hosts"]);
    }
}
