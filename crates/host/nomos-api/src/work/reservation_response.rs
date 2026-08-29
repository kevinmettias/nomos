//! [`ReservationResponse`], carried only by [`super::work_reservation_response::WorkReservationResponse::
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
