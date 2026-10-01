//! [`ReservationResponse`], carried only by [`super::reservation_outcome_response::ReservationOutcomeResponse::
//! Reserved`].

use nomos_ledger::{ItemId, Reservation};
use nomos_platform::Timestamp;
use serde::Serialize;

/// A serializable twin of [`nomos_ledger::Reservation`], which does not derive `Serialize`.
#[derive(Debug, Serialize)]
pub struct ReservationResponse
{
    /// The item claimed.
    pub item: ItemId,
    /// Who holds it.
    pub holder: String,
    /// When the lease lapses.
    #[serde(serialize_with = "nomos_platform::timestamp_serde::Write_Unix_Seconds")]
    pub expires_at: Timestamp,
}

impl ReservationResponse
{
    pub(crate) fn From(reservation: Reservation) -> Self
    {
        return Self {
            item: reservation.item,
            holder: reservation.holder,
            expires_at: reservation.expires_at,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The fixture lease's expiry, distinct from every other timestamp in this crate's tests
    /// so a `From` that read the wrong field could not still compare equal below.
    const RESERVATION_EXPIRY_UNIX_SECONDS: i64 = 42;

    #[test]
    fn Test_From_Should_Carry_The_Reservations_Item_Holder_And_Expiry()
    {
        let reservation = Reservation {
            item: ItemId::New("SCRATCH-RESERVATION"),
            holder: "test-holder".to_owned(),
            expires_at: Timestamp::From_Unix_Seconds(RESERVATION_EXPIRY_UNIX_SECONDS),
        };

        let response = ReservationResponse::From(reservation.clone());

        assert_eq!(response.item, reservation.item);
        assert_eq!(response.holder, reservation.holder);
        assert_eq!(response.expires_at, reservation.expires_at);
    }
}
