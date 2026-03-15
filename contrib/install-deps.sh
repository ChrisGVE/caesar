#!/usr/bin/env bash
# Install optional viewer tools for vidi.
# Safe to run multiple times — skips already-installed tools.
set -euo pipefail

MINIMAL=false

usage() {
	cat <<EOF
Usage: $(basename "$0") [--minimal] [--help]

Install optional viewer tools used by vidi.

Options:
  --minimal  Install the most impactful subset only:
             bat, glow, chafa, mpv, pandoc, hexyl
  --help     Show this help
EOF
}

for arg in "$@"; do
	case "$arg" in
	--minimal) MINIMAL=true ;;
	-h | --help)
		usage
		exit 0
		;;
	*)
		echo "Unknown option: $arg" >&2
		usage >&2
		exit 1
		;;
	esac
done

# ── OS detection ───────────────────────────────────────────────────────────────
OS="unknown"
case "$(uname -s)" in
Darwin) OS="macos" ;;
Linux) OS="linux" ;;
esac

HAS_BREW=false
command -v brew &>/dev/null && HAS_BREW=true
HAS_APT=false
command -v apt-get &>/dev/null && HAS_APT=true
HAS_PIPX=false
command -v pipx &>/dev/null && HAS_PIPX=true
HAS_PIP=false
command -v pip3 &>/dev/null && HAS_PIP=true
$HAS_PIP || { command -v pip &>/dev/null && HAS_PIP=true; } || true

installed=()
skipped=()
manual=()

# ── Helpers ────────────────────────────────────────────────────────────────────

_brew() {
	local pkg="$1" binary="${2:-$1}"
	if command -v "$binary" &>/dev/null; then
		installed+=("$binary (already installed)")
		return
	fi
	if $HAS_BREW; then
		if brew install "$pkg" >/dev/null 2>&1; then
			installed+=("$binary (brew $pkg)")
		else
			skipped+=("$binary — brew install $pkg failed")
		fi
	else
		skipped+=("$binary — brew not available; run: brew install $pkg")
	fi
}

# On Linux prefer apt, fall back to brew; on macOS always brew.
_install() {
	local brew_pkg="$1" apt_pkg="${2:-$1}" binary="${3:-$1}"
	if command -v "$binary" &>/dev/null; then
		installed+=("$binary (already installed)")
		return
	fi
	if [[ "$OS" == "linux" ]] && $HAS_APT; then
		if sudo apt-get install -y "$apt_pkg" >/dev/null 2>&1; then
			installed+=("$binary (apt $apt_pkg)")
			return
		fi
	fi
	_brew "$brew_pkg" "$binary"
}

_epy() {
	if command -v epy &>/dev/null; then
		installed+=("epy (already installed)")
		return
	fi
	if $HAS_PIPX; then
		if pipx install epy-reader >/dev/null 2>&1; then
			installed+=("epy (pipx epy-reader)")
			return
		fi
	fi
	if $HAS_PIP; then
		if pip3 install epy-reader >/dev/null 2>&1 ||
			pip install epy-reader >/dev/null 2>&1; then
			installed+=("epy (pip epy-reader)")
			return
		fi
	fi
	manual+=("epy: pip install epy-reader  (or pipx install epy-reader)")
}

# ── Minimal install ────────────────────────────────────────────────────────────
if $MINIMAL; then
	echo "==> vidi minimal install"
	_install bat bat bat          # text / code
	_install glow glow glow       # markdown
	_install chafa chafa chafa    # images
	_install mpv mpv mpv          # video / audio
	_install pandoc pandoc pandoc # ebooks / office fallback
	_install hexyl hexyl hexyl    # binary / hex
else

	# ── Full install ───────────────────────────────────────────────────────────────
	echo "==> vidi dependency installer  (OS: $OS)"

	echo ""
	echo "── Syntax highlighting ──────────────────────────────────────────"
	_install bat bat bat
	_install highlight highlight highlight

	echo "── Markdown ─────────────────────────────────────────────────────"
	_install glow glow glow
	_install mdcat mdcat mdcat

	echo "── Images ───────────────────────────────────────────────────────"
	_install chafa chafa chafa
	_install viu viu viu
	_install timg timg timg

	echo "── Video / audio ────────────────────────────────────────────────"
	_install ffmpeg ffmpeg ffprobe # ffprobe ships inside ffmpeg
	_install mpv mpv mpv

	echo "── PDF ──────────────────────────────────────────────────────────"
	if [[ "$OS" == "linux" ]]; then
		_install zathura zathura zathura # Linux only
	else
		echo "    zathura: not available on macOS; mutool and pdftotext used instead"
	fi
	_install mupdf-tools mupdf-tools mutool
	_install poppler poppler-utils pdftotext

	echo "── Ebooks ───────────────────────────────────────────────────────"
	_epy
	_install pandoc pandoc pandoc

	echo "── Office documents ─────────────────────────────────────────────"
	_install doxx doxx doxx

	echo "── Spreadsheets ─────────────────────────────────────────────────"
	_install visidata visidata vd
	_install sc-im sc-im sc-im

	echo "── CSV / tabular data ───────────────────────────────────────────"
	_install csvlens csvlens csvlens
	_install tidy-viewer tidy-viewer tv # binary is 'tv'
	_install miller miller mlr          # binary is 'mlr'

	echo "── JSON / YAML / TOML ───────────────────────────────────────────"
	_install jless jless jless
	_install jq jq jq
	_install yq yq yq
	_install taplo taplo taplo

	echo "── Archives ─────────────────────────────────────────────────────"
	_install ouch ouch ouch

	echo "── Binary / hex ─────────────────────────────────────────────────"
	_install hexyl hexyl hexyl

	echo "── LaTeX / Typst ────────────────────────────────────────────────"
	_install tectonic tectonic tectonic
	_install typst typst typst

fi # end full install

# ── Summary ────────────────────────────────────────────────────────────────────
echo ""
echo "==> Summary"
if [[ ${#installed[@]} -gt 0 ]]; then
	echo ""
	printf "Installed (%d):\n" "${#installed[@]}"
	for item in "${installed[@]}"; do printf "  ✓ %s\n" "$item"; done
fi
if [[ ${#skipped[@]} -gt 0 ]]; then
	echo ""
	printf "Skipped (%d):\n" "${#skipped[@]}"
	for item in "${skipped[@]}"; do printf "  ✗ %s\n" "$item"; done
fi
if [[ ${#manual[@]} -gt 0 ]]; then
	echo ""
	printf "Requires manual install (%d):\n" "${#manual[@]}"
	for item in "${manual[@]}"; do printf "  → %s\n" "$item"; done
fi
if [[ ${#skipped[@]} -eq 0 && ${#manual[@]} -eq 0 ]]; then
	echo "  All tools installed."
fi
