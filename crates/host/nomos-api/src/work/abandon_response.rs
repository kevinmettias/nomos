//! [`Handle_Work_Abandon`] and its own [`AbandonResponse`].

use nomos_ledger::{ClaimRefusal, Territory};
use nomos_work_orchestration::{EndingRequest, WorkCommand};
use serde::Serialize;
use std::path::Path;

/// Releases `request`'s claim on the board at `directory` without finishing it, exactly as
/// `nomos work abandon` would, and hands back a JSON-serializable response.
#[must_use]
pub fn Handle_Work_Abandon(directory: &Path, request: &EndingRequest) -> AbandonResponse
{
    use super::Ledger_At;
    use nomos_platform_std::StdProcessLauncher;

    let mut ledger = Ledger_At(directory);

    let outcome = nomos_work_orchestration::Run(
        &WorkCommand::Abandon(request.clone()),
        &mut ledger,
        &StdProcessLauncher,
        || Territory::Of_Files(std::iter::empty::<String>()),
    );

    let nomos_work_orchestration::WorkOutcome::Abandon(released) = outcome
    else
    {
        // rust-panic: allow: Run's own contract guarantees it returns the WorkOutcome variant
        // naming the WorkCommand it was given -- any other outcome means Run itself is broken.
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
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
    use crate::work::tests_support::{Ending_Request, Scratch_Board_With_A_Claimed_Item};

    #[test]
    fn Test_Abandoning_A_Real_Claim_This_Holder_Actually_Has_Should_Release_It()
    {
        let (directory, id) = Scratch_Board_With_A_Claimed_Item("test-holder", i64::from(u32::MAX));
        let request = Ending_Request(id, "test fixture");

        let response = Handle_Work_Abandon(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        assert!(matches!(response, AbandonResponse::Abandoned), "{response:?}");
    }

    #[test]
    fn Test_Abandoning_A_Claim_A_Different_Holder_Actually_Has_Should_Be_Refused_And_Retryable()
    {
        let (directory, id) = Scratch_Board_With_A_Claimed_Item("someone-else", i64::from(u32::MAX));
        let request = Ending_Request(id, "test fixture");

        let response = Handle_Work_Abandon(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        let AbandonResponse::Refused { retryable, cause } = response
        else
        {
            // This fixture claims the item as "someone-else" and then abandons it as
            // "test-holder", so a real refusal is the only correct outcome -- reaching any
            // other variant means the holder check itself stopped enforcing, not a condition
            // this test should assert around.
            panic!("a holder abandoning a claim somebody else actually has is a real refusal");
        };
        assert!(retryable, "{cause}");
    }
}
