use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "veni", version, about = "TUI file manager")]
struct Cli {
    /// Directory to open
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Theme name
    #[arg(long)]
    theme: Option<String>,

    /// Disable neovim integration even if available
    #[arg(long)]
    no_nvim: bool,

    /// Path to config file
    #[arg(long)]
    config: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = veni::run(cli.path, cli.theme, cli.config.as_deref()) {
        eprintln!("veni: {e}");
        std::process::exit(1);
    }
}
