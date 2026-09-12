//! File access, with a replace that cannot be observed half-done.

use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_platform::{FileSystem, FileSystemError};
use std::io::Write;
use std::path::Path;

/// Ordinary filesystem access.
#[derive(Clone, Copy, Debug, Default)]
pub struct StdFileSystem;

impl StdFileSystem
{
    /// Leaving a stray temporary behind after a failed rename would accumulate one file per
    /// failure next to the real one, so it is cleaned up.
    ///
    /// The rename's error stays the one reported, because it is the reason the caller's write
    /// did not happen — a cleanup that also failed is appended to it rather than replacing it,
    /// so neither failure is the price of naming the other.
    fn Unrenamed(path: &Path, temporary: &Path, error: &std::io::Error) -> FileSystemError
    {
        let Err(stray) = std::fs::remove_file(temporary)
        else
        {
            return Self::Classify(path, error);
        };

        return FileSystemError::Other {
            path: path.display().to_string(),
            cause: format!(
                "{error}, and the temporary {} it left could not be removed either: {stray}",
                temporary.display()
            ),
        };
    }

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

/// Writes and flushes a file completely before anybody can see it under its real name.
///
/// A rename over a partially written file publishes that partial content atomically, which
/// is worse than a torn write because it looks intact.
fn Write_Fully(temporary: &Path, contents: &str) -> Result<(), FileSystemError>
{
    let mut file = std::fs::File::create(temporary)
        .map_err(|error| return StdFileSystem::Classify(temporary, &error))?;
    file.write_all(contents.as_bytes())
        .map_err(|error| return StdFileSystem::Classify(temporary, &error))?;
    file.sync_all()
        .map_err(|error| return StdFileSystem::Classify(temporary, &error))?;

    return Ok(());
}

/// Reaches the real machine, so it reproduces nothing and says so.
impl Strategy for StdFileSystem
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::None;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::NotApplicable;
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

        Write_Fully(&temporary, contents)?;

        // Rename rather than truncate-and-write. A truncating write has a window in
        // which the file is empty; a reader arriving in that window sees a ledger with
        // no items, and a crash landing in it leaves one. A rename has no such window:
        // a reader sees the old contents or the new ones and never anything between.
        return std::fs::rename(&temporary, path)
            .map_err(|error| return Self::Unrenamed(path, &temporary, &error));
    }

    fn Exists(&self, path: &Path) -> bool
    {
        return path.exists();
    }

    fn Remove_File(&self, path: &Path) -> Result<(), FileSystemError>
    {
        return std::fs::remove_file(path).map_err(|error| Self::Classify(path, &error));
    }

    fn Read_Directory(&self, path: &Path) -> Result<Vec<std::path::PathBuf>, FileSystemError>
    {
        let entries = std::fs::read_dir(path).map_err(|error| Self::Classify(path, &error))?;

        return entries
            .map(|entry| return entry.map(|entry| return entry.path()).map_err(|error| Self::Classify(path, &error)))
            .collect();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn Test_Replace_Should_Round_Trip_Contents()
    {
        let path = Temporary_Path("round-trip");
        let filesystem = StdFileSystem;

        filesystem.Replace_Atomically(&path, "first").unwrap();
        assert_eq!(filesystem.Read_To_String(&path).unwrap(), "first");

        filesystem.Replace_Atomically(&path, "second").unwrap();
        assert_eq!(filesystem.Read_To_String(&path).unwrap(), "second");

        Removed_If_Present(&path);
    }

    /// A replace must not leave the temporary file behind. One stray file per write
    /// beside the real one is how a directory listing stops being readable.
    #[test]
    fn Test_Replace_Should_Not_Leave_A_Temporary_Behind()
    {
        let path = Temporary_Path("no-temp");
        let filesystem = StdFileSystem;

        filesystem.Replace_Atomically(&path, "contents").unwrap();

        assert!(!path.with_extension("tmp").exists());
        Removed_If_Present(&path);
    }

    /// A missing file must be distinguishable from an empty one, because the ledger
    /// treats them differently: absent means "not initialized yet", empty means
    /// "initialized and holding nothing".
    #[test]
    fn Test_A_Missing_File_Should_Be_Reported_As_Not_Found()
    {
        let path = Temporary_Path("absent");
        let filesystem = StdFileSystem;

        let error = filesystem.Read_To_String(&path).unwrap_err();

        assert!(matches!(error, FileSystemError::NotFound { .. }));
        assert!(!filesystem.Exists(&path));
    }

    #[test]
    fn Test_Replace_Should_Create_Missing_Parent_Directories()
    {
        let mut path = Temporary_Path("nested-parent");
        path.push("inner");
        path.push("ledger.json");
        let filesystem = StdFileSystem;

        filesystem.Replace_Atomically(&path, "{}").unwrap();

        assert_eq!(filesystem.Read_To_String(&path).unwrap(), "{}");
        if let Some(root) = path.parent().and_then(Path::parent)
            && let Err(cause) = std::fs::remove_dir_all(root)
            && cause.kind() != std::io::ErrorKind::NotFound
        {
            eprintln!("{} could not be cleared: {cause}", root.display());
        }
    }

    #[test]
    fn Test_Remove_File_Should_Delete_A_Real_File()
    {
        let path = Temporary_Path("remove");
        let filesystem = StdFileSystem;
        filesystem.Replace_Atomically(&path, "contents").unwrap();

        filesystem.Remove_File(&path).unwrap();

        assert!(!filesystem.Exists(&path));
    }

    #[test]
    fn Test_Remove_File_Should_Report_A_Missing_File_As_Not_Found()
    {
        let path = Temporary_Path("remove-absent");
        let filesystem = StdFileSystem;

        let error = filesystem.Remove_File(&path).unwrap_err();

        assert!(matches!(error, FileSystemError::NotFound { .. }));
    }

    #[test]
    fn Test_Read_Directory_Should_List_Every_Immediate_Child()
    {
        let directory = Temporary_Path("read-directory");
        std::fs::create_dir_all(&directory).unwrap();
        let filesystem = StdFileSystem;
        filesystem.Replace_Atomically(&directory.join("a.txt"), "a").unwrap();
        filesystem.Replace_Atomically(&directory.join("b.txt"), "b").unwrap();
        std::fs::create_dir_all(directory.join("nested")).unwrap();

        let mut entries = filesystem.Read_Directory(&directory).unwrap();
        entries.sort();

        assert_eq!(
            entries,
            vec![directory.join("a.txt"), directory.join("b.txt"), directory.join("nested")]
        );
        let _ignored = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn Test_Read_Directory_Should_Report_A_Missing_Directory_As_Not_Found()
    {
        let directory = Temporary_Path("read-directory-absent");
        let filesystem = StdFileSystem;

        let error = filesystem.Read_Directory(&directory).unwrap_err();

        assert!(matches!(error, FileSystemError::NotFound { .. }));
    }

    fn Temporary_Path(name: &str) -> PathBuf
    {
        let mut path = std::env::temp_dir();
        path.push(format!("nomos-fs-test-{name}-{}", std::process::id()));
        Removed_If_Present(&path);
        return path;
    }

    /// Removes a fixture file, tolerating the one failure that is not one.
    ///
    /// A path that is already absent is the state this asks for, so `NotFound` is success.
    /// Anything else is said out loud rather than discarded: a teardown that quietly cannot
    /// delete leaves one file per run in the temporary directory and never reports it, and a
    /// setup that quietly cannot delete hands the test a fixture a previous run wrote.
    fn Removed_If_Present(path: &Path)
    {
        if let Err(cause) = std::fs::remove_file(path)
            && cause.kind() != std::io::ErrorKind::NotFound
        {
            eprintln!("{} could not be cleared: {cause}", path.display());
        }
    }
}
