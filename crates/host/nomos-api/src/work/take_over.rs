//! [`Handle_Work_TakeOver`]. Its own response type, [`super::WorkReservationResponse`], lives
//! in [`super::reservation`] -- see that module's own doc for why it cannot live beside this
//! function, [`super::claim::Handle_Work_Claim`] or [`super::renew::Handle_Work_Renew`].

use nomos_ledger::Territory;
use nomos_platform_std::StdProcessLauncher;
use nomos_work_orchestration::{ClaimRequest, WorkCommand};
use std::path::Path;

use super::{Ledger_At, WorkReservationResponse};

/// Takes over `request`'s item on the board at `directory` from a lapsed holder, exactly as
/// `nomos work takeover` would, and hands back a JSON-serializable response.
#[must_use]
pub fn Handle_Work_TakeOver(directory: &Path, request: &ClaimRequest) -> WorkReservationResponse
{
    let mut ledger = Ledger_At(directory);

    let outcome = nomos_work_orchestration::Run(
        &WorkCommand::TakeOver(request.clone()),
        &mut ledger,
        &StdProcessLauncher,
        || Territory::Of_Files(std::iter::empty::<String>()),
    );

    let nomos_work_orchestration::WorkOutcome::TakeOver(taken_over) = outcome
    else
    {
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return WorkReservationResponse::From(taken_over);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::work::tests_support::Scratch_Board_With_A_Claimed_Item;

    #[test]
    fn Test_Taking_Over_A_Real_Lapsed_Claim_Should_Grant_A_Reservation()
    {
        let (directory, id) = Scratch_Board_With_A_Claimed_Item("old-holder", 1);
        let request = ClaimRequest { item: id.clone(), holder: "new-holder".to_owned(), lease: std::time::Duration::from_secs(3600) };

        let response = Handle_Work_TakeOver(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkReservationResponse::Reserved { reservation } = response
        else
        {
            panic!("a real lapsed claim is takeable");
        };
        assert_eq!(reservation.item, id);
        assert_eq!(reservation.holder, "new-holder");
    }
}
