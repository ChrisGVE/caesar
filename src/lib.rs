//! `vidi` — universal terminal file viewer.
//!
//! Detects the file type, selects the best installed viewer for that type,
//! applies a consistent colour theme, and renders output in the terminal.
//!
//! # Modes
//!
//! - **Full-screen** (default): replaces the process with the selected tool.
//! - **Inline** (`--inline`): captures and truncates output to stdout.
//! - **Toggle** (LaTeX/Typst): cycles between source and rendered views.
//!
//! # Crate structure
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`detector`] | File-type detection (extension → magic → content) |
//! | [`registry`] | Tool registry and availability probing |
//! | [`terminal`] | Terminal capabilities and graphics protocol detection |
//! | [`theme`] | Theme definitions, resolution, and per-tool mapping |
//! | [`config`] | Config file loading (`~/.config/vidi/config.toml`) |
//! | [`launcher`] | Full-screen, inline, toggle, and media launchers |
//! | [`renderer`] | Internal ANSI renderers (HTML, EPUB) |
//! | [`error`] | Unified error type |

pub mod config;
pub mod detector;
pub mod error;
pub mod launcher;
pub mod registry;
pub mod renderer;
pub mod terminal;
pub mod theme;
