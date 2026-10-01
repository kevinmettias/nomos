//! A territory enlarged by the holder who found it short, and what it added.

use nomos_platform::Timestamp;
use serde::{Deserialize, Serialize};

/// One enlargement of an item's territory.
///
/// A reservation is authored before the change it reserves has been attempted, so it is a
/// prediction. `OD-LEDGER-039` decided that a live holder may correct one rather than lose the
/// item, and that the correction is kept. This is what is kept.
///
/// # Why the paths added and not the territory after
///
/// The territory after a widening is already on the item, so storing it here would be a second
/// shape that can drift from [`crate::Territory`] — the argument [`crate::LedgerItem::displaced`]
/// makes for keeping the [`crate::Claim`] itself rather than a summary of it. What cannot be
/// recovered from the item is which paths this particular widening contributed, and that is the
/// only thing here that exists nowhere else.
///
/// It is also the thing the measurement needs. How often a predicted cone escapes its
/// reservation, and by how much, is answerable from these rows and from nothing else on the
/// board: a decline carries a holder, a timestamp and free prose, so the same question asked of
/// declines yields a figure nobody can check.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Widening
{
    /// Who widened it.
    ///
    /// The holder of the live claim at the moment it happened, which is the only party
    /// `Widen_Refusal` admits. Kept beside the paths rather than inferred from
    /// [`crate::LedgerItem::claim`], because a later takeover replaces that claim and the
    /// question this row answers is who found the reservation short.
    pub holder: String,
    /// The paths this widening added, in the order they were given.
    ///
    /// Only the ones that were not already reserved. A path the territory already held
    /// contributes nothing and recording it would make the history overstate the escape,
    /// which is the one number these rows exist to carry honestly.
    pub added: Vec<String>,
    /// When it happened.
    #[serde(serialize_with = "nomos_platform::timestamp_serde::Write_Unix_Seconds")]
    #[serde(deserialize_with = "nomos_platform::timestamp_serde::Read_Unix_Seconds")]
    pub widened_at: Timestamp,
}
