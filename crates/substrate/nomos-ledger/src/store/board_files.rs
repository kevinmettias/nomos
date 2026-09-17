//! The two files a board is kept in, named once and handed back together.
//!
//! A board on disk is a document and a lock beside it. Before this module every composition
//! root knew that independently — `nomos-api`'s `work::ledger_at` and `nomos-cli`'s `work`
//! each joined `"ledger.json"` and `"ledger.lock"` onto a directory themselves, and so did
//! this crate's own fixtures — even though the pairing is this crate's to own: it is the
//! crate whose format the document is, and whose exclusion the lock enforces.
//!
//! # Why a value, rather than two constants
//!
//! Two constants would let a caller reach for one and not the other, which is the defect
//! restated rather than closed. The real hazard was never that the names were repeated; it
//! is that a caller repeating them can pair them *wrongly* — take the document at one path
//! and the lock at another — and nothing would say so. Both files would compile. Every test
//! in this workspace would still pass, because a lock guarding nothing is indistinguishable
//! from a lock guarding something until two processes actually race.
//!
//! [`Board_In`] closes that by construction: one call, both paths, derived from one
//! directory. A caller cannot obtain one half of the pairing without the other, so there is
//! no call site left at which the two can disagree.
//!
//! # Why this crate does not open either file
//!
//! [`BoardFiles`] is paths and nothing else. [`crate::FileLedger`] is generic over
//! `nomos_platform::FilesystemLock`, and the concrete `FileLock` belongs to
//! `nomos-platform-std` — `Zone::Backend` since `OD-RULES-028`, which `Substrate` may reach
//! but this crate has no reason to. Naming the files is this crate's job; opening them is
//! the composition root's, which is the same division `FileLedger::At` already draws by
//! taking a lock rather than making one.

use std::path::{Path, PathBuf};

/// The filename a board's document is kept under.
pub const DOCUMENT_FILENAME: &str = "ledger.json";

/// The filename of the lock guarding that document.
///
/// Beside the document rather than inside it, and a separate file rather than a region of
/// the same one, because the document is rewritten whole on every save — a lock living in
/// it would be released by the write it exists to guard.
pub const LOCK_FILENAME: &str = "ledger.lock";

/// The two files of the board in `directory`.
///
/// Both paths or neither. See this module's own documentation for why the pairing is a value
/// rather than two constants a caller joins itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardFiles
{
    /// The document the board's items are stored in.
    pub document: PathBuf,

    /// The lock a writer must hold before changing that document.
    pub lock: PathBuf,
}

/// The board kept in `directory`.
///
/// Names two paths and touches neither. A caller composing a [`crate::FileLedger`] hands
/// `document` to `FileLedger::At` and `lock` to whichever `FilesystemLock` its own
/// platform provides, which is the only shape in which the two can be told apart at all.
#[must_use]
pub fn Board_In(directory: &Path) -> BoardFiles
{
    return BoardFiles {
        document: directory.join(DOCUMENT_FILENAME),
        lock: directory.join(LOCK_FILENAME),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The pairing every composition root depends on, and the one nothing asserted before
    /// this module existed: the lock is beside the document, not merely somewhere.
    ///
    /// Written against the paths rather than against the two literals on purpose. Asserting
    /// `document` ends in `"ledger.json"` would only restate the constant and would pass
    /// just as happily if the lock had been pointed at another directory entirely, which is
    /// the exact failure this is here to catch.
    #[test]
    fn Test_A_Boards_Lock_Should_Sit_Beside_Its_Own_Document()
    {
        let board = Board_In(Path::new("/somewhere/work"));

        assert_eq!(
            board.document.parent(),
            board.lock.parent(),
            "a lock in a different directory from the document guards nothing: {board:?}"
        );
    }

    /// The other half: beside it, and not *it*. A lock and a document at one path would be
    /// a writer holding a lock on the file it is replacing.
    #[test]
    fn Test_A_Boards_Lock_Should_Not_Be_Its_Document()
    {
        let board = Board_In(Path::new("/somewhere/work"));

        assert_ne!(board.document, board.lock, "{board:?}");
    }

    /// Both paths are the caller's directory, so a board named for one directory cannot
    /// reach into another.
    #[test]
    fn Test_Both_Files_Should_Be_In_The_Directory_The_Caller_Named()
    {
        let directory = Path::new("/somewhere/work");

        let board = Board_In(directory);

        assert_eq!(board.document.parent(), Some(directory));
        assert_eq!(board.lock.parent(), Some(directory));
    }

    /// The names themselves, pinned once. `nomos work` is run by hand against boards that
    /// already exist on disk, so these are a published format and not an implementation
    /// detail this crate may rename freely.
    #[test]
    fn Test_The_Two_Filenames_Should_Be_The_Published_Ones()
    {
        let board = Board_In(Path::new("work"));

        assert_eq!(board.document, Path::new("work").join("ledger.json"));
        assert_eq!(board.lock, Path::new("work").join("ledger.lock"));
    }
}
