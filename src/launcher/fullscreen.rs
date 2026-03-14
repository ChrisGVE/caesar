use std::path::Path;

use crate::{
    error::{Result, VidiError},
    registry::ToolSpec,
    terminal::TerminalCaps,
    theme::ThemeMapper,
};

use super::args::build_args;

/// Launch the tool for full-screen viewing by exec()-ing into it, replacing
/// the vidi process.
///
/// On Unix, `exec()` replaces the current process image so this function
/// never returns on success.  The return type is `Result<()>` so callers
/// can use `?` for the error path only.
pub fn launch_fullscreen(
    spec: &ToolSpec,
    file: &Path,
    mapper: &ThemeMapper<'_>,
    caps: &TerminalCaps,
) -> Result<()> {
    // Use a placeholder line count for fullscreen (pagers handle scrolling).
    let lines = caps.rows.max(24);
    let args = build_args(spec, file, mapper, caps, lines, false);

    exec_tool(spec.binary, &args)
}

/// Replace the current process with `binary args…` via `execvp`.
///
/// On non-Unix targets we fall back to `spawn` + `wait`.
fn exec_tool(binary: &str, args: &[String]) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = std::process::Command::new(binary).args(args).exec();
        // exec() only returns on error.
        Err(VidiError::ToolFailed {
            tool: binary.to_string(),
            code: err.raw_os_error().unwrap_or(1),
        })
    }

    #[cfg(not(unix))]
    {
        let status = std::process::Command::new(binary)
            .args(args)
            .status()
            .map_err(|_| VidiError::ToolNotFound {
                tool: binary.to_string(),
            })?;
        if status.success() {
            Ok(())
        } else {
            Err(VidiError::ToolFailed {
                tool: binary.to_string(),
                code: status.code().unwrap_or(1),
            })
        }
    }
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

    /// Verify that `build_args` called from `launch_fullscreen` would produce
    /// the correct argument shape without actually exec()-ing.
    #[test]
    fn fullscreen_args_end_with_file_path() {
        let theme = builtin_theme("catppuccin-mocha").unwrap();
        let mapper = ThemeMapper::new(&theme);
        let bat = TEXT_TOOLS.iter().find(|s| s.binary == "bat").unwrap();
        let caps = default_caps();
        let file = Path::new("/tmp/test.txt");

        let lines = caps.rows.max(24);
        let args = build_args(bat, file, &mapper, &caps, lines, false);
        assert_eq!(args.last().unwrap(), "/tmp/test.txt");
    }

    #[test]
    fn fullscreen_args_contain_paging_always_for_bat() {
        let theme = builtin_theme("catppuccin-mocha").unwrap();
        let mapper = ThemeMapper::new(&theme);
        let bat = TEXT_TOOLS.iter().find(|s| s.binary == "bat").unwrap();
        let caps = default_caps();
        let file = Path::new("/tmp/test.txt");

        let lines = caps.rows.max(24);
        let args = build_args(bat, file, &mapper, &caps, lines, false);
        assert!(
            args.iter().any(|a| a.contains("always")),
            "fullscreen bat should use --paging=always; got: {args:?}"
        );
    }

    #[test]
    fn fullscreen_args_have_no_unexpanded_placeholders() {
        let theme = builtin_theme("catppuccin-mocha").unwrap();
        let mapper = ThemeMapper::new(&theme);
        let bat = TEXT_TOOLS.iter().find(|s| s.binary == "bat").unwrap();
        let caps = default_caps();
        let file = Path::new("/tmp/test.txt");

        let lines = caps.rows.max(24);
        let args = build_args(bat, file, &mapper, &caps, lines, false);
        for arg in &args {
            assert!(!arg.contains('{'), "unexpanded placeholder: {arg}");
        }
    }
}
