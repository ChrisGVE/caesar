pub mod app;
pub mod config;
pub mod error;
pub mod input;
pub mod ops;
pub mod pane;
pub mod ui;

pub use error::{Result, VeniError};

use app::App;
use caesar_common::terminal::detect_capabilities;
use config::load_config;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Entry point for the veni file manager.
pub fn run(path: PathBuf, theme: Option<String>, config_path: Option<&Path>) -> Result<()> {
    let path = std::fs::canonicalize(&path).unwrap_or(path);
    if !path.is_dir() {
        return Err(VeniError::NotADirectory(path));
    }

    // Load configuration, applying the optional explicit override.
    let mut cfg = load_config(config_path)?;
    // CLI --theme flag wins over file config.
    if let Some(t) = theme {
        cfg.theme = t;
    }

    let caps = detect_capabilities();
    let mut app = App::new(path, caps, cfg);
    app.load_dir()?;

    // Set up the terminal in raw/alternate-screen mode.
    enable_raw_mode().map_err(|e| VeniError::Terminal(e.to_string()))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| VeniError::Terminal(e.to_string()))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| VeniError::Terminal(e.to_string()))?;

    // Install a panic hook that restores the terminal before printing the
    // panic message so the user's shell is not left in a broken state.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    let result = run_app(&mut terminal, &mut app);

    // Always restore terminal — even on error.
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    terminal.show_cursor().ok();

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal
            .draw(|f| ui::draw(f, app))
            .map_err(|e| VeniError::Terminal(e.to_string()))?;

        if event::poll(Duration::from_millis(100))
            .map_err(|e| VeniError::Terminal(e.to_string()))?
        {
            if let Event::Key(key) =
                event::read().map_err(|e| VeniError::Terminal(e.to_string()))?
            {
                app.handle_key(key);
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
