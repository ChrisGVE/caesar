//! Toggle mode viewer for LaTeX (`.tex`) and Typst (`.typ`) files.
//!
//! Presents two views:
//! - **Source** — bat syntax-highlighted source (always available).
//! - **Rendered** — compiled PDF page displayed as a PNG via chafa.
//!
//! Key bindings: `s`/`S` → source, `r`/`R` → rendered, `q`/`Q`/Esc/Ctrl-C → quit.

mod compile;
mod event_loop;
mod render;

use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

use crate::{
    error::{Result, VidiError},
    registry::{is_available, TEXT_TOOLS},
    terminal::TerminalCaps,
    theme::ThemeMapper,
};

use compile::{compile_latex, compile_typst};
use event_loop::{run_event_loop, View};
use render::{display_png, render_pdf_page};

/// Determine whether the file extension indicates LaTeX.
fn is_latex(file: &Path) -> bool {
    file.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("tex"))
}

/// Print a status message to stdout (raw mode is active, use CR+LF).
fn print_status(msg: &str) {
    print!("\r{msg}\r\n");
    let _ = std::io::Write::flush(&mut std::io::stdout());
}

/// Build the bat source-view callback for the given file.
///
/// Returns a closure that runs `bat` (or `cat` as fallback) and shows the
/// file source.  Because `bat` is exec'd as a child — not `exec()` replacing
/// the process — it exits back to the event loop.
fn make_source_viewer<'a>(
    file: &'a Path,
    mapper: &'a ThemeMapper<'a>,
    caps: &'a TerminalCaps,
) -> impl FnMut() -> Result<()> + 'a {
    move || {
        let spec = if is_available("bat") {
            TEXT_TOOLS.iter().find(|s| s.binary == "bat").unwrap()
        } else {
            TEXT_TOOLS.last().unwrap()
        };

        let theme = mapper.bat_theme_name();
        let lines = caps.rows.max(24);

        let mut cmd = Command::new(spec.binary);

        if spec.binary == "bat" {
            cmd.args([
                "--paging=never",
                "--style=numbers,changes,header",
                "--color=always",
            ])
            .arg(format!("--theme={theme}"))
            .arg(format!("--terminal-width={}", caps.columns.max(80)))
            .arg(format!("--line-range=1:{}", lines * 4));
        }
        cmd.arg(file);

        let status = cmd.status().map_err(VidiError::Io)?;
        if status.success() {
            Ok(())
        } else {
            Err(VidiError::ToolFailed {
                tool: spec.binary.to_string(),
                code: status.code().unwrap_or(1),
            })
        }
    }
}

/// Launch toggle mode for a LaTeX or Typst file.
///
/// Presents an interactive full-screen viewer where the user can switch
/// between source and rendered views with single keypresses.
///
/// When neither `tectonic` (LaTeX) nor `typst` (Typst) is installed, or
/// when `mutool` is not installed, only the source view is available and
/// the user is informed.
///
/// # Errors
///
/// Propagates I/O errors from crossterm or child processes.
pub fn launch_toggle(file: &Path, mapper: &ThemeMapper<'_>, caps: &TerminalCaps) -> Result<()> {
    // Create a temporary directory that lives for the session.
    let tmpdir: TempDir = tempfile::tempdir()?;
    let outdir = tmpdir.path().to_path_buf();

    let latex = is_latex(file);

    // Attempt compilation once, cache the PDF path.
    let pdf_result = if latex {
        compile_latex(file, &outdir, is_available)
    } else {
        compile_typst(file, &outdir, is_available)
    };

    // Determine render availability.
    let render_available = pdf_result.is_ok() && is_available("mutool") && is_available("chafa");

    if !render_available {
        let reason = if pdf_result.is_err() {
            if latex {
                "tectonic not installed"
            } else {
                "typst not installed"
            }
        } else if !is_available("mutool") {
            "mutool not installed"
        } else {
            "chafa not installed"
        };
        print_status(&format!(
            "Rendered view unavailable: {reason}. Showing source only."
        ));
    }

    let pdf_path = pdf_result.ok();
    let outdir_clone = outdir.clone();
    let caps_cols = caps.columns;
    let caps_rows = caps.rows;

    let source_cb = make_source_viewer(file, mapper, caps);

    // Rendered callback: render PDF → PNG → display.
    let render_cb = {
        let pdf = pdf_path.clone();
        let outdir2 = outdir_clone.clone();
        move || -> Result<()> {
            let Some(ref pdf_path) = pdf else {
                print_status("Rendered view unavailable.");
                return Ok(());
            };

            let png = render_pdf_page(pdf_path, &outdir2, is_available)?;

            let disp_caps = TerminalCaps {
                graphics: caps.graphics,
                true_color: caps.true_color,
                columns: caps_cols,
                rows: caps_rows,
            };
            display_png(&png, &disp_caps, is_available)
        }
    };

    let initial = View::Source;
    run_event_loop(initial, source_cb, render_cb)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn is_latex_recognises_tex_extension() {
        assert!(is_latex(Path::new("document.tex")));
        assert!(is_latex(Path::new("/path/to/file.TEX")));
    }

    #[test]
    fn is_latex_rejects_typ_extension() {
        assert!(!is_latex(Path::new("report.typ")));
    }

    #[test]
    fn is_latex_rejects_no_extension() {
        assert!(!is_latex(Path::new("noextension")));
    }
}
