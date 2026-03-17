use crate::error::{Result, VidiError};
use std::io::Write as _;
use std::path::Path;

/// Write internally rendered ANSI bytes to stdout, truncated to `lines`.
pub fn launch_internal_inline(bytes: Vec<u8>, lines: u16) -> Result<()> {
    use crate::launcher::inline::truncate_ansi_safe;
    let truncated = truncate_ansi_safe(&bytes, lines);
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    lock.write_all(&truncated)?;
    Ok(())
}

/// Page internally rendered ANSI bytes through `less -R`.
///
/// Writes bytes to a temporary file and exec()-s into `less -R`.
/// On Unix this replaces the process; on other platforms it spawns and waits.
pub fn launch_internal_fullscreen(bytes: Vec<u8>) -> Result<()> {
    let mut tmp = tempfile::Builder::new().suffix(".ansi").tempfile()?;
    tmp.write_all(&bytes)?;
    tmp.flush()?;
    let path = tmp
        .into_temp_path()
        .keep()
        .map_err(|e| VidiError::Io(e.error))?;
    exec_less_r(&path)
}

fn exec_less_r(path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy().into_owned();

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = std::process::Command::new("less")
            .args(["-R", &path_str])
            .exec();
        Err(VidiError::ToolFailed {
            tool: "less".to_string(),
            code: err.raw_os_error().unwrap_or(1),
        })
    }

    #[cfg(not(unix))]
    {
        let status = std::process::Command::new("less")
            .args(["-R", &path_str])
            .status()
            .map_err(|_| VidiError::ToolNotFound {
                tool: "less".to_string(),
            })?;
        if status.success() {
            Ok(())
        } else {
            Err(VidiError::ToolFailed {
                tool: "less".to_string(),
                code: status.code().unwrap_or(1),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_internal_inline_truncates_to_lines() {
        let bytes = b"line1\nline2\nline3\n".to_vec();
        // Inline output goes to stdout; we can't easily capture it in a unit test.
        // Verify it does not panic and the function succeeds.
        let result = launch_internal_inline(bytes, 3);
        // This writes to the actual stdout during test — acceptable for unit tests.
        assert!(result.is_ok());
    }

    #[test]
    fn launch_internal_inline_empty_bytes_ok() {
        let result = launch_internal_inline(Vec::new(), 10);
        assert!(result.is_ok());
    }
}
