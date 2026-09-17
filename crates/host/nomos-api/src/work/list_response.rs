//! [`Handle_Work_List`] and its own [`ListResponse`].

use nomos_ledger::LedgerDocument;
use nomos_platform::Timestamp;
use nomos_work_orchestration::WorkCommand;
use serde::Serialize;
use std::path::Path;

/// Lists every item on the board at `directory`, exactly as `nomos work list` would, and
/// hands back a JSON-serializable response.
///
/// `directory` is expected to hold `ledger.json` and `ledger.lock`, the same layout
/// `crates/host/nomos-cli/src/work.rs`'s own composition root reads. The `published`
/// closure `nomos_work_orchestration::Run` takes is asked for lazily and reached only by
/// `WorkCommand::Add` (that crate's own doc), so `List` never reaches it: this handler runs
/// through [`super::Run_Empty_Territory_Command`], which hands the empty, real `Territory`
/// every non-`Add` verb wants rather than a closure that would panic if this crate's own
/// scope ever widened past `List` without updating this comment.
#[must_use]
pub fn Handle_Work_List(directory: &Path) -> ListResponse
{
    let outcome = super::Run_Empty_Territory_Command(directory, WorkCommand::List { state: None });

    let nomos_work_orchestration::WorkOutcome::List(listed) = outcome
    else
    {
        return ListResponse::Unreadable {
            cause: super::UNANSWERED_WORK_OUTCOME.to_owned(),
        };
    };

    return ListResponse::From(listed);
}

/// What a real `nomos work list` produced, in a shape `serde_json` can hand across a wire.
///
/// A tagged enum rather than a bare `Result`-shaped struct: `nomos_ledger::LedgerError`
/// does not derive `Serialize` -- nothing needed a wire format for it before this crate
/// existed -- and naming both outcomes here, the same honesty this workspace's other
/// two-state vocabularies (`Applicability`, `GateRunResponse::Disposition`) already hold
/// to, is clearer than collapsing a real refusal into an empty success.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum ListResponse
{
    /// The board was read.
    Listed
    {
        /// Every item, as the ledger holds them.
        document: LedgerDocument,
        /// The moment the board was read.
        #[serde(serialize_with = "nomos_platform::timestamp_serde::Write_Unix_Seconds")]
        now: Timestamp,
    },
    /// The ledger file could not be read at all.
    Unreadable
    {
        /// What went wrong, as `LedgerError`'s own `Display` renders it.
        cause: String,
    },
}

impl ListResponse
{
    pub(crate) fn From(listed: Result<nomos_work_orchestration::BoardView, nomos_ledger::LedgerError>) -> Self
    {
        return match listed
        {
            Ok(view) => Self::Listed { document: view.document, now: view.now },
            Err(error) => Self::Unreadable { cause: error.to_string() },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::work::tests_support::Scratch_Board;

    #[test]
    fn Test_Handle_Work_List_Should_Read_A_Real_Empty_Board_Not_Refuse_It()
    {
        let directory = Scratch_Board().expect("the temp directory is writable and the scratch ledger is writable");

        let response = Handle_Work_List(&directory);

        let _ignored = std::fs::remove_dir_all(&directory);

        let ListResponse::Listed { document, .. } = response
        else
        {
            // This fixture writes a real, well-formed, empty ledger file, so reading it back
            // must succeed -- reaching `Unreadable` here means the scratch fixture itself is
            // broken, not a runtime condition this test should tolerate.
            panic!("a well-formed, empty ledger reads cleanly");
        };
        assert!(document.items.is_empty());
    }

    #[test]
    fn Test_From_Should_Produce_A_Listed_Response_That_Round_Trips_As_Json()
    {
        let directory = Scratch_Board().expect("the temp directory is writable and the scratch ledger is writable");

        let response = Handle_Work_List(&directory);

        let _ignored = std::fs::remove_dir_all(&directory);

        crate::test_support::Assert_Round_Trips_As_Json(&response, "listed")
            .expect("a listed response serializes and parses back as a tagged object");
    }
}
