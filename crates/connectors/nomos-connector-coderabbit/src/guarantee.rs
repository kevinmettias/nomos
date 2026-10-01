//! What this provider offers, and at what guarantee.

use nomos_cap_review_finding::{Capability, CONTRACT_VERSION};
#[cfg(test)]
use nomos_cap_review_finding::Ceiling;
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

/// This provider's own name.
///
/// Named for the vendor whose judgment the fact represents: this provider reads the
/// record through `gh api`, GitHub's own CLI, but the judgment reported -- that this file
/// and line have this severity and category -- is `CodeRabbit`'s, not GitHub's. GitHub is
/// the transport; `CodeRabbit` is the authority the resulting `Observed` fact is a claim
/// about, which is why this provider's own name and
/// [`crate::translation::Translate_Review_Comment`]'s `external_system` field both read
/// `"coderabbit"` rather than `"github"`.
pub const PROVIDER: &str = "nomos.connector.coderabbit";

/// What this provider claims, on every axis.
///
/// [`FactVariant::RuntimeObserved`] at the ceiling -- `nomos_cap_review_finding::Ceiling`'s own doc gives
/// the reason, and this provider meets it exactly: `gh api` against one comment id is a
/// live read of GitHub's current record for it, not a cached or derived one.
///
/// Soundness [`Assurance::Sound`]: every canonical field this provider reports is either a
/// field `gh`'s own JSON response formally carries for this endpoint, or a value read from
/// a portion of the response `CodeRabbit`'s own comment body explicitly version-marks
/// ([`crate::translation`]'s module doc gives the full accounting) -- translated by
/// [`crate::translation::Translate_Review_Comment`] with no field this reader cannot point
/// to a real backing for, and a refusal rather than a guess wherever it cannot.
///
/// Completeness [`Assurance::Unknown`], deliberately not `Sound` -- this provider reads
/// `id`, `user.login`, `path`, `line`/`original_line`, `html_url`, and a comment's category
/// and severity, not `CodeRabbit`'s full comment vocabulary (effort classification, thread
/// resolution state) or the diff hunk itself. A deliberately curated field selection
/// bounds what is canonicalized here, the same "no floor no provider can honestly meet"
/// reasoning this workspace's other real providers already give for their own curated
/// selections.
///
/// [`IncrementalGranularity::None`]: this provider implements no caching, `ETag`, or
/// webhook-driven refresh. Every call to [`crate::provider::Materialize_Review_Comment`]
/// is a full, fresh `gh api` invocation.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(
        FactVariant::RuntimeObserved,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::None,
    );
}

/// This provider's offer against [`nomos_cap_review_finding::Capability_Contract`].
#[must_use]
pub fn Provider_Offer() -> ProviderOffer
{
    return ProviderOffer {
        provider: ProviderId::New(PROVIDER),
        capability: Capability(),
        version: CONTRACT_VERSION,
        guarantee: Declared_Guarantee(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The offer must satisfy the contract's own ceiling -- `contract.ceiling.
    /// Satisfies(&offer.guarantee)` is the exact check `Registry::Offer` runs at
    /// composition time, asserted here in the same direction so a future weakening of
    /// either constant is caught beside the constants rather than only at whatever
    /// composition root happens to run first.
    #[test]
    fn Test_The_Ceiling_Should_Satisfy_The_Declared_Guarantee()
    {
        assert!(Ceiling().Satisfies(&Declared_Guarantee()));
    }
}
