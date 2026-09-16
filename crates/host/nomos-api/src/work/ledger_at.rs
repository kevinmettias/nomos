//! [`Ledger_At`], the `FileLedger` composition every `Handle_Work_*` function in this crate
//! builds, and [`Run_Reservation_Command`], the ledger/territory/launcher wiring
//! [`super::claim::Handle_Work_Claim`], [`super::renew::Handle_Work_Renew`] and
//! [`super::take_over::Handle_Work_TakeOver`] shared byte-for-byte before this was extracted --
//! the one part of those three functions that was not itself the `WorkCommand` each names or
//! the `WorkOutcome` variant each destructures.

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
    fn Test_Run_Reservation_Command_Should_Run_A_Real_Claim_Against_The_Board_At_Directory()
    {
        let BoardWithAClaimableItem { directory, id } =
            Scratch_Board_With_A_Claimable_Item()
                .expect("the temp directory is writable and the scratch ledger is writable");
        let request = Claim_Request(id, "test-holder");

        let outcome = Run_Reservation_Command(&directory, WorkCommand::Claim(request));

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkOutcome::Claim(claimed) = outcome
        else
        {
            // This test hands Run_Reservation_Command a WorkCommand::Claim, and its own
            // contract guarantees the WorkOutcome it returns names that same command --
            // reaching else here means Run_Reservation_Command itself is broken, not a
            // condition this test should assert around.
            panic!("Run_Reservation_Command must return the WorkOutcome variant naming the WorkCommand it was given")
        };
        assert!(claimed.is_ok(), "{claimed:?}");
    }
}
