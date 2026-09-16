//! [`Handle_Work_Renew`]. Its own response type, [`super::ReservationOutcomeResponse`], lives in
//! [`super::reservation_outcome_response`] -- see that module's own doc for why it cannot live
//! beside this function, [`super::claim::Handle_Work_Claim`] or
//! [`super::take_over::Handle_Work_TakeOver`].

use nomos_work_orchestration::{ClaimRequest, WorkCommand};
use std::path::Path;

use super::{ReservationOutcomeResponse, Run_Empty_Territory_Command};

/// Extends `request`'s already-held lease on the board at `directory`, exactly as `nomos
/// work renew` would, and hands back a JSON-serializable response.
#[must_use]
pub fn Handle_Work_Renew(directory: &Path, request: &ClaimRequest) -> ReservationOutcomeResponse
{
    let outcome = Run_Empty_Territory_Command(directory, WorkCommand::Renew(request.clone()));

    let nomos_work_orchestration::WorkOutcome::Renew(renewed) = outcome
    else
    {
        return ReservationOutcomeResponse::Refused {
            retryable: false,
            cause: super::UNANSWERED_WORK_OUTCOME.to_owned(),
        };
    };

    return ReservationOutcomeResponse::From(renewed);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::work::tests_support::{
        BoardWithAClaimedItem, Claim_Request, Scratch_Board_With_A_Claimed_Item,
    };

    #[test]
    fn Test_Claim_Request_Should_Let_Handle_Work_Renew_Extend_This_Holders_Own_Lease()
    {
        let BoardWithAClaimedItem { directory, id } =
            Scratch_Board_With_A_Claimed_Item("test-holder", i64::from(u32::MAX))
                .expect("the temp directory is writable and the scratch ledger is writable");
        let request = Claim_Request(id.clone(), "test-holder");

        let response = Handle_Work_Renew(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        let ReservationOutcomeResponse::Reserved { reservation } = response
        else
        {
            // This fixture claims the item as "test-holder" and then renews it as the same
            // holder, so a grant is the only correct outcome -- reaching a refusal here means
            // the renewal path itself regressed, not a condition this test should assert
            // around.
            panic!("renewing a claim this holder already has grants a reservation");
        };
        assert_eq!(reservation.item, id);
        assert_eq!(reservation.holder, "test-holder");
    }
}
