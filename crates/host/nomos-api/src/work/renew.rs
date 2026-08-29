//! [`Handle_Work_Renew`]. Its own response type, [`super::WorkReservationResponse`], lives in
//! [`super::work_reservation_response`] -- see that module's own doc for why it cannot live
//! beside this function, [`super::claim::Handle_Work_Claim`] or
//! [`super::take_over::Handle_Work_TakeOver`].

use nomos_work_orchestration::{ClaimRequest, WorkCommand};
use std::path::Path;

use super::{Ledger_At, WorkReservationResponse};

/// Extends `request`'s already-held lease on the board at `directory`, exactly as `nomos
/// work renew` would, and hands back a JSON-serializable response.
#[must_use]
pub fn Handle_Work_Renew(directory: &Path, request: &ClaimRequest) -> WorkReservationResponse
{
    use nomos_ledger::Territory;
    use nomos_platform_std::StdProcessLauncher;

    let mut ledger = Ledger_At(directory);

    let outcome = nomos_work_orchestration::Run(
        &WorkCommand::Renew(request.clone()),
        &mut ledger,
        &StdProcessLauncher,
        || Territory::Of_Files(std::iter::empty::<String>()),
    );

    let nomos_work_orchestration::WorkOutcome::Renew(renewed) = outcome
    else
    {
        // rust-panic: allow: Run's own contract guarantees it returns the WorkOutcome variant
        // naming the WorkCommand it was given -- any other outcome means Run itself is broken.
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return WorkReservationResponse::From(renewed);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::work::tests_support::Scratch_Board_With_A_Claimed_Item;

    #[test]
    fn Test_Renewing_This_Holders_Own_Claim_Should_Extend_The_Lease()
    {
        let (directory, id) = Scratch_Board_With_A_Claimed_Item("test-holder", i64::from(u32::MAX));
        let request = ClaimRequest { item: id.clone(), holder: "test-holder".to_owned(), lease: std::time::Duration::from_secs(3600) };

        let response = Handle_Work_Renew(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkReservationResponse::Reserved { reservation } = response
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
