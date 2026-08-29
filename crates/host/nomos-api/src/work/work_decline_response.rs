//! [`Handle_Work_Decline`] and its own [`WorkDeclineResponse`].

use nomos_ledger::{ClaimRefusal, Territory};
use nomos_work_orchestration::{EndingRequest, WorkCommand};
use serde::Serialize;
use std::path::Path;

/// Ends `request`'s item on the board at `directory` as not being work, exactly as `nomos
/// work decline` would, and hands back a JSON-serializable response.
#[must_use]
pub fn Handle_Work_Decline(directory: &Path, request: &EndingRequest) -> WorkDeclineResponse
{
    use super::Ledger_At;
    use nomos_platform_std::StdProcessLauncher;

    let mut ledger = Ledger_At(directory);

    let outcome = nomos_work_orchestration::Run(
        &WorkCommand::Decline(request.clone()),
        &mut ledger,
        &StdProcessLauncher,
        || Territory::Of_Files(std::iter::empty::<String>()),
    );

    let nomos_work_orchestration::WorkOutcome::Decline(declined) = outcome
    else
    {
        // rust-panic: allow: Run's own contract guarantees it returns the WorkOutcome variant
        // naming the WorkCommand it was given -- any other outcome means Run itself is broken.
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return WorkDeclineResponse::From(declined);
}

/// An item ended, or the refusal that kept it open, in a shape `serde_json` can hand across
/// a wire. Not shared with `crate::work::work_abandon_response::WorkAbandonResponse`; see that type's own
/// doc for why.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum WorkDeclineResponse
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

impl WorkDeclineResponse
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
    use crate::work::tests_support::{Scratch_Board_With_A_Claimable_Item, Scratch_Board_With_A_Claimed_Item};

    #[test]
    fn Test_Declining_A_Real_Unclaimed_Ready_Item_Should_End_It()
    {
        let (directory, id) = Scratch_Board_With_A_Claimable_Item();
        let request = EndingRequest { item: id, holder: "test-holder".to_owned(), reason: "superseded".to_owned() };

        let response = Handle_Work_Decline(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        assert!(matches!(response, WorkDeclineResponse::Declined), "{response:?}");
    }

    #[test]
    fn Test_Declining_An_Item_A_Real_Active_Claim_Still_Holds_Should_Be_Refused_And_Retryable()
    {
        let (directory, id) = Scratch_Board_With_A_Claimed_Item("someone-else", i64::from(u32::MAX));
        let request = EndingRequest { item: id, holder: "test-holder".to_owned(), reason: "superseded".to_owned() };

        let response = Handle_Work_Decline(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkDeclineResponse::Refused { retryable, cause } = response
        else
        {
            // This fixture claims the item as "someone-else" and leaves that claim active, so
            // declining it as "test-holder" must refuse -- reaching `Declined` here means the
            // active-claim check itself stopped enforcing, not a condition this test should
            // assert around.
            panic!("an item a real active claim still holds is a real refusal, not an end");
        };
        assert!(retryable, "{cause}");
    }

    #[test]
    fn Test_A_Real_Declined_Response_Should_Round_Trip_As_Json()
    {
        let (directory, id) = Scratch_Board_With_A_Claimable_Item();
        let request = EndingRequest { item: id, holder: "test-holder".to_owned(), reason: "superseded".to_owned() };

        let response = Handle_Work_Decline(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        let json = serde_json::to_string(&response).expect("a WorkDeclineResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized WorkDeclineResponse always has this field");

        assert_eq!(outcome, "declined", "{json}");
    }
}
