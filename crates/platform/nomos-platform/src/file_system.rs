//! Reading and writing files, with the durability rules stated rather than assumed.

mod error;

pub use error::Error as FileSystemError;

use nomos_contracts::Strategy;
use std::path::{Path, PathBuf};

/// File access.
///
/// Small on purpose. This is not an abstraction over filesystems in general — it is the
/// set of operations Nomos's durable state actually needs: read, atomically replace, check
/// existence, and remove. The fourth is defaulted rather than required, so it does not
/// widen what every implementor must supply — see [`FileSystem::Remove_File`].
/// # What an implementor promises
///
/// The supertrait is [`nomos_contracts::Strategy`], so every implementor states its
/// determinism triple. This is the seam where that question is sharpest and where it had
/// no answer: the twenty-nine types in this workspace that declared a triple were rules,
/// providers and formats, and not one of them was a port -- while the implementations
/// that actually cross the machine boundary, and the doubles that stand in for them,
/// declared nothing. The real one promises nothing and says so; a double built from fixed
/// data reproduces and says that. A caller reading `S::STRENGTH` can tell them apart
/// without knowing either type.
pub trait FileSystem: Strategy
{
    /// Reads a file's entire contents as text.
    ///
    /// # Errors
    ///
    /// Returns [`FileSystemError::NotFound`] when the file is absent, and a
    /// [`FileSystemError::Denied`] or [`FileSystemError::Other`] otherwise.
    fn Read_To_String(&self, path: &Path) -> Result<String, FileSystemError>;

    /// Replaces a file's contents atomically.
    ///
    /// **Write to a temporary file and rename**, never truncate and rewrite. A
    /// truncating write has a window in which the file is empty or half-written, and a
    /// reader that arrives in that window — or a crash that lands in it — sees a
    /// ledger with no items in it. A rename has no such window: a reader sees either
    /// the old contents or the new ones.
    ///
    /// # Errors
    ///
    /// Returns a [`FileSystemError`] when the file could not be replaced. On failure
    /// the previous contents must remain intact.
    fn Replace_Atomically(&self, path: &Path, contents: &str) -> Result<(), FileSystemError>;

    /// Whether a path exists.
    fn Exists(&self, path: &Path) -> bool;

    /// Removes a file.
    ///
    /// Defaulted, unlike this trait's other three operations: most implementors — a fake
    /// built to exercise one narrow test — have no real file to remove and no reason to
    /// support removing one. An implementor backing real durable state overrides this to
    /// remove for real; one that does not inherits a refusal rather than silently doing
    /// nothing, which would read exactly like a successful removal to a caller that checked
    /// only the `Result`.
    ///
    /// # Errors
    ///
    /// Returns a [`FileSystemError`] when the file could not be removed, or when this
    /// implementation does not support removal at all.
    fn Remove_File(&self, path: &Path) -> Result<(), FileSystemError>
    {
        return Err(FileSystemError::Other {
            path: path.display().to_string(),
            cause: "removal is not supported by this FileSystem implementation".to_owned(),
        });
    }

    /// The immediate children of a directory: every entry's full path, files and
    /// subdirectories together, in whatever order the filesystem reports them.
    ///
    /// Defaulted, for the identical reason [`FileSystem::Remove_File`] is: most implementors
    /// exist to exercise one narrow, hand-written fixture and have no directory to enumerate.
    /// An implementor backing a real tree overrides this; one that does not inherits a
    /// refusal rather than an empty listing, which would read exactly like a real, empty
    /// directory to a caller that checked only the `Result`.
    ///
    /// One level, not a walk: `OD-PLATFORM-002` is this port's floor, not a recursive
    /// traversal, so a caller that needs to descend composes its own recursion from this
    /// primitive the same way [`Self::Read_To_String`]'s callers already compose their own
    /// retry or fallback policy from a single read.
    ///
    /// # Errors
    ///
    /// Returns [`FileSystemError::NotFound`] when `path` does not exist, and a
    /// [`FileSystemError::Denied`] or [`FileSystemError::Other`] otherwise -- including when
    /// this implementation does not support enumeration at all.
    fn Read_Directory(&self, path: &Path) -> Result<Vec<PathBuf>, FileSystemError>
    {
        return Err(FileSystemError::Other {
            path: path.display().to_string(),
            cause: "directory enumeration is not supported by this FileSystem implementation".to_owned(),
        });
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{DeterminismStrength, ReproducibilityScope, TraceEquivalence};

    /// An implementor that overrides none of `FileSystem`'s three required operations
    /// meaningfully, so that `Remove_File`'s own default is the only behaviour under test.
    struct NoOverride;

    /// Answers from fixed data, so its outputs reproduce byte for byte.
    impl Strategy for NoOverride
    {
        const STRENGTH: DeterminismStrength = DeterminismStrength::State;
        const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
        const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
    }

    impl FileSystem for NoOverride
    {
        fn Read_To_String(&self, _path: &Path) -> Result<String, FileSystemError>
        {
            unimplemented!("not exercised by this test")
        }

        fn Replace_Atomically(&self, _path: &Path, _contents: &str) -> Result<(), FileSystemError>
        {
            unimplemented!("not exercised by this test")
        }

        fn Exists(&self, _path: &Path) -> bool
        {
            unimplemented!("not exercised by this test")
        }
    }

    #[test]
    fn Test_An_Implementor_That_Does_Not_Override_Read_Directory_Should_Refuse_Rather_Than_Succeed()
    {
        let error = NoOverride.Read_Directory(Path::new("anything")).unwrap_err();

        assert!(
            matches!(error, FileSystemError::Other { .. }),
            "a fake that never enumerates anything must say so rather than report an empty \
             directory for an enumeration it never performed"
        );
    }

    #[test]
    fn Test_An_Implementor_That_Does_Not_Override_Remove_File_Should_Refuse_Rather_Than_Succeed()
    {
        let error = NoOverride.Remove_File(Path::new("anything")).unwrap_err();

        assert!(
            matches!(error, FileSystemError::Other { .. }),
            "a fake that never removes anything must say so rather than report success for a \
             removal it never performed"
        );
    }
}
