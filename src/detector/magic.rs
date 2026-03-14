use super::FileKind;
use crate::error::{Result, VidiError};
use std::path::Path;

/// Detect `FileKind` by reading the first bytes of the file and matching
/// against known magic byte signatures via the `infer` crate.
///
/// Returns `None` if the file is empty or the type is unrecognised.
pub fn detect_by_magic(path: &Path) -> Result<Option<FileKind>> {
    let bytes = read_header(path)?;
    if bytes.is_empty() {
        return Ok(None);
    }
    Ok(infer_kind(&bytes))
}

/// Read up to 512 bytes from the start of a file for magic detection.
fn read_header(path: &Path) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut f = std::fs::File::open(path).map_err(|e| VidiError::FileUnreadable {
        path: path.to_path_buf(),
        source: e,
    })?;
    let mut buf = vec![0u8; 512];
    let n = f.read(&mut buf).map_err(|e| VidiError::FileUnreadable {
        path: path.to_path_buf(),
        source: e,
    })?;
    buf.truncate(n);
    Ok(buf)
}

/// Map an `infer::Type` MIME string to a `FileKind`.
fn infer_kind(bytes: &[u8]) -> Option<FileKind> {
    let t = infer::get(bytes)?;
    let mime = t.mime_type();

    if mime.starts_with("image/") {
        return Some(FileKind::Image);
    }
    if mime.starts_with("video/") {
        return Some(FileKind::Video);
    }
    if mime.starts_with("audio/") {
        return Some(FileKind::Audio);
    }

    match mime {
        "application/pdf" => Some(FileKind::Pdf),
        "application/epub+zip" => Some(FileKind::Ebook),
        "application/zip"
        | "application/x-tar"
        | "application/gzip"
        | "application/x-bzip2"
        | "application/x-xz"
        | "application/x-7z-compressed"
        | "application/x-rar-compressed"
        | "application/zstd"
        | "application/x-lz4" => Some(FileKind::Archive),
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        | "application/vnd.openxmlformats-officedocument.presentationml.presentation" => {
            Some(FileKind::OfficeDocs)
        }
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        | "application/vnd.ms-excel" => Some(FileKind::Spreadsheet),
        "application/json" => Some(FileKind::Json),
        "text/xml" | "application/xml" => Some(FileKind::Text),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_bytes_returns_none() {
        assert_eq!(infer_kind(&[]), None);
    }

    #[test]
    fn png_magic_detects_image() {
        // PNG magic: \x89PNG\r\n\x1a\n
        let png_header = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR";
        assert_eq!(infer_kind(png_header), Some(FileKind::Image));
    }

    #[test]
    fn pdf_magic_detects_pdf() {
        let pdf_header = b"%PDF-1.4 some content";
        assert_eq!(infer_kind(pdf_header), Some(FileKind::Pdf));
    }

    #[test]
    fn zip_magic_detects_archive() {
        // ZIP magic: PK\x03\x04
        let zip_header = b"PK\x03\x04\x00\x00\x00\x00";
        assert_eq!(infer_kind(zip_header), Some(FileKind::Archive));
    }

    #[test]
    fn unknown_bytes_returns_none() {
        let unknown = b"\x00\x01\x02\x03\x04\x05";
        // infer may or may not recognise this; the function must not panic
        let _ = infer_kind(unknown);
    }
}
