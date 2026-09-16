//! [`Handle_Work_Decline`] and its own [`DeclineResponse`].

use nomos_ledger::ClaimRefusal;
use nomos_work_orchestration::{EndingRequest, WorkCommand};
use serde::Serialize;
use std::path::Path;

/// Ends `request`'s item on the board at `directory` as not being work, exactly as `nomos
/// work decline` would, and hands back a JSON-serializable response.
#[must_use]
pub fn Handle_Work_Decline(directory: &Path, request: &EndingRequest) -> DeclineResponse
{
    let outcome = super::Run_Empty_Territory_Command(directory, WorkCommand::Decline(request.clone()));

    let nomos_work_orchestration::WorkOutcome::Decline { declined, .. } = outcome
    else
    {
        return DeclineResponse::Refused {
            retryable: false,
            cause: super::UNANSWERED_WORK_OUTCOME.to_owned(),
        };
    };

    return DeclineResponse::From(declined);
}

/// An item ended, or the refusal that kept it open, in a shape `serde_json` can hand across
/// a wire. Not shared with `crate::work::abandon_response::AbandonResponse`; see that type's own
/// doc for why.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum DeclineResponse
{
    /// The item was ended; it is no longer workable.
    Declined,
    /// The item could not be ended.
    Refused
    {
        /// What went wrong, as `ClaimRefusal::Describe` renders it.
        cause: String,
        /// Whether retrying later might succeed -- `ClaimRefusal::Is_Retryable`.
        retryable: bool,
    },
}

impl DeclineResponse
{
    pub(crate) fn From(result: Result<(), ClaimRefusal>) -> Self
    {
        return match result
        {
            Ok(()) => Self::Declined,
            Err(refusal) => Self::Refused { retryable: refusal.Is_Retryable(), cause: refusal.Describe() },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::work::tests_support::{
        BoardWithAClaimableItem, Ending_Request, Scratch_Board_With_A_Claimable_Item,
        With_A_Board_Somebody_Else_Holds,
    };

    #[test]
    fn Test_Handle_Work_Decline_Should_End_A_Real_Unclaimed_Ready_Item()
    {
        let BoardWithAClaimableItem { directory, id } =
            Scratch_Board_With_A_Claimable_Item()
                .expect("the temp directory is writable and the scratch ledger is writable");
        let request = Ending_Request(id, "superseded");

        let response = Handle_Work_Decline(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        assert!(matches!(response, DeclineResponse::Declined), "{response:?}");
    }

    #[test]
    fn Test_Ending_Request_Should_Be_Refused_And_Retryable_When_A_Real_Active_Claim_Still_Holds_The_Item()
    {
        let response = With_A_Board_Somebody_Else_Holds("superseded", Handle_Work_Decline);

        let DeclineResponse::Refused { retryable, cause } = response
        else
        {
            // The board holds the item as "someone-else" and leaves that claim active, so
            // reaching it as "test-holder" must refuse -- reaching `Declined` here means the
            // active-claim check itself stopped enforcing, not a condition this test should
            // assert around.
            panic!("an item a real active claim still holds is a real refusal, not an end");
        };
        assert!(retryable, "{cause}");
    }

    #[test]
    fn Test_From_Should_Produce_A_Declined_Response_That_Round_Trips_As_Json()
    {
        let BoardWithAClaimableItem { directory, id } =
            Scratch_Board_With_A_Claimable_Item()
                .expect("the temp directory is writable and the scratch ledger is writable");
        let request = Ending_Request(id, "superseded");

        let response = Handle_Work_Decline(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        crate::test_support::Assert_Round_Trips_As_Json(&response, "declined")
            .expect("a declined response serializes and parses back as a tagged object");
    }
}
