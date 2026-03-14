use super::tools::ToolSpec;

pub static TEXT_TOOLS: &[ToolSpec] = &[
    ToolSpec {
        name: "bat",
        binary: "bat",
        fullscreen_args: &["--paging=always", "--theme={theme}"],
        inline_args: &[
            "--paging=never",
            "--plain",
            "--terminal-width={cols}",
            "--line-range=:{lines}",
            "--theme={theme}",
        ],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: true,
    },
    ToolSpec {
        name: "highlight",
        binary: "highlight",
        fullscreen_args: &["--out-format=ansi", "--force"],
        inline_args: &["--out-format=ansi", "--force"],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: false,
    },
    ToolSpec {
        name: "cat",
        binary: "cat",
        fullscreen_args: &[],
        inline_args: &[],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: false,
    },
];

pub static MARKDOWN_TOOLS: &[ToolSpec] = &[
    ToolSpec {
        name: "glow",
        binary: "glow",
        fullscreen_args: &["-p"],
        inline_args: &["-w={cols}"],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: true,
    },
    ToolSpec {
        name: "mdcat",
        binary: "mdcat",
        fullscreen_args: &["-p"],
        inline_args: &["--columns={cols}"],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: false,
    },
    ToolSpec {
        name: "bat",
        binary: "bat",
        fullscreen_args: &["--paging=always", "--language=markdown", "--theme={theme}"],
        inline_args: &[
            "--paging=never",
            "--plain",
            "--language=markdown",
            "--terminal-width={cols}",
            "--line-range=:{lines}",
            "--theme={theme}",
        ],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: true,
    },
];

pub static IMAGE_TOOLS: &[ToolSpec] = &[
    ToolSpec {
        name: "chafa",
        binary: "chafa",
        fullscreen_args: &["--size={cols}x{rows}"],
        inline_args: &["--size={cols}x{lines}"],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: false,
    },
    ToolSpec {
        name: "viu",
        binary: "viu",
        fullscreen_args: &["-w={cols}"],
        inline_args: &["-w={cols}", "-h={lines}"],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: false,
    },
    ToolSpec {
        name: "timg",
        binary: "timg",
        fullscreen_args: &["-g{cols}x{rows}"],
        inline_args: &["-g{cols}x{lines}"],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: false,
    },
];

pub static VIDEO_TOOLS: &[ToolSpec] = &[
    ToolSpec {
        name: "timg",
        binary: "timg",
        fullscreen_args: &["-g{cols}x{rows}", "--frames=1"],
        inline_args: &["-g{cols}x{lines}", "--frames=1"],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: false,
    },
    ToolSpec {
        name: "mpv",
        binary: "mpv",
        fullscreen_args: &["--vo=kitty", "--really-quiet"],
        inline_args: &[],
        supports_inline: false,
        supports_fullscreen: true,
        supports_theming: false,
    },
];

pub static AUDIO_TOOLS: &[ToolSpec] = &[ToolSpec {
    name: "ffprobe",
    binary: "ffprobe",
    fullscreen_args: &[
        "-v",
        "quiet",
        "-print_format",
        "json",
        "-show_format",
        "-show_streams",
    ],
    inline_args: &[
        "-v",
        "quiet",
        "-print_format",
        "json",
        "-show_format",
        "-show_streams",
    ],
    supports_inline: true,
    supports_fullscreen: true,
    supports_theming: false,
}];

pub static PDF_TOOLS: &[ToolSpec] = &[
    ToolSpec {
        name: "zathura",
        binary: "zathura",
        fullscreen_args: &[],
        inline_args: &[],
        supports_inline: false,
        supports_fullscreen: true,
        supports_theming: true,
    },
    ToolSpec {
        name: "mutool",
        binary: "mutool",
        fullscreen_args: &["draw", "-F", "png", "-o", "-"],
        inline_args: &["draw", "-F", "png", "-o", "-"],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: false,
    },
    ToolSpec {
        name: "pdftotext",
        binary: "pdftotext",
        fullscreen_args: &["-layout", "-", "-"],
        inline_args: &["-layout", "-f", "1", "-l", "1", "-", "-"],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: false,
    },
];

pub static EBOOK_TOOLS: &[ToolSpec] = &[
    ToolSpec {
        name: "epy",
        binary: "epy",
        fullscreen_args: &[],
        inline_args: &[],
        supports_inline: false,
        supports_fullscreen: true,
        supports_theming: false,
    },
    ToolSpec {
        name: "pandoc",
        binary: "pandoc",
        fullscreen_args: &["-t", "plain"],
        inline_args: &["-t", "plain"],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: false,
    },
];

pub static OFFICE_TOOLS: &[ToolSpec] = &[
    ToolSpec {
        name: "doxx",
        binary: "doxx",
        fullscreen_args: &[],
        inline_args: &[],
        supports_inline: false,
        supports_fullscreen: true,
        supports_theming: false,
    },
    ToolSpec {
        name: "pandoc",
        binary: "pandoc",
        fullscreen_args: &["-t", "markdown"],
        inline_args: &["-t", "plain"],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: false,
    },
];

pub static SPREADSHEET_TOOLS: &[ToolSpec] = &[
    ToolSpec {
        name: "visidata",
        binary: "vd",
        fullscreen_args: &[],
        inline_args: &[],
        supports_inline: false,
        supports_fullscreen: true,
        supports_theming: false,
    },
    ToolSpec {
        name: "sc-im",
        binary: "sc-im",
        fullscreen_args: &[],
        inline_args: &[],
        supports_inline: false,
        supports_fullscreen: true,
        supports_theming: true,
    },
];

pub static CSV_TOOLS: &[ToolSpec] = &[
    ToolSpec {
        name: "csvlens",
        binary: "csvlens",
        fullscreen_args: &[],
        inline_args: &[],
        supports_inline: false,
        supports_fullscreen: true,
        supports_theming: false,
    },
    ToolSpec {
        name: "tidy-viewer",
        binary: "tv",
        fullscreen_args: &[],
        inline_args: &["-n={lines}"],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: false,
    },
    ToolSpec {
        name: "miller",
        binary: "mlr",
        fullscreen_args: &["--csv", "--opprint", "cat"],
        inline_args: &["--csv", "--opprint", "cat"],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: false,
    },
];

pub static LATEX_TOOLS: &[ToolSpec] = &[
    ToolSpec {
        name: "tectonic",
        binary: "tectonic",
        fullscreen_args: &[],
        inline_args: &[],
        supports_inline: false,
        supports_fullscreen: true,
        supports_theming: false,
    },
    ToolSpec {
        name: "bat",
        binary: "bat",
        fullscreen_args: &["--paging=always", "--language=latex", "--theme={theme}"],
        inline_args: &[
            "--paging=never",
            "--plain",
            "--language=latex",
            "--terminal-width={cols}",
            "--line-range=:{lines}",
            "--theme={theme}",
        ],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: true,
    },
];

pub static TYPST_TOOLS: &[ToolSpec] = &[
    ToolSpec {
        name: "typst",
        binary: "typst",
        fullscreen_args: &["compile"],
        inline_args: &[],
        supports_inline: false,
        supports_fullscreen: true,
        supports_theming: false,
    },
    ToolSpec {
        name: "bat",
        binary: "bat",
        fullscreen_args: &["--paging=always", "--language=typst", "--theme={theme}"],
        inline_args: &[
            "--paging=never",
            "--plain",
            "--language=typst",
            "--terminal-width={cols}",
            "--line-range=:{lines}",
            "--theme={theme}",
        ],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: true,
    },
];

pub static JSON_TOOLS: &[ToolSpec] = &[
    ToolSpec {
        name: "jless",
        binary: "jless",
        fullscreen_args: &[],
        inline_args: &[],
        supports_inline: false,
        supports_fullscreen: true,
        supports_theming: false,
    },
    ToolSpec {
        name: "jq",
        binary: "jq",
        fullscreen_args: &["-C", "."],
        inline_args: &["-C", "."],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: false,
    },
    ToolSpec {
        name: "bat",
        binary: "bat",
        fullscreen_args: &["--paging=always", "--language=json", "--theme={theme}"],
        inline_args: &[
            "--paging=never",
            "--plain",
            "--language=json",
            "--terminal-width={cols}",
            "--line-range=:{lines}",
            "--theme={theme}",
        ],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: true,
    },
];

pub static YAML_TOOLS: &[ToolSpec] = &[
    ToolSpec {
        name: "yq",
        binary: "yq",
        fullscreen_args: &["."],
        inline_args: &["."],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: false,
    },
    ToolSpec {
        name: "bat",
        binary: "bat",
        fullscreen_args: &["--paging=always", "--language=yaml", "--theme={theme}"],
        inline_args: &[
            "--paging=never",
            "--plain",
            "--language=yaml",
            "--terminal-width={cols}",
            "--line-range=:{lines}",
            "--theme={theme}",
        ],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: true,
    },
];

pub static TOML_TOOLS: &[ToolSpec] = &[
    ToolSpec {
        name: "taplo",
        binary: "taplo",
        fullscreen_args: &["fmt", "--stdin-filepath=file.toml", "-"],
        inline_args: &["fmt", "--stdin-filepath=file.toml", "-"],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: false,
    },
    ToolSpec {
        name: "bat",
        binary: "bat",
        fullscreen_args: &["--paging=always", "--language=toml", "--theme={theme}"],
        inline_args: &[
            "--paging=never",
            "--plain",
            "--language=toml",
            "--terminal-width={cols}",
            "--line-range=:{lines}",
            "--theme={theme}",
        ],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: true,
    },
];

pub static ARCHIVE_TOOLS: &[ToolSpec] = &[
    ToolSpec {
        name: "ouch",
        binary: "ouch",
        fullscreen_args: &["list"],
        inline_args: &["list"],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: false,
    },
    ToolSpec {
        name: "bsdtar",
        binary: "bsdtar",
        fullscreen_args: &["-tv", "-f"],
        inline_args: &["-tv", "-f"],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: false,
    },
];

pub static BINARY_TOOLS: &[ToolSpec] = &[
    ToolSpec {
        name: "hexyl",
        binary: "hexyl",
        fullscreen_args: &[],
        inline_args: &["--length={bytes}"],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: false,
    },
    ToolSpec {
        name: "xxd",
        binary: "xxd",
        fullscreen_args: &[],
        inline_args: &["-l", "{bytes}"],
        supports_inline: true,
        supports_fullscreen: true,
        supports_theming: false,
    },
];
