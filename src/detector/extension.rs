use super::FileKind;
use std::path::Path;

/// Map of lowercase file extensions to `FileKind`.
/// Ordered from most-specific to most-general within each category.
fn extension_map(ext: &str) -> Option<FileKind> {
    match ext {
        // Markdown
        "md" | "markdown" | "mdown" | "mkd" | "mkdn" | "mdx" => Some(FileKind::Markdown),

        // LaTeX
        "tex" | "sty" | "cls" | "bib" | "bst" | "dtx" | "ins" => Some(FileKind::LaTeX),

        // Typst
        "typ" => Some(FileKind::Typst),

        // JSON
        "json" | "jsonc" | "json5" | "geojson" | "jsonl" | "ndjson" => Some(FileKind::Json),

        // YAML
        "yaml" | "yml" => Some(FileKind::Yaml),

        // TOML
        "toml" => Some(FileKind::Toml),

        // CSV / tabular (detect CSV before generic spreadsheet)
        "csv" | "tsv" | "psv" => Some(FileKind::Csv),

        // Spreadsheets
        "xlsx" | "xls" | "xlsm" | "ods" | "numbers" => Some(FileKind::Spreadsheet),

        // Office documents
        "docx" | "doc" | "odt" | "rtf" | "pages" | "pptx" | "ppt" | "odp" | "key" => {
            Some(FileKind::OfficeDocs)
        }

        // PDF
        "pdf" => Some(FileKind::Pdf),

        // Ebooks
        "epub" | "mobi" | "azw" | "azw3" | "fb2" | "djvu" | "djv" => Some(FileKind::Ebook),

        // Images
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "tif" | "webp" | "avif" | "heic"
        | "heif" | "ico" | "svg" | "svgz" | "psd" | "xcf" | "raw" | "cr2" | "nef" | "orf"
        | "arw" => Some(FileKind::Image),

        // Video
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg" | "ogv"
        | "ts" | "m2ts" | "mts" => Some(FileKind::Video),

        // Audio
        "mp3" | "flac" | "ogg" | "opus" | "wav" | "aac" | "m4a" | "wma" | "aiff" | "aif"
        | "ape" | "wv" | "mka" | "mid" | "midi" => Some(FileKind::Audio),

        // Archives
        "tar" | "gz" | "tgz" | "bz2" | "tbz2" | "xz" | "txz" | "zst" | "tzst" | "zip" | "7z"
        | "rar" | "lz4" | "lzma" | "lz" | "brotli" | "br" | "iso" | "deb" | "rpm" | "apk"
        | "jar" | "war" | "ear" | "whl" => Some(FileKind::Archive),

        // HTML documents
        "html" | "htm" | "xhtml" => Some(FileKind::Html),

        // Text (source code and plain text — catch-all for known text extensions)
        "rs" | "py" | "js" | "jsx" | "tsx" | "go" | "c" | "h" | "cpp" | "hpp" | "cc" | "cxx"
        | "cs" | "java" | "kt" | "swift" | "rb" | "php" | "sh" | "bash" | "zsh" | "fish"
        | "ps1" | "lua" | "r" | "jl" | "ex" | "exs" | "erl" | "hrl" | "hs" | "lhs" | "ml"
        | "mli" | "clj" | "cljs" | "cljc" | "scala" | "groovy" | "pl" | "pm" | "tcl" | "vim"
        | "el" | "lisp" | "scm" | "rkt" | "zig" | "nim" | "d" | "v" | "sv" | "vhd" | "vhdl"
        | "f" | "f90" | "f95" | "for" | "ada" | "adb" | "ads" | "dart" | "cr" | "txt" | "text"
        | "log" | "conf" | "cfg" | "ini" | "env" | "gitignore" | "gitattributes"
        | "editorconfig" | "makefile" | "dockerfile" | "vagrantfile" | "gemfile" | "rakefile"
        | "podfile" | "diff" | "patch" | "sql" | "graphql" | "gql" | "proto" | "thrift" | "tf"
        | "hcl" | "nix" | "cmake" | "xml" | "css" | "scss" | "sass" | "less" | "styl" | "vue"
        | "svelte" | "rst" | "adoc" | "asciidoc" | "org" | "wiki" | "textile" => {
            Some(FileKind::Text)
        }

        _ => None,
    }
}

/// Detect `FileKind` from the file extension alone. Returns `None` if the
/// extension is unknown or absent.
pub fn detect_by_extension(path: &Path) -> Option<FileKind> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    extension_map(&ext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn detect(name: &str) -> Option<FileKind> {
        detect_by_extension(Path::new(name))
    }

    #[test]
    fn detects_markdown() {
        assert_eq!(detect("README.md"), Some(FileKind::Markdown));
        assert_eq!(detect("notes.markdown"), Some(FileKind::Markdown));
    }

    #[test]
    fn detects_rust_source_as_text() {
        assert_eq!(detect("main.rs"), Some(FileKind::Text));
    }

    #[test]
    fn detects_json() {
        assert_eq!(detect("config.json"), Some(FileKind::Json));
        assert_eq!(detect("data.jsonl"), Some(FileKind::Json));
    }

    #[test]
    fn detects_yaml() {
        assert_eq!(detect("config.yaml"), Some(FileKind::Yaml));
        assert_eq!(detect("ci.yml"), Some(FileKind::Yaml));
    }

    #[test]
    fn detects_toml() {
        assert_eq!(detect("Cargo.toml"), Some(FileKind::Toml));
    }

    #[test]
    fn detects_csv() {
        assert_eq!(detect("data.csv"), Some(FileKind::Csv));
        assert_eq!(detect("data.tsv"), Some(FileKind::Csv));
    }

    #[test]
    fn detects_image_formats() {
        assert_eq!(detect("photo.jpg"), Some(FileKind::Image));
        assert_eq!(detect("icon.PNG"), Some(FileKind::Image));
        assert_eq!(detect("anim.gif"), Some(FileKind::Image));
        assert_eq!(detect("modern.webp"), Some(FileKind::Image));
    }

    #[test]
    fn detects_video() {
        assert_eq!(detect("movie.mkv"), Some(FileKind::Video));
        assert_eq!(detect("clip.mp4"), Some(FileKind::Video));
    }

    #[test]
    fn detects_audio() {
        assert_eq!(detect("song.flac"), Some(FileKind::Audio));
        assert_eq!(detect("track.mp3"), Some(FileKind::Audio));
    }

    #[test]
    fn detects_pdf() {
        assert_eq!(detect("report.pdf"), Some(FileKind::Pdf));
    }

    #[test]
    fn detects_ebook() {
        assert_eq!(detect("book.epub"), Some(FileKind::Ebook));
        assert_eq!(detect("novel.mobi"), Some(FileKind::Ebook));
        assert_eq!(detect("doc.djvu"), Some(FileKind::Ebook));
    }

    #[test]
    fn detects_office_docs() {
        assert_eq!(detect("report.docx"), Some(FileKind::OfficeDocs));
        assert_eq!(detect("slides.pptx"), Some(FileKind::OfficeDocs));
        assert_eq!(detect("doc.odt"), Some(FileKind::OfficeDocs));
    }

    #[test]
    fn detects_spreadsheet() {
        assert_eq!(detect("data.xlsx"), Some(FileKind::Spreadsheet));
        assert_eq!(detect("sheet.ods"), Some(FileKind::Spreadsheet));
    }

    #[test]
    fn detects_archive() {
        assert_eq!(detect("src.tar.gz"), Some(FileKind::Archive));
        assert_eq!(detect("backup.zip"), Some(FileKind::Archive));
        assert_eq!(detect("pkg.deb"), Some(FileKind::Archive));
    }

    #[test]
    fn detects_html() {
        assert_eq!(detect("page.html"), Some(FileKind::Html));
        assert_eq!(detect("page.htm"), Some(FileKind::Html));
        assert_eq!(detect("page.xhtml"), Some(FileKind::Html));
    }

    #[test]
    fn html_is_not_text() {
        assert_ne!(detect("index.html"), Some(FileKind::Text));
    }

    #[test]
    fn detects_latex() {
        assert_eq!(detect("thesis.tex"), Some(FileKind::LaTeX));
        assert_eq!(detect("style.sty"), Some(FileKind::LaTeX));
    }

    #[test]
    fn detects_typst() {
        assert_eq!(detect("doc.typ"), Some(FileKind::Typst));
    }

    #[test]
    fn returns_none_for_no_extension() {
        assert_eq!(detect("Makefile"), None);
        assert_eq!(detect("noext"), None);
    }

    #[test]
    fn returns_none_for_unknown_extension() {
        assert_eq!(detect("file.xyz123"), None);
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(detect("IMAGE.JPG"), Some(FileKind::Image));
        assert_eq!(detect("DOC.PDF"), Some(FileKind::Pdf));
    }
}
