//! [`Ledger_At`], the `FileLedger` composition every `Handle_Work_*` function in this crate
//! builds, and [`Run_Reservation_Command`], the ledger/territory/launcher wiring
//! [`super::claim::Handle_Work_Claim`], [`super::renew::Handle_Work_Renew`] and
//! [`super::take_over::Handle_Work_TakeOver`] shared byte-for-byte before this was extracted --
//! the one part of those three functions that was not itself the `WorkCommand` each names or
//! the `WorkOutcome` variant each destructures.

use nomos_ledger::{FileLedger, Territory};
use nomos_platform_std::{FileLock, StdFileSystem, StdProcessLauncher, SystemClock};
use nomos_work_orchestration::{WorkCommand, WorkOutcome};
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

/// Runs `command` against the board at `directory`, exactly as [`super::claim::Handle_Work_Claim`],
/// [`super::renew::Handle_Work_Renew`] and [`super::take_over::Handle_Work_TakeOver`] each do --
/// none of the three reservation verbs reaches a territory of its own (`ClaimRequest` carries
/// only the item and holder it already names), so each hands an empty, real `Territory` here
/// the same way `Handle_Work_List`, `Handle_Work_Show` and this module's other non-`Add` verbs
/// already do inline for the same reason.
pub(crate) fn Run_Reservation_Command(directory: &Path, command: WorkCommand) -> WorkOutcome
{
    let mut ledger = Ledger_At(directory);

    return nomos_work_orchestration::Run(
        &command,
        &mut ledger,
        &StdProcessLauncher,
        || Territory::Of_Files(std::iter::empty::<String>()),
    );
}
