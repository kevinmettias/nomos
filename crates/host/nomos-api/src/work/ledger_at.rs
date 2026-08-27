//! [`Ledger_At`], the `FileLedger` composition every `Handle_Work_*` function in this crate
//! builds.

use nomos_ledger::FileLedger;
use nomos_platform_std::{FileLock, StdFileSystem, SystemClock};
use std::path::Path;

/// The `FileLedger` composition every `Handle_Work_*` function in this crate builds.
pub(crate) fn Ledger_At(directory: &Path) -> FileLedger<StdFileSystem, SystemClock, FileLock>
{
    return FileLedger::At(
        directory.join("ledger.json"),
        StdFileSystem,
        SystemClock,
        FileLock::At(directory.join("ledger.lock")),
    );
}
