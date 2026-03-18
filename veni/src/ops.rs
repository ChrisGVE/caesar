use crate::error::{Result, VeniError};
use std::path::{Path, PathBuf};

/// Represents a reversible file-system operation.
#[derive(Debug, Clone)]
pub enum FileOp {
    /// Copy one or more sources into a destination directory.
    Copy {
        sources: Vec<PathBuf>,
        dest: PathBuf,
    },
    /// Move one or more sources into a destination directory.
    Move {
        sources: Vec<PathBuf>,
        dest: PathBuf,
    },
    /// Delete one or more paths (optionally moving them to trash first).
    Delete {
        paths: Vec<PathBuf>,
        trash: bool,
        /// Recorded trash paths so delete can be undone.
        trash_paths: Vec<PathBuf>,
    },
    /// Rename a single file or directory in place.
    Rename {
        /// Original path before rename.
        from: PathBuf,
        /// New path after rename.
        to: PathBuf,
    },
}

/// Base directory used as the trash store.
fn trash_dir() -> PathBuf {
    dirs_next::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("veni")
        .join("trash")
}

/// Execute a file operation.
pub fn execute_op(op: &FileOp) -> Result<()> {
    match op {
        FileOp::Copy { sources, dest } => copy_files(sources, dest),
        FileOp::Move { sources, dest } => move_files(sources, dest),
        FileOp::Delete {
            paths,
            trash,
            trash_paths: _,
        } => delete_files(paths, *trash),
        FileOp::Rename { from, to } => std::fs::rename(from, to).map_err(VeniError::Io),
    }
}

/// Build the inverse of a completed operation so it can be undone.
///
/// - Inverse of Copy   → Delete the copies (no trash needed, they are new).
/// - Inverse of Move   → Move the files back to their original locations.
/// - Inverse of Delete → Restore the files from trash back to their original paths.
pub fn inverse_op(op: &FileOp) -> FileOp {
    match op {
        FileOp::Copy { sources, dest } => {
            // The copies live at dest/<filename>.
            let copy_paths: Vec<PathBuf> = sources
                .iter()
                .filter_map(|s| s.file_name().map(|n| dest.join(n)))
                .collect();
            FileOp::Delete {
                paths: copy_paths,
                trash: false,
                trash_paths: Vec::new(),
            }
        }
        FileOp::Move { sources, dest } => {
            // The files now live at dest/<filename>; move them back.
            let moved_paths: Vec<PathBuf> = sources
                .iter()
                .filter_map(|s| s.file_name().map(|n| dest.join(n)))
                .collect();
            // Original locations are the sources themselves.
            // We repurpose sources as dest dir-per-file by getting the parent.
            // Each original source had a unique parent — we reconstruct via
            // Move { sources: moved_paths, dest_per_file: original_parents }.
            // Since FileOp::Move uses a single dest dir, we can only undo
            // cleanly when all sources share the same parent.
            let original_dest = sources
                .first()
                .and_then(|p| p.parent())
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            FileOp::Move {
                sources: moved_paths,
                dest: original_dest,
            }
        }
        FileOp::Delete {
            paths, trash_paths, ..
        } => {
            // Restore: move each trash path back to its original location.
            // We store original paths in `paths` and corresponding trash
            // locations in `trash_paths`.
            FileOp::Move {
                sources: trash_paths.clone(),
                dest: paths
                    .first()
                    .and_then(|p| p.parent())
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| PathBuf::from(".")),
            }
        }
        // Inverse of Rename is just swapping from/to.
        FileOp::Rename { from, to } => FileOp::Rename {
            from: to.clone(),
            to: from.clone(),
        },
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn copy_files(sources: &[PathBuf], dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    for src in sources {
        let file_name = src.file_name().ok_or_else(|| {
            VeniError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("source has no file name: {}", src.display()),
            ))
        })?;
        let dst_path = dest.join(file_name);
        if src.is_dir() {
            copy_dir_recursive(src, &dst_path)?;
        } else {
            std::fs::copy(src, &dst_path)?;
        }
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dst = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &dst)?;
        } else {
            std::fs::copy(entry.path(), &dst)?;
        }
    }
    Ok(())
}

fn move_files(sources: &[PathBuf], dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    for src in sources {
        let file_name = src.file_name().ok_or_else(|| {
            VeniError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("source has no file name: {}", src.display()),
            ))
        })?;
        let dst_path = dest.join(file_name);
        // Try atomic rename first; fall back to copy+delete for cross-device.
        if std::fs::rename(src, &dst_path).is_err() {
            if src.is_dir() {
                copy_dir_recursive(src, &dst_path)?;
                std::fs::remove_dir_all(src)?;
            } else {
                std::fs::copy(src, &dst_path)?;
                std::fs::remove_file(src)?;
            }
        }
    }
    Ok(())
}

fn delete_files(paths: &[PathBuf], use_trash: bool) -> Result<()> {
    if use_trash {
        let trash = trash_dir();
        std::fs::create_dir_all(&trash)?;
        for path in paths {
            if let Some(name) = path.file_name() {
                let dst = trash.join(name);
                // Rename into trash; fall back to copy+delete cross-device.
                if std::fs::rename(path, &dst).is_err() {
                    if path.is_dir() {
                        copy_dir_recursive(path, &dst)?;
                        std::fs::remove_dir_all(path)?;
                    } else {
                        std::fs::copy(path, &dst)?;
                        std::fs::remove_file(path)?;
                    }
                }
            }
        }
    } else {
        for path in paths {
            if path.is_dir() {
                std::fs::remove_dir_all(path)?;
            } else {
                std::fs::remove_file(path)?;
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ------------------------------------------------------------------
    // Copy
    // ------------------------------------------------------------------

    #[test]
    fn copy_single_file() {
        let src_dir = TempDir::new().unwrap();
        let dst_dir = TempDir::new().unwrap();
        let src = src_dir.path().join("hello.txt");
        fs::write(&src, b"hello").unwrap();

        let op = FileOp::Copy {
            sources: vec![src.clone()],
            dest: dst_dir.path().to_path_buf(),
        };
        execute_op(&op).unwrap();

        let dst = dst_dir.path().join("hello.txt");
        assert!(dst.exists());
        assert_eq!(fs::read(&dst).unwrap(), b"hello");
        // Source still exists after copy.
        assert!(src.exists());
    }

    #[test]
    fn copy_directory_recursively() {
        let src_dir = TempDir::new().unwrap();
        let dst_dir = TempDir::new().unwrap();

        let sub = src_dir.path().join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("file.txt"), b"data").unwrap();

        let op = FileOp::Copy {
            sources: vec![sub.clone()],
            dest: dst_dir.path().to_path_buf(),
        };
        execute_op(&op).unwrap();

        let dst_file = dst_dir.path().join("sub").join("file.txt");
        assert!(dst_file.exists());
    }

    // ------------------------------------------------------------------
    // Move
    // ------------------------------------------------------------------

    #[test]
    fn move_single_file() {
        let src_dir = TempDir::new().unwrap();
        let dst_dir = TempDir::new().unwrap();
        let src = src_dir.path().join("move_me.txt");
        fs::write(&src, b"moving").unwrap();

        let op = FileOp::Move {
            sources: vec![src.clone()],
            dest: dst_dir.path().to_path_buf(),
        };
        execute_op(&op).unwrap();

        let dst = dst_dir.path().join("move_me.txt");
        assert!(dst.exists());
        assert_eq!(fs::read(&dst).unwrap(), b"moving");
        // Source must no longer exist after move.
        assert!(!src.exists());
    }

    // ------------------------------------------------------------------
    // Delete
    // ------------------------------------------------------------------

    #[test]
    fn delete_file_no_trash() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("delete_me.txt");
        fs::write(&file, b"bye").unwrap();

        let op = FileOp::Delete {
            paths: vec![file.clone()],
            trash: false,
            trash_paths: Vec::new(),
        };
        execute_op(&op).unwrap();
        assert!(!file.exists());
    }

    #[test]
    fn delete_directory_no_trash() {
        let dir = TempDir::new().unwrap();
        let sub = dir.path().join("rm_dir");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("inner.txt"), b"").unwrap();

        let op = FileOp::Delete {
            paths: vec![sub.clone()],
            trash: false,
            trash_paths: Vec::new(),
        };
        execute_op(&op).unwrap();
        assert!(!sub.exists());
    }

    // ------------------------------------------------------------------
    // Inverse operations
    // ------------------------------------------------------------------

    #[test]
    fn inverse_of_copy_is_delete_of_copies() {
        let src_dir = TempDir::new().unwrap();
        let dst_dir = TempDir::new().unwrap();
        let src = src_dir.path().join("f.txt");
        fs::write(&src, b"x").unwrap();

        let op = FileOp::Copy {
            sources: vec![src.clone()],
            dest: dst_dir.path().to_path_buf(),
        };
        execute_op(&op).unwrap();

        let inv = inverse_op(&op);
        execute_op(&inv).unwrap();

        // The copy in dst_dir should now be gone.
        assert!(!dst_dir.path().join("f.txt").exists());
        // The original is still there.
        assert!(src.exists());
    }

    #[test]
    fn inverse_of_move_moves_back() {
        let src_dir = TempDir::new().unwrap();
        let dst_dir = TempDir::new().unwrap();
        let src = src_dir.path().join("back.txt");
        fs::write(&src, b"come back").unwrap();

        let op = FileOp::Move {
            sources: vec![src.clone()],
            dest: dst_dir.path().to_path_buf(),
        };
        execute_op(&op).unwrap();
        assert!(!src.exists());

        let inv = inverse_op(&op);
        execute_op(&inv).unwrap();

        assert!(src.exists());
        assert_eq!(fs::read(&src).unwrap(), b"come back");
    }

    // ------------------------------------------------------------------
    // Rename
    // ------------------------------------------------------------------

    #[test]
    fn rename_file() {
        let dir = TempDir::new().unwrap();
        let from = dir.path().join("old.txt");
        let to = dir.path().join("new.txt");
        fs::write(&from, b"data").unwrap();

        let op = FileOp::Rename {
            from: from.clone(),
            to: to.clone(),
        };
        execute_op(&op).unwrap();

        assert!(!from.exists(), "old path must be gone after rename");
        assert!(to.exists(), "new path must exist after rename");
        assert_eq!(fs::read(&to).unwrap(), b"data");
    }

    #[test]
    fn inverse_of_rename_renames_back() {
        let dir = TempDir::new().unwrap();
        let from = dir.path().join("original.txt");
        let to = dir.path().join("renamed.txt");
        fs::write(&from, b"hello").unwrap();

        let op = FileOp::Rename {
            from: from.clone(),
            to: to.clone(),
        };
        execute_op(&op).unwrap();
        assert!(to.exists());

        let inv = inverse_op(&op);
        execute_op(&inv).unwrap();

        assert!(from.exists(), "original path must be restored after undo");
        assert!(!to.exists(), "renamed path must be gone after undo");
    }
}
