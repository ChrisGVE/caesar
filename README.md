# vidi

Opens any file in the terminal. Detects the type, picks the best installed viewer, and renders it — full-screen or inline.

[![CI](https://github.com/ChrisGVE/vidi/actions/workflows/ci.yml/badge.svg)](https://github.com/ChrisGVE/vidi/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/vidi)](https://crates.io/crates/vidi)
[![docs.rs](https://img.shields.io/docsrs/vidi)](https://docs.rs/vidi)

## Installation

```sh
cargo install vidi
```

Rust 1.82 or later is required.

## Usage

```sh
vidi <file>              # full-screen view (replaces the shell)
vidi --inline <file>     # write output to stdout, constrained to terminal height
vidi --inline --lines 30 <file>   # constrained to 30 lines
vidi --theme catppuccin-latte <file>
vidi --tool bat <file>   # force a specific viewer
vidi -                   # read from stdin
```

## Supported formats

| Category | Example extensions | Preferred tool | Fallback |
|---|---|---|---|
| Source code / text | `.rs` `.py` `.js` `.sh` `.txt` | bat | highlight, cat |
| Markdown | `.md` `.markdown` | glow | mdcat, bat |
| Images | `.jpg` `.png` `.gif` `.webp` `.svg` | chafa | viu, timg |
| Video | `.mp4` `.mkv` `.mov` | timg | mpv |
| Audio | `.mp3` `.flac` `.ogg` `.wav` | ffprobe (metadata) | — |
| PDF | `.pdf` | zathura | mutool, pdftotext |
| Ebooks | `.epub` `.mobi` `.djvu` | epy | pandoc |
| Office documents | `.docx` `.odt` `.pptx` | doxx | pandoc |
| Spreadsheets | `.xlsx` `.ods` `.numbers` | visidata | sc-im |
| CSV / tabular | `.csv` `.tsv` | csvlens | tidy-viewer, miller |
| JSON | `.json` `.jsonl` | jless | jq, bat |
| YAML | `.yaml` `.yml` | yq | bat |
| TOML | `.toml` | taplo | bat |
| LaTeX | `.tex` `.sty` `.cls` | tectonic + bat (toggle) | bat |
| Typst | `.typ` | typst + bat (toggle) | bat |
| Archives | `.tar.gz` `.zip` `.7z` | ouch | bsdtar |
| Binary / hex | any unrecognised binary | hexyl | xxd |

None of the preferred tools are required. vidi probes what is installed and selects the best available option. `cat` and `xxd` are the universal fallbacks for text and binary respectively.

## Theming

The default theme is `catppuccin-mocha`. vidi translates the active theme into tool-specific flags before launching each viewer, so bat, glow, and other tools all reflect the same palette.

**Built-in themes**

- `catppuccin-mocha` (default, dark)
- `catppuccin-latte` (light)
- `catppuccin-frappe` (dark)
- `catppuccin-macchiato` (dark)

**Selecting a theme**

In priority order, highest first:

1. `VIDI_THEME` environment variable
2. `--theme <name>` CLI flag
3. `theme` key in `~/.config/vidi/config.toml`
4. Built-in default (`catppuccin-mocha`)

The `VIDI_THEME` variable is intended for host tools (e.g. yazi) that want vidi's output to match their own active theme.

**Custom themes**

Define custom themes in `config.toml` as TOML tables with `name`, `bg`, `fg`, `cursor`, a 16-entry `ansi` array, and a 4-entry `accents` array. Each color is an RGB table `{ r = 0, g = 0, b = 0 }`.

## Configuration

Config file location: `~/.config/vidi/config.toml` (respects `XDG_CONFIG_HOME`).

All keys are optional. Missing file or missing keys fall back to defaults.

```toml
# Active theme name. Built-in: catppuccin-mocha, catppuccin-latte,
# catppuccin-frappe, catppuccin-macchiato.
theme = "catppuccin-mocha"

# Per-tool argument overrides. The key is the tool name as listed in
# the supported formats table above.
[tool_overrides]
bat = ["--paging=always", "--style=numbers,changes"]

# Additional directories to search for viewer binaries beyond PATH.
extra_search_paths = ["/opt/homebrew/bin"]

# Custom theme definitions (optional).
# [[custom_themes]]
# name = "my-theme"
# bg   = { r = 30,  g = 30,  b = 46  }
# fg   = { r = 205, g = 214, b = 244 }
# cursor = { r = 245, g = 224, b = 220 }
# ansi = [
#   { r = 69,  g = 71,  b = 90  },   # 0  black
#   { r = 243, g = 139, b = 168 },   # 1  red
#   { r = 166, g = 227, b = 161 },   # 2  green
#   { r = 249, g = 226, b = 175 },   # 3  yellow
#   { r = 137, g = 180, b = 250 },   # 4  blue
#   { r = 245, g = 194, b = 231 },   # 5  magenta
#   { r = 148, g = 226, b = 213 },   # 6  cyan
#   { r = 186, g = 194, b = 222 },   # 7  white
#   { r = 88,  g = 91,  b = 112 },   # 8  bright black
#   { r = 243, g = 139, b = 168 },   # 9  bright red
#   { r = 166, g = 227, b = 161 },   # 10 bright green
#   { r = 249, g = 226, b = 175 },   # 11 bright yellow
#   { r = 137, g = 180, b = 250 },   # 12 bright blue
#   { r = 245, g = 194, b = 231 },   # 13 bright magenta
#   { r = 148, g = 226, b = 213 },   # 14 bright cyan
#   { r = 205, g = 214, b = 244 },   # 15 bright white
# ]
# accents = [
#   { r = 203, g = 166, b = 247 },   # mauve
#   { r = 250, g = 179, b = 135 },   # peach
#   { r = 137, g = 220, b = 235 },   # sky
#   { r = 116, g = 199, b = 236 },   # sapphire
# ]
```

## Yazi integration

vidi can serve as both a **previewer** and a **full-screen opener** inside [yazi](https://github.com/sxyazi/yazi).

Configuration files are in `contrib/yazi/`. See [`contrib/yazi/vidi.yazi/README.md`](contrib/yazi/vidi.yazi/README.md) for full setup instructions.

Yazi already handles images, plain text, source code, PDF thumbnails, and video thumbnails natively. vidi adds previews for formats yazi does not cover: ebooks, office documents, spreadsheets, CSV, audio metadata, LaTeX/Typst sources, and archives.

## LaTeX and Typst

For `.tex` and `.typ` files, vidi opens a toggle view:

- The initial view shows the source with syntax highlighting via `bat`.
- Press `r` to compile and display the rendered PDF (requires `tectonic` for LaTeX or `typst compile` for Typst, plus `mutool` for PDF-to-image conversion).
- Press `s` to return to the source view.
- Press `q` to quit.

Compilation runs asynchronously; the source view is shown immediately while rendering is in progress.

## Requirements

No tool is strictly required. vidi degrades gracefully:

- **Text and source code**: `cat` is the universal fallback (POSIX standard).
- **Binary files**: `xxd` is the universal fallback (ships with vim).

For richer output, install any combination of the tools below.

### macOS (Homebrew)

```sh
brew install bat glow mdcat highlight chafa viu timg ffmpeg mpv \
             mupdf-tools poppler pandoc doxx \
             visidata sc-im csvlens tidy-viewer miller \
             jless jq yq taplo ouch hexyl \
             tectonic typst
```

`epy` is not in Homebrew; install it with `pip install epy-reader`.

`zathura` is not packaged for macOS via Homebrew; `mutool` (from `mupdf-tools`) is used as the PDF fallback instead.

### Linux (apt + Homebrew)

Packages available in standard apt repositories:

```sh
apt install bat chafa timg ffmpeg mpv zathura mupdf-tools poppler-utils \
            pandoc visidata sc-im miller jq hexyl highlight
```

For tools not in standard apt repositories, use [Homebrew on Linux](https://brew.sh):

```sh
brew install glow mdcat viu doxx csvlens tidy-viewer jless yq taplo ouch tectonic typst
```

`epy` is not in any package manager; install it with `pip install epy-reader`.

On older Ubuntu/Debian, `bat` is installed as `batcat`. Create an alias:
`mkdir -p ~/.local/bin && ln -sf "$(which batcat)" ~/.local/bin/bat`

### Per-tool reference

| Tool | Binary | macOS | Linux | Notes |
|---|---|---|---|---|
| bat | `bat` | `brew install bat` | `apt install bat` | Text/code viewer with syntax highlighting |
| highlight | `highlight` | `brew install highlight` | `apt install highlight` | Fallback syntax highlighter |
| glow | `glow` | `brew install glow` | `brew install glow` | Markdown renderer |
| mdcat | `mdcat` | `brew install mdcat` | `brew install mdcat` | Markdown fallback |
| chafa | `chafa` | `brew install chafa` | `apt install chafa` | Image renderer (all terminals) |
| viu | `viu` | `brew install viu` | `brew install viu` | Image viewer (Kitty/iTerm2) |
| timg | `timg` | `brew install timg` | `apt install timg` | Image and video thumbnails |
| ffmpeg | `ffprobe` | `brew install ffmpeg` | `apt install ffmpeg` | Audio metadata; includes `ffprobe` |
| mpv | `mpv` | `brew install mpv` | `apt install mpv` | Video and audio playback |
| zathura | `zathura` | — | `apt install zathura` | PDF viewer; macOS: not available |
| mupdf-tools | `mutool` | `brew install mupdf-tools` | `apt install mupdf-tools` | PDF rendering fallback |
| poppler | `pdftotext` | `brew install poppler` | `apt install poppler-utils` | PDF text extraction fallback |
| epy | `epy` | `pip install epy-reader` | `pip install epy-reader` | Ebook reader (epub/mobi) |
| pandoc | `pandoc` | `brew install pandoc` | `apt install pandoc` | Ebook and office doc fallback |
| doxx | `doxx` | `brew install doxx` | `brew install doxx` | Office document viewer |
| visidata | `vd` | `brew install visidata` | `apt install visidata` | Spreadsheet viewer |
| sc-im | `sc-im` | `brew install sc-im` | `apt install sc-im` | Spreadsheet fallback |
| csvlens | `csvlens` | `brew install csvlens` | `brew install csvlens` | CSV viewer |
| tidy-viewer | `tv` | `brew install tidy-viewer` | `brew install tidy-viewer` | CSV fallback |
| miller | `mlr` | `brew install miller` | `apt install miller` | CSV/tabular data fallback |
| jless | `jless` | `brew install jless` | `brew install jless` | JSON viewer |
| jq | `jq` | `brew install jq` | `apt install jq` | JSON fallback |
| yq | `yq` | `brew install yq` | `brew install yq` | YAML viewer |
| taplo | `taplo` | `brew install taplo` | `brew install taplo` | TOML formatter/viewer |
| ouch | `ouch` | `brew install ouch` | `brew install ouch` | Archive listing |
| hexyl | `hexyl` | `brew install hexyl` | `apt install hexyl` | Hex viewer |
| tectonic | `tectonic` | `brew install tectonic` | `brew install tectonic` | LaTeX compilation |
| typst | `typst` | `brew install typst` | `brew install typst` | Typst compilation |

## License

MIT
