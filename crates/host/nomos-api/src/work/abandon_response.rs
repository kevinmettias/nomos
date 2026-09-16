//! [`Handle_Work_Abandon`] and its own [`AbandonResponse`].

use nomos_ledger::ClaimRefusal;
use nomos_work_orchestration::{EndingRequest, WorkCommand};
use serde::Serialize;
use std::path::Path;

/// Releases `request`'s claim on the board at `directory` without finishing it, exactly as
/// `nomos work abandon` would, and hands back a JSON-serializable response.
#[must_use]
pub fn Handle_Work_Abandon(directory: &Path, request: &EndingRequest) -> AbandonResponse
{
    let outcome = super::Run_Empty_Territory_Command(directory, WorkCommand::Abandon(request.clone()));

    let nomos_work_orchestration::WorkOutcome::Abandon(released) = outcome
    else
    {
        return AbandonResponse::Refused {
            retryable: false,
            cause: super::UNANSWERED_WORK_OUTCOME.to_owned(),
        };
    };

    return AbandonResponse::From(released);
}

/// A claim released, or the refusal that kept it held, in a shape `serde_json` can hand
/// across a wire.
///
/// Not shared with `crate::work::decline_response::DeclineResponse`, even though both wrap
/// `Result<(), ClaimRefusal>` today: `nomos_work_orchestration::EndingRequest`'s own doc says
/// `abandon` and `decline` "stay two verbs and two ledger calls" past sharing an argument
/// shape, and `crates/host/nomos-cli/src/work.rs`'s own dispatch keeps their rendering apart
/// the same way.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum AbandonResponse
{
    /// The claim was given up; the item is claimable again.
    Abandoned,
    /// The claim could not be released.
    Refused
    {
        /// What went wrong, as `ClaimRefusal::Describe` renders it.
        cause: String,
        /// Whether retrying later might succeed -- `ClaimRefusal::Is_Retryable`.
        retryable: bool,
    },
}

impl AbandonResponse
{
    pub(crate) fn From(result: Result<(), ClaimRefusal>) -> Self
    {
        return match result
        {
            Ok(()) => Self::Abandoned,
            Err(refusal) => Self::Refused { retryable: refusal.Is_Retryable(), cause: refusal.Describe() },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::work::tests_support::{
        BoardWithAClaimedItem, Ending_Request, Scratch_Board_With_A_Claimed_Item,
        With_A_Board_Somebody_Else_Holds,
    };

    #[test]
    fn Test_Handle_Work_Abandon_And_Scratch_Board_With_A_Claimed_Item_Should_Release_A_Real_Claim_This_Holder_Actually_Has()
    {
        let BoardWithAClaimedItem { directory, id } =
            Scratch_Board_With_A_Claimed_Item("test-holder", i64::from(u32::MAX))
                .expect("the temp directory is writable and the scratch ledger is writable");
        let request = Ending_Request(id, "test fixture");

        let response = Handle_Work_Abandon(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        assert!(matches!(response, AbandonResponse::Abandoned), "{response:?}");
    }

    #[test]
    fn Test_From_Should_Refuse_And_Mark_Retryable_A_Claim_A_Different_Holder_Actually_Has()
    {
        let response = With_A_Board_Somebody_Else_Holds("test fixture", Handle_Work_Abandon);

        let AbandonResponse::Refused { retryable, cause } = response
        else
        {
            // The board holds the item as "someone-else" and this verb reaches it as
            // "test-holder", so a real refusal is the only correct outcome -- reaching any
            // other variant means the holder check itself stopped enforcing, not a condition
            // this test should assert around.
            panic!("a holder abandoning a claim somebody else actually has is a real refusal");
        };
        assert!(retryable, "{cause}");
    }
}
