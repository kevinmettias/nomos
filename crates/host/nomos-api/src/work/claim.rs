//! [`Handle_Work_Claim`]. Its own response type, [`super::ReservationOutcomeResponse`], lives in
//! [`super::reservation_outcome_response`] -- shared with [`super::renew::Handle_Work_Renew`] and
//! [`super::take_over::Handle_Work_TakeOver`], so it cannot live beside any one of the three
//! without three-in-one-file becoming exactly the `Handle_Work` cluster this split exists to
//! avoid.

use nomos_work_orchestration::{ClaimRequest, WorkCommand};
use std::path::Path;

use super::{ReservationOutcomeResponse, Run_Reservation_Command};

/// Grants `request` a reservation on the board at `directory`, exactly as `nomos work claim`
/// would, and hands back a JSON-serializable response.
#[must_use]
pub fn Handle_Work_Claim(directory: &Path, request: &ClaimRequest) -> ReservationOutcomeResponse
{
    let outcome = Run_Reservation_Command(directory, WorkCommand::Claim(request.clone()));

    let nomos_work_orchestration::WorkOutcome::Claim(claimed) = outcome
    else
    {
        // rust-panic: allow: Run's own contract guarantees it returns the WorkOutcome variant
        // naming the WorkCommand it was given -- any other outcome means Run itself is broken.
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return ReservationOutcomeResponse::From(claimed);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::work::tests_support::{
        Claim_Request, Scratch_Board_With_A_Claimable_Item, Scratch_Board_With_A_Held_Territory_Conflict,
    };

    #[test]
    fn Test_Handle_Work_Claim_Should_Grant_A_Reservation_For_A_Real_Ready_Item()
    {
        let (directory, id) = Scratch_Board_With_A_Claimable_Item();
        let request = Claim_Request(id.clone(), "test-holder");

        let response = Handle_Work_Claim(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        let ReservationOutcomeResponse::Reserved { reservation } = response
        else
        {
            // This fixture builds a board with a real, unclaimed Ready item and no territory
            // conflict, so a grant is the only correct outcome -- reaching a refusal here
            // means the claim path itself regressed, not a condition this test should assert
            // around.
            panic!("an unclaimed Ready item grants a reservation");
        };
        assert_eq!(reservation.item, id);
        assert_eq!(reservation.holder, "test-holder");
    }

    #[test]
    fn Test_Scratch_Board_With_A_Held_Territory_Conflict_Should_Refuse_A_Claim_And_Mark_It_Retryable()
    {
        let (directory, id) = Scratch_Board_With_A_Held_Territory_Conflict();
        let request = Claim_Request(id, "test-holder");

        let response = Handle_Work_Claim(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        let ReservationOutcomeResponse::Refused { retryable, cause } = response
        else
        {
            // This fixture builds a board where another live claim already holds the same
            // territory, so a refusal is the only correct outcome -- reaching a grant here
            // means the territory-conflict check itself stopped enforcing, not a condition
            // this test should assert around.
            panic!("ground another live claim holds is a real refusal, not a grant");
        };
        assert!(retryable, "{cause}");
    }

    #[test]
    fn Test_From_Should_Produce_A_Reserved_Response_That_Round_Trips_As_Json()
    {
        let (directory, id) = Scratch_Board_With_A_Claimable_Item();
        let request = Claim_Request(id, "test-holder");

        let response = Handle_Work_Claim(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        crate::test_support::Assert_Round_Trips_As_Json(&response, "reserved");
    }
}
