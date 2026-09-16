//! [`Ledger_At`], the `FileLedger` composition every `Handle_Work_*` function in this crate
//! builds, and [`Run_Empty_Territory_Command`], the ledger/launcher wiring every verb of
//! `nomos_work_orchestration::WorkCommand` except `Add` shares -- the one part of a handler
//! that is neither the `WorkCommand` it names nor the `WorkOutcome` variant it destructures.

use nomos_ledger::{Board_In, FileLedger, Territory};
use nomos_composer_std::{CLOCK, FILE_SYSTEM, LAUNCHER, Lock_At};
use nomos_composer_std::{ClockType, FileSystemType, LockType};
use nomos_work_orchestration::{WorkCommand, WorkOutcome};
use std::path::Path;

/// The `FileLedger` composition every `Handle_Work_*` function in this crate builds.
pub(crate) fn Ledger_At(directory: &Path) -> FileLedger<FileSystemType, ClockType, LockType>
{
    let board = Board_In(directory);

    return FileLedger::At(board.document, FILE_SYSTEM, CLOCK, Lock_At(board.lock));
}

/// Runs `command` against the board at `directory` with an empty, real `Territory`.
///
/// Every `WorkCommand` variant but `Add` carries no territory of its own: `ClaimRequest`
/// names only the item and holder it already names, `Finish` an item and a holder, `Validate`
/// nothing at all. `nomos_work_orchestration::Run`'s own `published` closure exists so each
/// composition root can answer "which records has this repository already published" its own
/// way, and that crate's own doc says the closure is asked for lazily and reached only by
/// `Add` -- so every other verb hands this empty, real `Territory` instead of a closure over a
/// directory it will never look at. `add_response::Handle_Work_Add` is the one handler in this
/// crate that passes a populated one.
pub(crate) fn Run_Empty_Territory_Command(directory: &Path, command: WorkCommand) -> WorkOutcome
{
    let mut ledger = Ledger_At(directory);

    return nomos_work_orchestration::Run(
        &command,
        &mut ledger,
        &LAUNCHER,
        || Territory::Of_Files(std::iter::empty::<String>()),
    );
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::work::tests_support::{
        BoardWithAClaimableItem, Claim_Request, Scratch_Board_With_A_Claimable_Item,
    };

    #[test]
    fn Test_Ledger_At_Should_Compose_A_File_Ledger_That_Loads_A_Real_Boards_Own_File()
    {
        let BoardWithAClaimableItem { directory, .. } =
            Scratch_Board_With_A_Claimable_Item()
                .expect("the temp directory is writable and the scratch ledger is writable");

        let ledger = Ledger_At(&directory);
        let loaded = ledger.Load().expect("loads the ledger this fixture just wrote");

        let _ignored = std::fs::remove_dir_all(&directory);

        assert_eq!(loaded.items.len(), 1, "{loaded:?}");
    }

    #[test]
    fn Test_Run_Empty_Territory_Command_Should_Run_A_Real_Claim_Against_The_Board_At_Directory()
    {
        let BoardWithAClaimableItem { directory, id } =
            Scratch_Board_With_A_Claimable_Item()
                .expect("the temp directory is writable and the scratch ledger is writable");
        let request = Claim_Request(id, "test-holder");

        let outcome = Run_Empty_Territory_Command(&directory, WorkCommand::Claim(request));

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkOutcome::Claim(claimed) = outcome
        else
        {
            // This test hands Run_Empty_Territory_Command a WorkCommand::Claim, and its own
            // contract guarantees the WorkOutcome it returns names that same command --
            // reaching else here means Run_Empty_Territory_Command itself is broken, not a
            // condition this test should assert around.
            panic!(
                "Run_Empty_Territory_Command must return the WorkOutcome variant naming the WorkCommand it was given"
            )
        };
        assert!(claimed.is_ok(), "{claimed:?}");
    }
}
