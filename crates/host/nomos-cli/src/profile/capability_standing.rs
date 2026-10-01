//! One declared capability, and what this host establishes about answering it.

use super::{OfferStanding, ProviderStanding};

/// A capability this workspace declares, with every provider that offers it and what this
/// host settles about each.
///
/// Every offer rather than the best one, and the reason is measured rather than tidy.
/// `nomos.cap.dependency.edges` carries two: `nomos-lang-go-modules`, which reads `go.mod` as
/// text and needs nothing, and `nomos-lang-rust-cargo`, which runs `cargo metadata`. A report
/// that showed only the strongest would tell a reader with a Rust workspace and no `cargo`
/// that dependency edges are available here, which is true of the capability and false of
/// their repository. So the verdict is a summary of rows a reader can see, never a
/// replacement for them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CapabilityStanding
{
    /// The capability's declared identity, as the registry spells it.
    pub(crate) capability: String,
    /// Every provider offering it, in the order the registry holds them.
    pub(crate) offers: Vec<OfferStanding>,
}

impl CapabilityStanding
{
    /// The strongest standing any of this capability's offers reaches, which is whether a
    /// provider on this host can answer it at all.
    ///
    /// [`ProviderStanding::NothingOffered`] when there are no offers: a declared capability
    /// nobody offers is settled by the registry alone and needs no host to decide it.
    #[must_use]
    pub(crate) fn Summary(&self) -> ProviderStanding
    {
        return self
            .offers
            .iter()
            .map(|offer| return offer.standing.clone())
            .max_by_key(ProviderStanding::Rank)
            .unwrap_or(ProviderStanding::NothingOffered);
    }
}
