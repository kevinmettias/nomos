//! Somebody holding an item, until when.

// Why a claim was refused, beneath the claim it would have been.
#[path = "claim/refusal.rs"]
mod refusal;

pub use refusal::{Refusal as ClaimRefusal, Layer as RefusalLayer};

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
    #[serde(with = "nomos_platform::timestamp_serde")]
    pub acquired_at: Timestamp,
    /// When it lapses if not renewed.
    #[serde(with = "nomos_platform::timestamp_serde")]
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
    /// installs the new claim and [`LedgerItem::Try_Replace_Lapsed_Claim`] moves the claim it
    /// replaced onto [`LedgerItem::displaced`], so what a lapse recorded survives the thing
    /// that ends it.
    #[must_use]
    pub fn Has_Lapsed(&self, now: Timestamp) -> bool
    {
        return now > self.lease_expires_at;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// When the claim under test was taken.
    const CLAIM_TAKEN_AT_SECONDS: i64 = 1_000;

    /// When that claim's lease runs out, which is the instant the boundary case turns on.
    const LEASE_ENDS_AT_SECONDS: i64 = 2_000;

    /// One second past the lease, where the claim has lapsed.
    const ONE_SECOND_PAST_THE_LEASE: i64 = 2_001;

    /// The boundary case: a lease expiring exactly now has not yet lapsed, or a holder
    /// renewing at the moment of expiry would race against being displaced.
    #[test]
    fn Test_Has_Lapsed_Should_Be_False_On_The_Expiry_Second_And_True_After()
    {
        let claim = Claim {
            holder: "agent-a".to_owned(),
            acquired_at: Timestamp::From_Unix_Seconds(CLAIM_TAKEN_AT_SECONDS),
            lease_expires_at: Timestamp::From_Unix_Seconds(LEASE_ENDS_AT_SECONDS),
        };

        assert!(!claim.Has_Lapsed(Timestamp::From_Unix_Seconds(LEASE_ENDS_AT_SECONDS)));
        assert!(claim.Has_Lapsed(Timestamp::From_Unix_Seconds(ONE_SECOND_PAST_THE_LEASE)));
    }
}
