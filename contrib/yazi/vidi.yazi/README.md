# vidi.yazi

A [yazi](https://github.com/sxyazi/yazi) plugin that uses [vidi](https://github.com/ChrisGVE/vidi) as a universal file previewer and opener.

- **Previewer**: renders any file type inline in the yazi preview pane via `vidi --inline`.
- **Opener**: launches vidi full-screen when you open a file from yazi.

## Requirements

- yazi ≥ 0.4
- vidi installed and on your `$PATH`

## Installation

### 1. Install vidi

Once published to crates.io:

```sh
cargo install vidi
```

Until then, build from source:

```sh
git clone https://github.com/ChrisGVE/vidi
cd vidi
cargo install --path .
```

### 2. Install the plugin

Copy the `vidi.yazi` directory to yazi's plugin folder:

```sh
# macOS / Linux
cp -r vidi.yazi ~/.config/yazi/plugins/

# Or, if you cloned the vidi repo:
cp -r contrib/yazi/vidi.yazi ~/.config/yazi/plugins/
```

### 3. Configure the previewer

Add to `~/.config/yazi/yazi.toml`:

```toml
[plugin]
prepend_previewers = [
  { name = "*", run = "vidi" },
]
```

This places vidi first in the previewer chain so it handles every file type.
Yazi's built-in previewers remain as fallbacks for anything vidi does not cover.

### 4. Configure the opener (optional)

To open files with vidi in full-screen mode when pressing Enter in yazi,
add to `~/.config/yazi/yazi.toml`:

```toml
[opener]
vidi = [
  { run = 'vidi "$@"', block = true, for = "unix" },
]

[open]
prepend_rules = [
  { name = "*", use = "vidi" },
]
```

`block = true` keeps yazi suspended while vidi runs, restoring the yazi UI
cleanly when you quit vidi.

## Theme detection

The plugin attempts to map the active yazi Catppuccin flavor to the matching
vidi theme.  For other themes, set `VIDI_THEME` in your shell environment:

```sh
export VIDI_THEME=catppuccin-mocha
```

Supported values mirror vidi's built-in theme names (e.g. `catppuccin-latte`,
`catppuccin-frappe`, `catppuccin-macchiato`, `catppuccin-mocha`).
