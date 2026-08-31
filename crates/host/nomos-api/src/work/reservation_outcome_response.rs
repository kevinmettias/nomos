//! [`ReservationOutcomeResponse`], shared by [`super::claim::Handle_Work_Claim`],
//! [`super::renew::Handle_Work_Renew`] and [`super::take_over::Handle_Work_TakeOver`].
//!
//! One shared response type for the three of them rather than three that could only ever
//! agree by discipline -- the exact reason `nomos_work_orchestration::ClaimRequest` is
//! already one shared request type for these three verbs (its own doc says so), and the same
//! division `crates/host/nomos-cli/src/work.rs`'s own dispatch already draws: one
//! `Report_Claim` renders all three outcomes today. The three handler functions stay in
//! three separate files of their own -- `Handle_Work_Claim`, `Handle_Work_Renew` and
//! `Handle_Work_TakeOver` share the `Handle_Work` prefix, and three sharing one file would be
//! exactly the cluster `check-responsibility-extraction` looks for.

use nomos_ledger::{ClaimRefusal, Reservation};
use serde::Serialize;

use super::ReservationResponse;

/// A granted or refused reservation, in a shape `serde_json` can hand across a wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum ReservationOutcomeResponse
{
    /// The reservation was granted.
    Reserved
    {
        /// The reservation itself.
        reservation: ReservationResponse,
    },
    /// The reservation was refused.
    Refused
    {
        /// What went wrong, as `ClaimRefusal::Describe` renders it.
        cause: String,
        /// Whether retrying later might succeed -- `ClaimRefusal::Is_Retryable`, the same
        /// signal `crates/host/nomos-cli/src/work/report.rs`'s own `Code_For` already
        /// branches an exit code on, handed to a wire caller directly since it has no
        /// process exit code to read.
        retryable: bool,
    },
}

impl ReservationOutcomeResponse
{
    pub(crate) fn From(result: Result<Reservation, ClaimRefusal>) -> Self
    {
        return match result
        {
            Ok(reservation) => Self::Reserved { reservation: ReservationResponse::From(reservation) },
            Err(refusal) => Self::Refused { retryable: refusal.Is_Retryable(), cause: refusal.Describe() },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_ledger::ItemId;
    use nomos_platform::Timestamp;

    #[test]
    fn Test_From_Should_Map_A_Granted_Reservation_Into_Reserved()
    {
        let reservation = Reservation {
            item: ItemId::New("SCRATCH-OUTCOME-RESERVED"),
            holder: "test-holder".to_owned(),
            expires_at: Timestamp::From_Unix_Seconds(42),
        };

        let response = ReservationOutcomeResponse::From(Ok(reservation));

        assert!(matches!(response, ReservationOutcomeResponse::Reserved { .. }), "{response:?}");
    }

    #[test]
    fn Test_From_Should_Map_A_Held_By_Refusal_Into_Refused_And_Retryable()
    {
        let refusal = ClaimRefusal::HeldBy {
            holder: "someone-else".to_owned(),
            until: Timestamp::From_Unix_Seconds(2_000),
            item: ItemId::New("SCRATCH-OUTCOME-HELD"),
        };

        let response = ReservationOutcomeResponse::From(Err(refusal));

        let ReservationOutcomeResponse::Refused { retryable, .. } = response
        else
        {
            // ReservationOutcomeResponse::From's own match (above) maps every Err into
            // Refused unconditionally -- reaching else here means that mapping itself
            // regressed, not a condition this test should assert around.
            panic!("a HeldBy refusal must map to Refused")
        };
        assert!(retryable);
    }
}
