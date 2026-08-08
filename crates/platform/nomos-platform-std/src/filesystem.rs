//! File access, with a replace that cannot be observed half-done.

use nomos_platform::{FileSystem, FileSystemError};
use std::io::Write;
use std::path::Path;

/// Ordinary filesystem access.
#[derive(Clone, Copy, Debug, Default)]
pub struct StdFileSystem;

impl StdFileSystem
{
    fn Classify(path: &Path, error: &std::io::Error) -> FileSystemError
    {
        let displayed = path.display().to_string();

        return match error.kind()
        {
            std::io::ErrorKind::NotFound => FileSystemError::NotFound { path: displayed },
            std::io::ErrorKind::PermissionDenied => FileSystemError::Denied {
                path: displayed,
                cause: error.to_string(),
            },
            _ => FileSystemError::Other {
                path: displayed,
                cause: error.to_string(),
            },
        };
    }
}

impl FileSystem for StdFileSystem
{
    fn Read_To_String(&self, path: &Path) -> Result<String, FileSystemError>
    {
        return std::fs::read_to_string(path).map_err(|error| Self::Classify(path, &error));
    }

    fn Replace_Atomically(&self, path: &Path, contents: &str) -> Result<(), FileSystemError>
    {
        let temporary = path.with_extension("tmp");

        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent).map_err(|error| Self::Classify(parent, &error))?;
        }

        // Write and flush the temporary file completely before the rename. A rename
        // over a partially written file publishes that partial content atomically,
        // which is worse than a torn write because it looks intact.
        {
            let mut file =
                std::fs::File::create(&temporary).map_err(|error| Self::Classify(&temporary, &error))?;
            file.write_all(contents.as_bytes())
                .map_err(|error| Self::Classify(&temporary, &error))?;
            file.sync_all()
                .map_err(|error| Self::Classify(&temporary, &error))?;
        }

        // Rename rather than truncate-and-write. A truncating write has a window in
        // which the file is empty; a reader arriving in that window sees a ledger with
        // no items, and a crash landing in it leaves one. A rename has no such window:
        // a reader sees the old contents or the new ones and never anything between.
        return std::fs::rename(&temporary, path).map_err(|error| {
            // Leaving a stray temporary behind after a failed rename would accumulate
            // one file per failure next to the real one, so clean it up — but report
            // the rename's error, not the cleanup's.
            let _ = std::fs::remove_file(&temporary);
            Self::Classify(path, &error)
        });
    }

    fn Exists(&self, path: &Path) -> bool
    {
        return path.exists();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::path::PathBuf;

    fn Temp_Path(name: &str) -> PathBuf
    {
        let mut path = std::env::temp_dir();
        path.push(format!("nomos-fs-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_file(&path);
        return path;
    }

    #[test]
    fn Test_Replace_Should_Round_Trip_Contents()
    {
        let path = Temp_Path("round-trip");
        let filesystem = StdFileSystem;

        filesystem.Replace_Atomically(&path, "first").unwrap();
        assert_eq!(filesystem.Read_To_String(&path).unwrap(), "first");

        filesystem.Replace_Atomically(&path, "second").unwrap();
        assert_eq!(filesystem.Read_To_String(&path).unwrap(), "second");

        let _ = std::fs::remove_file(&path);
    }

    /// A replace must not leave the temporary file behind. One stray file per write
    /// beside the real one is how a directory listing stops being readable.
    #[test]
    fn Test_Replace_Should_Not_Leave_A_Temporary_Behind()
    {
        let path = Temp_Path("no-temp");
        let filesystem = StdFileSystem;

        filesystem.Replace_Atomically(&path, "contents").unwrap();

        assert!(!path.with_extension("tmp").exists());
        let _ = std::fs::remove_file(&path);
    }

    /// A missing file must be distinguishable from an empty one, because the ledger
    /// treats them differently: absent means "not initialized yet", empty means
    /// "initialized and holding nothing".
    #[test]
    fn Test_A_Missing_File_Should_Be_Reported_As_Not_Found()
    {
        let path = Temp_Path("absent");
        let filesystem = StdFileSystem;

        let error = filesystem.Read_To_String(&path).unwrap_err();

        assert!(matches!(error, FileSystemError::NotFound { .. }));
        assert!(!filesystem.Exists(&path));
    }

    #[test]
    fn Test_Replace_Should_Create_Missing_Parent_Directories()
    {
        let mut path = Temp_Path("nested-parent");
        path.push("inner");
        path.push("ledger.json");
        let filesystem = StdFileSystem;

        filesystem.Replace_Atomically(&path, "{}").unwrap();

        assert_eq!(filesystem.Read_To_String(&path).unwrap(), "{}");
        if let Some(root) = path.parent().and_then(Path::parent)
        {
            let _ = std::fs::remove_dir_all(root);
        }
    }
}
