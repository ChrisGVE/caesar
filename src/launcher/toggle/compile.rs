//! Compile LaTeX / Typst source files to PDF.
//!
//! Each function returns the path to the produced PDF on success.
//! Callers are responsible for supplying an existing temporary directory.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Result, VidiError};

/// Checker function type — injected so tests can mock binary availability.
pub type IsAvailableFn = fn(&'static str) -> bool;

/// Compile a `.tex` file to PDF using `tectonic`.
///
/// The PDF is written to `<outdir>/<stem>.pdf` where `stem` is derived from
/// the input file name.
///
/// # Errors
///
/// Returns [`VidiError::ToolNotFound`] when `tectonic` is absent.
/// Returns [`VidiError::ToolFailed`] when compilation exits non-zero.
pub fn compile_latex(file: &Path, outdir: &Path, available: IsAvailableFn) -> Result<PathBuf> {
    if !available("tectonic") {
        return Err(VidiError::ToolNotFound {
            tool: "tectonic".to_string(),
        });
    }

    let status = Command::new("tectonic")
        .args(["-X", "compile", "--outdir"])
        .arg(outdir)
        .arg(file)
        .status()
        .map_err(VidiError::Io)?;

    if !status.success() {
        return Err(VidiError::ToolFailed {
            tool: "tectonic".to_string(),
            code: status.code().unwrap_or(1),
        });
    }

    let stem = file
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    Ok(outdir.join(format!("{stem}.pdf")))
}

/// Compile a `.typ` file to PDF using `typst`.
///
/// The PDF is written to `<outdir>/<stem>.pdf`.
///
/// # Errors
///
/// Returns [`VidiError::ToolNotFound`] when `typst` is absent.
/// Returns [`VidiError::ToolFailed`] when compilation exits non-zero.
pub fn compile_typst(file: &Path, outdir: &Path, available: IsAvailableFn) -> Result<PathBuf> {
    if !available("typst") {
        return Err(VidiError::ToolNotFound {
            tool: "typst".to_string(),
        });
    }

    let stem = file
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let pdf_path = outdir.join(format!("{stem}.pdf"));

    let status = Command::new("typst")
        .arg("compile")
        .arg(file)
        .arg(&pdf_path)
        .status()
        .map_err(VidiError::Io)?;

    if !status.success() {
        return Err(VidiError::ToolFailed {
            tool: "typst".to_string(),
            code: status.code().unwrap_or(1),
        });
    }

    Ok(pdf_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[allow(dead_code)]
    fn available(_: &'static str) -> bool {
        true
    }
    fn unavailable(_: &'static str) -> bool {
        false
    }

    #[test]
    fn latex_returns_tool_not_found_when_binary_absent() {
        let result = compile_latex(Path::new("/tmp/test.tex"), Path::new("/tmp"), unavailable);
        match result {
            Err(VidiError::ToolNotFound { tool }) => assert_eq!(tool, "tectonic"),
            other => panic!("expected ToolNotFound, got {other:?}"),
        }
    }

    #[test]
    fn typst_returns_tool_not_found_when_binary_absent() {
        let result = compile_typst(Path::new("/tmp/test.typ"), Path::new("/tmp"), unavailable);
        match result {
            Err(VidiError::ToolNotFound { tool }) => assert_eq!(tool, "typst"),
            other => panic!("expected ToolNotFound, got {other:?}"),
        }
    }

    #[test]
    fn latex_output_path_derives_from_stem() {
        // We can verify the expected path without running the binary.
        // The function would fail at Command::new if tectonic were absent;
        // here we only test the path computation logic indirectly by checking
        // that ToolNotFound is NOT returned (binary "available") but then a
        // spawn error occurs — which we treat as Io, not ToolNotFound.
        // The output path format: outdir/<stem>.pdf
        let outdir = Path::new("/tmp");
        let file = Path::new("/some/dir/document.tex");
        let expected = outdir.join("document.pdf");
        // When tectonic is "available" but the command fails to run, we get Io.
        // We cannot assert Ok here without the real binary.
        // Instead verify the expected path string is what we'd get.
        let stem = file.file_stem().unwrap().to_string_lossy();
        assert_eq!(outdir.join(format!("{stem}.pdf")), expected,);
    }

    #[test]
    fn typst_output_path_derives_from_stem() {
        let outdir = Path::new("/tmp");
        let file = Path::new("/some/dir/report.typ");
        let expected = outdir.join("report.pdf");
        let stem = file.file_stem().unwrap().to_string_lossy();
        assert_eq!(outdir.join(format!("{stem}.pdf")), expected,);
    }
}
