//! [`Handle_Work_Claim`]. Its own response type, [`super::WorkReservationResponse`], lives in
//! [`super::work_reservation_response`] -- shared with [`super::renew::Handle_Work_Renew`] and
//! [`super::take_over::Handle_Work_TakeOver`], so it cannot live beside any one of the three
//! without three-in-one-file becoming exactly the `Handle_Work` cluster this split exists to
//! avoid.

use nomos_work_orchestration::{ClaimRequest, WorkCommand};
use std::path::Path;

use super::{Ledger_At, WorkReservationResponse};

/// Grants `request` a reservation on the board at `directory`, exactly as `nomos work claim`
/// would, and hands back a JSON-serializable response.
#[must_use]
pub fn Handle_Work_Claim(directory: &Path, request: &ClaimRequest) -> WorkReservationResponse
{
    use nomos_ledger::Territory;
    use nomos_platform_std::StdProcessLauncher;

    let mut ledger = Ledger_At(directory);

    let outcome = nomos_work_orchestration::Run(
        &WorkCommand::Claim(request.clone()),
        &mut ledger,
        &StdProcessLauncher,
        || Territory::Of_Files(std::iter::empty::<String>()),
    );

    let nomos_work_orchestration::WorkOutcome::Claim(claimed) = outcome
    else
    {
        // rust-panic: allow: Run's own contract guarantees it returns the WorkOutcome variant
        // naming the WorkCommand it was given -- any other outcome means Run itself is broken.
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return WorkReservationResponse::From(claimed);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::work::tests_support::{Scratch_Board_With_A_Claimable_Item, Scratch_Board_With_A_Held_Territory_Conflict};

    #[test]
    fn Test_Claiming_A_Real_Ready_Item_Should_Grant_A_Reservation()
    {
        let (directory, id) = Scratch_Board_With_A_Claimable_Item();
        let request = ClaimRequest { item: id.clone(), holder: "test-holder".to_owned(), lease: std::time::Duration::from_secs(3600) };

        let response = Handle_Work_Claim(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkReservationResponse::Reserved { reservation } = response
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
    fn Test_Claiming_An_Item_Whose_Territory_Another_Live_Claim_Holds_Should_Be_Refused_And_Retryable()
    {
        let (directory, id) = Scratch_Board_With_A_Held_Territory_Conflict();
        let request = ClaimRequest { item: id, holder: "test-holder".to_owned(), lease: std::time::Duration::from_secs(3600) };

        let response = Handle_Work_Claim(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkReservationResponse::Refused { retryable, cause } = response
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
    fn Test_A_Real_Reserved_Response_Should_Round_Trip_As_Json()
    {
        let (directory, id) = Scratch_Board_With_A_Claimable_Item();
        let request = ClaimRequest { item: id, holder: "test-holder".to_owned(), lease: std::time::Duration::from_secs(3600) };

        let response = Handle_Work_Claim(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        let json = serde_json::to_string(&response).expect("a WorkReservationResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized WorkReservationResponse always has this field");

        assert_eq!(outcome, "reserved", "{json}");
    }
}
