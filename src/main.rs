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
        detector::{detect, FileKind},
        launcher::{launch_media, launch_media_inline},
        registry::resolve_viewer_choice,
        terminal::detect_capabilities,
        theme::{mapper::ThemeMapper, resolve::resolve_theme},
    };

    let config = load_config(cli.config.as_deref())?;

    let env_theme = std::env::var("VIDI_THEME").ok();
    let theme = resolve_theme(
        env_theme,
        cli.theme,
        Some(config.theme.clone()),
        &config.custom_themes,
    );
    let mapper = ThemeMapper::new(&theme);
    let caps = detect_capabilities();

    let file = resolve_file(&cli.file)?;
    let file = canonicalize_or_original(file);
    let kind = detect(&file)?;
    let lines = cli.lines.unwrap_or_else(|| caps.rows.max(24));

    // Audio and video bypass the viewer-choice mechanism.
    if matches!(kind, FileKind::Audio | FileKind::Video) {
        return if cli.inline {
            launch_media_inline(&file, lines)
        } else {
            launch_media(&file, kind, &mapper, &caps)
        };
    }

    let choice = resolve_viewer_choice(kind, cli.tool.as_deref(), &config);
    dispatch_choice(choice, kind, cli.inline, &file, &mapper, &caps, lines)
}

fn dispatch_choice(
    choice: vidi::registry::ViewerChoice,
    kind: vidi::detector::FileKind,
    inline: bool,
    file: &std::path::Path,
    mapper: &vidi::theme::mapper::ThemeMapper<'_>,
    caps: &vidi::terminal::TerminalCaps,
    lines: u16,
) -> vidi::error::Result<()> {
    use vidi::{registry::ViewerChoice, renderer::internal_render};

    match choice {
        ViewerChoice::ToolHard(ref name) => {
            let spec = require_named_spec(name)?;
            dispatch_tool(spec, kind, inline, file, mapper, caps, lines)
        }
        ViewerChoice::ToolSoft(ref name) => {
            let spec = prefer_named_spec(kind, name);
            dispatch_tool(spec, kind, inline, file, mapper, caps, lines)
        }
        ViewerChoice::Internal => {
            match internal_render(kind, file, caps, lines, inline) {
                Some(Ok(bytes)) => dispatch_rendered(bytes, inline, lines),
                Some(Err(e)) => Err(e),
                None => {
                    // No internal renderer for this sub-format; fall to registry.
                    let spec = best_tool_or_err(kind)?;
                    dispatch_tool(spec, kind, inline, file, mapper, caps, lines)
                }
            }
        }
        ViewerChoice::Default => {
            // Try internal renderer first; fall back to registry.
            if let Some(result) = internal_render(kind, file, caps, lines, inline) {
                return result.and_then(|b| dispatch_rendered(b, inline, lines));
            }
            let spec = best_tool_or_err(kind)?;
            dispatch_tool(spec, kind, inline, file, mapper, caps, lines)
        }
    }
}

fn dispatch_rendered(bytes: Vec<u8>, inline: bool, lines: u16) -> vidi::error::Result<()> {
    use vidi::launcher::{launch_internal_fullscreen, launch_internal_inline};
    if inline {
        launch_internal_inline(bytes, lines)
    } else {
        launch_internal_fullscreen(bytes)
    }
}

fn dispatch_tool(
    spec: &'static vidi::registry::ToolSpec,
    kind: vidi::detector::FileKind,
    inline: bool,
    file: &std::path::Path,
    mapper: &vidi::theme::mapper::ThemeMapper<'_>,
    caps: &vidi::terminal::TerminalCaps,
    lines: u16,
) -> vidi::error::Result<()> {
    use vidi::{
        detector::FileKind,
        launcher::{launch_fullscreen, launch_inline, launch_toggle},
        registry::TEXT_TOOLS,
    };

    if inline {
        if spec.supports_inline {
            launch_inline(spec, file, mapper, caps, lines)
        } else {
            let fallback = TEXT_TOOLS.last().unwrap();
            launch_inline(fallback, file, mapper, caps, lines)
        }
    } else {
        match kind {
            FileKind::LaTeX | FileKind::Typst => launch_toggle(file, mapper, caps),
            _ => launch_fullscreen(spec, file, mapper, caps),
        }
    }
}

/// Find any spec with `name` that is currently installed. Fails if not found.
fn require_named_spec(name: &str) -> vidi::error::Result<&'static vidi::registry::ToolSpec> {
    vidi::registry::REGISTRY
        .iter()
        .flat_map(|(_, specs)| specs.iter())
        .find(|s| s.name == name && which::which(s.binary).is_ok())
        .ok_or_else(|| vidi::error::VidiError::ToolNotFound {
            tool: name.to_string(),
        })
}

/// Prefer a named spec (if installed); otherwise fall back to the registry.
fn prefer_named_spec(
    kind: vidi::detector::FileKind,
    name: &str,
) -> &'static vidi::registry::ToolSpec {
    let named = vidi::registry::REGISTRY
        .iter()
        .flat_map(|(_, specs)| specs.iter())
        .find(|s| s.name == name && which::which(s.binary).is_ok());
    named
        .or_else(|| vidi::registry::best_tool(kind, None))
        .unwrap_or_else(|| vidi::registry::TEXT_TOOLS.last().unwrap())
}

/// Return the best available tool for `kind`, or error if none is installed.
fn best_tool_or_err(
    kind: vidi::detector::FileKind,
) -> vidi::error::Result<&'static vidi::registry::ToolSpec> {
    vidi::registry::best_tool(kind, None).ok_or_else(|| vidi::error::VidiError::NoViewerAvailable {
        kind: kind.to_string(),
    })
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

/// Follow symlinks via `canonicalize`; fall back to the original path on error.
fn canonicalize_or_original(path: PathBuf) -> PathBuf {
    std::fs::canonicalize(&path).unwrap_or(path)
}

/// Read all of stdin into a temporary file and return its path.
///
/// The file is persisted so that the viewer tool launched afterwards can read
/// it. On a long-lived process this would be a leak, but vidi is single-shot
/// so the OS reclaims the file on exit.
fn read_stdin_to_tempfile() -> vidi::error::Result<PathBuf> {
    use std::io::Read;
    let mut buf = Vec::new();
    std::io::stdin().read_to_end(&mut buf)?;

    let tmp = tempfile::Builder::new().suffix(".txt").tempfile()?;
    std::fs::write(tmp.path(), &buf)?;

    let path = tmp
        .into_temp_path()
        .keep()
        .map_err(|e| vidi::error::VidiError::Io(e.error))?;

    Ok(path)
}
