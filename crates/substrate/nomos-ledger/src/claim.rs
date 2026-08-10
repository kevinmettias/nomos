//! Somebody holding an item, until when.

use serde::Deserialize;
use serde::Serialize;
use nomos_platform::Timestamp;
/// A held claim on an item.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim
{
    /// Who holds it.
    pub holder: String,
    /// When they took it.
    pub acquired_at: Timestamp,
    /// When it lapses if not renewed.
    pub lease_expires_at: Timestamp,
}

impl Claim
{
    /// Whether this claim has lapsed as of `now`.
    ///
    /// A lapsed claim does not release itself. It stops excluding — every *other* item is
    /// claimable again, which is what `MAXIMUM_LEASE` exists for — and it stays visible so
    /// that a person can see the work was abandoned rather than never started.
    ///
    /// It does not let the next agent *claim* this item, and this comment said it did until
    /// `P10-LAPSE-BRICKS` measured it. The item stays `Claimed`, and `Claim_Refusal` refuses
    /// a plain claim on it — with [`crate::ClaimRefusal::Lapsed`], which names the holder
    /// whose lease ran out and the remedy. That refusal is deliberate as of `OD-LEDGER-009`
    /// rather than merely true: a claim overwrites `claim`, and `claim` is the only thing
    /// recording that the work was ever started, which `OD-LEDGER-006` decided must survive.
    ///
    /// Taking a lapsed item over is therefore a different operation from claiming a free
    /// one, and `OD-LEDGER-012` is where it became one: [`crate::FileLedger::Take_Over`]
    /// installs the new claim and [`LedgerItem::Replace_Lapsed_Claim`] moves the claim it
    /// replaced onto [`LedgerItem::displaced`], so what a lapse recorded survives the thing
    /// that ends it.
    #[must_use]
    pub fn Has_Lapsed(&self, now: Timestamp) -> bool
    {
        return now > self.lease_expires_at;
    }
}
