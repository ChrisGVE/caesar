use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "vidi", version, about = "Universal terminal file viewer")]
struct Cli {
    /// File to view ('-' reads from stdin)
    file: String,
    /// Constrained output mode (non-interactive, writes to stdout)
    #[arg(long)]
    inline: bool,
    /// Max output lines in inline mode [default: terminal height]
    #[arg(short = 'n', long)]
    lines: Option<u16>,
    /// Theme name or path to custom theme file
    #[arg(long)]
    theme: Option<String>,
    /// Force a specific tool by name
    #[arg(long)]
    tool: Option<String>,
    /// Path to config file
    #[arg(long)]
    config: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("vidi: {e}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> vidi::error::Result<()> {
    use vidi::{
        config::load_config,
        detector::detect,
        launcher::{launch_fullscreen, launch_inline, launch_toggle},
        registry::best_tool,
        terminal::detect_capabilities,
        theme::{mapper::ThemeMapper, resolve::resolve_theme},
    };

    // Load config.
    let config = load_config(cli.config.as_deref())?;

    // Resolve active theme.
    let env_theme = std::env::var("VIDI_THEME").ok();
    let theme = resolve_theme(
        env_theme,
        cli.theme,
        Some(config.theme),
        &config.custom_themes,
    );
    let mapper = ThemeMapper::new(&theme);

    // Detect terminal capabilities.
    let caps = detect_capabilities();

    // Resolve file path (stdin handled separately).
    let file = resolve_file(&cli.file)?;

    // Detect file kind.
    let kind = detect(&file)?;

    // Resolve tool.
    let spec = best_tool(kind, cli.tool.as_deref()).ok_or_else(|| {
        vidi::error::VidiError::NoViewerAvailable {
            kind: kind.to_string(),
        }
    })?;

    // Determine line count for inline mode.
    let lines = cli.lines.unwrap_or_else(|| caps.rows.max(24));

    // Launch.
    if cli.inline {
        if spec.supports_inline {
            launch_inline(spec, &file, &mapper, &caps, lines)
        } else {
            // Fall back: plain text output via the last TEXT_TOOLS entry.
            let fallback = vidi::registry::TEXT_TOOLS.last().unwrap();
            launch_inline(fallback, &file, &mapper, &caps, lines)
        }
    } else {
        match kind {
            vidi::detector::FileKind::LaTeX | vidi::detector::FileKind::Typst => {
                launch_toggle(&file, &mapper, &caps)
            }
            _ => launch_fullscreen(spec, &file, &mapper, &caps),
        }
    }
}

/// Resolve the target file path, buffering stdin to a temp file when needed.
fn resolve_file(arg: &str) -> vidi::error::Result<PathBuf> {
    if arg == "-" {
        return read_stdin_to_tempfile();
    }
    let p = PathBuf::from(arg);
    if !p.exists() {
        return Err(vidi::error::VidiError::FileNotFound(p));
    }
    Ok(p)
}

/// Read all of stdin into a temporary file and return its path.
///
/// The file is persisted (not cleaned up on drop) so that the viewer tool
/// launched afterwards can read it.  On a long-lived process this would be a
/// leak, but vidi is single-shot so the OS reclaims the file on exit.
fn read_stdin_to_tempfile() -> vidi::error::Result<PathBuf> {
    use std::io::Read;
    let mut buf = Vec::new();
    std::io::stdin().read_to_end(&mut buf)?;

    let tmp = tempfile::Builder::new().suffix(".txt").tempfile()?;

    std::fs::write(tmp.path(), &buf)?;

    // Keep the file alive past the tempfile guard.
    let path = tmp
        .into_temp_path()
        .keep()
        .map_err(|e| vidi::error::VidiError::Io(e.error))?;

    Ok(path)
}
