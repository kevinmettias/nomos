//! [`Handle_Work_Finish`] and its own [`FinishResponse`].

use nomos_ledger::{FinishRefusal, ItemId, Territory, VerificationRecord};
use serde::Serialize;
use std::path::Path;

/// Runs `item`'s verification predicate as `holder` on the board at `directory`, recording
/// it done if it passes, exactly as `nomos work finish` would, and hands back a
/// JSON-serializable response.
///
/// `nomos_work_orchestration::Run`'s own `WorkCommand::Finish` arm always calls
/// `nomos_ledger::Finish_Item` with a `None` `working_directory` -- not a choice this function or
/// its caller can vary, so the gate's own lint step (derived from `.github/workflows/
/// gate.yml`) resolves relative to the calling process's own current directory, the same
/// fixed composition `nomos-cli`'s own `nomos work finish` already runs under.
#[must_use]
pub fn Handle_Work_Finish(directory: &Path, item: &ItemId, holder: &str) -> FinishResponse
{
    use super::Ledger_At;
    use nomos_composer_std::LAUNCHER;
    use nomos_work_orchestration::WorkCommand;

    let mut ledger = Ledger_At(directory);

    let outcome = nomos_work_orchestration::Run(
        &WorkCommand::Finish { item: item.clone(), holder: holder.to_owned() },
        &mut ledger,
        &LAUNCHER,
        || Territory::Of_Files(std::iter::empty::<String>()),
    );

    let nomos_work_orchestration::WorkOutcome::Finish(finished) = outcome
    else
    {
        // rust-panic: allow: Run's own contract guarantees it returns the WorkOutcome variant
        // naming the WorkCommand it was given -- any other outcome means Run itself is broken.
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return FinishResponse::From(finished);
}

/// What a real `nomos work finish` produced, in a shape `serde_json` can hand across a wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum FinishResponse
{
    /// The predicate passed, and the item is recorded done.
    Finished
    {
        /// The evidence. `nomos_ledger::VerificationRecord` already derives `Serialize` --
        /// it is stored directly in `ledger.json` -- so no local twin is needed.
        record: VerificationRecord,
    },
    /// The item was not finished.
    Refused
    {
        /// What went wrong, as `FinishRefusal::Describe` renders it.
        cause: String,
        /// Whether the predicate ran and said no -- `FinishRefusal::Has_Judged_The_Work`, the
        /// same signal `crates/host/nomos-cli/src/work/report.rs`'s own `Report_Finish`
        /// already branches an exit code on (a real judgment versus nobody finding out),
        /// handed to a wire caller directly since it has no process exit code to read.
        judged: bool,
    },
}

impl FinishResponse
{
    pub(crate) fn From(result: Result<VerificationRecord, FinishRefusal>) -> Self
    {
        return match result
        {
            Ok(record) => Self::Finished { record },
            Err(refusal) => Self::Refused { judged: refusal.Has_Judged_The_Work(), cause: refusal.Describe() },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::work::tests_support::{Scratch_Board, Scratch_Board_With_A_Claimable_Item};

    #[test]
    fn Test_Handle_Work_Finish_Should_Refuse_And_Not_Judge_An_Id_Absent_From_A_Real_Readable_Board()
    {
        let directory = Scratch_Board();
        let absent = ItemId::New("NO-SUCH-ITEM");

        let response = Handle_Work_Finish(&directory, &absent, "test-holder");

        let _ignored = std::fs::remove_dir_all(&directory);

        let FinishResponse::Refused { judged, cause } = response
        else
        {
            // This fixture writes a real, readable board and then finishes an id that was
            // never put on it, so a refusal is the only correct outcome -- reaching a finish
            // here means the id-lookup check itself stopped enforcing, not a condition this
            // test should assert around.
            panic!("an id absent from a real, readable board is a real refusal, not a finish");
        };
        assert!(!judged, "{cause}");
    }

    #[test]
    fn Test_Scratch_Board_With_A_Claimable_Item_Should_Be_Refused_And_Not_Judged_When_Finished_With_No_Verification_Predicate()
    {
        let (directory, id) = Scratch_Board_With_A_Claimable_Item();

        let response = Handle_Work_Finish(&directory, &id, "test-holder");

        let _ignored = std::fs::remove_dir_all(&directory);

        let FinishResponse::Refused { judged, cause } = response
        else
        {
            // This fixture builds an item with no `verification` field set at all, so
            // finishing it must refuse -- reaching `Finished` here means the
            // no-predicate check itself stopped enforcing, not a condition this test should
            // assert around.
            panic!("an item with no verification predicate at all is a real refusal, not a finish");
        };
        assert!(!judged, "{cause}");
    }

    #[test]
    fn Test_From_Should_Produce_A_Refused_Finish_Response_That_Round_Trips_As_Json()
    {
        let directory = Scratch_Board();
        let absent = ItemId::New("NO-SUCH-ITEM");

        let response = Handle_Work_Finish(&directory, &absent, "test-holder");

        let _ignored = std::fs::remove_dir_all(&directory);

        crate::test_support::Assert_Round_Trips_As_Json(&response, "refused");
    }
}
