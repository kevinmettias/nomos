//! [`Handle_Work_Abandon`] and its own [`WorkAbandonResponse`].

use nomos_ledger::{ClaimRefusal, Territory};
use nomos_platform_std::StdProcessLauncher;
use nomos_work_orchestration::{EndingRequest, WorkCommand};
use serde::Serialize;
use std::path::Path;

use super::Ledger_At;

/// Releases `request`'s claim on the board at `directory` without finishing it, exactly as
/// `nomos work abandon` would, and hands back a JSON-serializable response.
#[must_use]
pub fn Handle_Work_Abandon(directory: &Path, request: &EndingRequest) -> WorkAbandonResponse
{
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
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return WorkAbandonResponse::From(released);
}

/// A claim released, or the refusal that kept it held, in a shape `serde_json` can hand
/// across a wire.
///
/// Not shared with `crate::work::decline::WorkDeclineResponse`, even though both wrap
/// `Result<(), ClaimRefusal>` today: `nomos_work_orchestration::EndingRequest`'s own doc says
/// `abandon` and `decline` "stay two verbs and two ledger calls" past sharing an argument
/// shape, and `crates/host/nomos-cli/src/work.rs`'s own dispatch keeps their rendering apart
/// the same way.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum WorkAbandonResponse
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

impl WorkAbandonResponse
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
    use crate::work::tests_support::Scratch_Board_With_A_Claimed_Item;

    #[test]
    fn Test_Abandoning_A_Real_Claim_This_Holder_Actually_Has_Should_Release_It()
    {
        let (directory, id) = Scratch_Board_With_A_Claimed_Item("test-holder", i64::from(u32::MAX));
        let request = EndingRequest { item: id, holder: "test-holder".to_owned(), reason: "test fixture".to_owned() };

        let response = Handle_Work_Abandon(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        assert!(matches!(response, WorkAbandonResponse::Abandoned), "{response:?}");
    }

    #[test]
    fn Test_Abandoning_A_Claim_A_Different_Holder_Actually_Has_Should_Be_Refused_And_Retryable()
    {
        let (directory, id) = Scratch_Board_With_A_Claimed_Item("someone-else", i64::from(u32::MAX));
        let request = EndingRequest { item: id, holder: "test-holder".to_owned(), reason: "test fixture".to_owned() };

        let response = Handle_Work_Abandon(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkAbandonResponse::Refused { retryable, cause } = response
        else
        {
            panic!("a holder abandoning a claim somebody else actually has is a real refusal");
        };
        assert!(retryable, "{cause}");
    }
}
