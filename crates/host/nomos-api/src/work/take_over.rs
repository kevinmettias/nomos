//! [`Handle_Work_TakeOver`]. Its own response type, [`super::ReservationOutcomeResponse`], lives
//! in [`super::reservation_outcome_response`] -- see that module's own doc for why it cannot
//! live beside this function, [`super::claim::Handle_Work_Claim`] or
//! [`super::renew::Handle_Work_Renew`].

use nomos_work_orchestration::{ClaimRequest, WorkCommand};
use std::path::Path;

use super::{ReservationOutcomeResponse, Run_Reservation_Command};

/// Takes over `request`'s item on the board at `directory` from a lapsed holder, exactly as
/// `nomos work takeover` would, and hands back a JSON-serializable response.
#[must_use]
pub fn Handle_Work_TakeOver(directory: &Path, request: &ClaimRequest) -> ReservationOutcomeResponse
{
    let outcome = Run_Reservation_Command(directory, WorkCommand::TakeOver(request.clone()));

    let nomos_work_orchestration::WorkOutcome::TakeOver(taken_over) = outcome
    else
    {
        // rust-panic: allow: Run's own contract guarantees it returns the WorkOutcome variant
        // naming the WorkCommand it was given -- any other outcome means Run itself is broken.
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return ReservationOutcomeResponse::From(taken_over);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::work::tests_support::{Claim_Request, Scratch_Board_With_A_Claimed_Item};

    #[test]
    fn Test_Run_Reservation_Command_Should_Let_Handle_Work_TakeOver_Grant_A_Reservation_For_A_Real_Lapsed_Claim()
    {
        let (directory, id) = Scratch_Board_With_A_Claimed_Item("old-holder", 1);
        let request = Claim_Request(id.clone(), "new-holder");

        let response = Handle_Work_TakeOver(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        let ReservationOutcomeResponse::Reserved { reservation } = response
        else
        {
            // This fixture claims the item with a one-second lease and lets it lapse before
            // taking it over as a new holder, so a grant is the only correct outcome --
            // reaching a refusal here means the lapsed-lease check itself stopped enforcing,
            // not a condition this test should assert around.
            panic!("a real lapsed claim is takeable");
        };
        assert_eq!(reservation.item, id);
        assert_eq!(reservation.holder, "new-holder");
    }
}
