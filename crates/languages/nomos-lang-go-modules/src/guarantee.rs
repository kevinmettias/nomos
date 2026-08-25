//! What this provider offers, and at what guarantee.

use nomos_cap_dependency::{Capability, CONTRACT_VERSION};
#[cfg(test)]
use nomos_cap_dependency::Ceiling;
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

/// This provider's own name.
///
/// Named for the workspace shape it reads, the same way `nomos.lang.rust.cargo` is named
/// for the tool it runs: a future provider answering the same capability from Go's own
/// `go list -m` output, once a real toolchain is a dependency this workspace is willing
/// to take on, would need to disagree with this name honestly.
pub const PROVIDER: &str = "nomos.lang.go.modules";

/// What this provider claims, on every axis.
///
/// [`FactVariant::SemanticallyResolved`] at the ceiling: a `require` line's module path is
/// already Go's own globally unique identity for the module it names, the same standard
/// this crate's own module doc holds itself to — there is no second, weaker reading this
/// provider settled for.
///
/// Soundness [`Assurance::Sound`]: every edge this provider reports is a `require` line
/// this workspace's own `go.mod` files actually declare, matched against a module path
/// another workspace member's own `go.mod` actually states; nothing here infers an edge
/// from context.
///
/// Completeness [`Assurance::Unknown`], deliberately not `Sound` — unlike
/// `nomos-lang-rust-cargo`'s `--all-features` invocation, this provider does not resolve a
/// `replace` directive (its own module doc says why), so a workspace that uses one has a
/// real edge this provider would not report. A floor this provider cannot honestly meet
/// is not caution; it is the declared gap its own module doc already names.
///
/// [`IncrementalGranularity::Project`]: a module's whole `go.mod` is one declaration, and
/// there is no slice of "which requirement changed" below re-reading the file as a whole
/// — the same reasoning `nomos-lang-rust-cargo`'s own guarantee gives for its identical
/// axis.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::Project,
    );
}

/// This provider's offer against [`nomos_cap_dependency::Capability_Contract`].
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

    /// The offer must satisfy the contract's own ceiling — the property
    /// `Registry::Offer` checks at composition time (`nomos_capability::registry::
    /// declaring::Offer`), asserted here in the same direction so a future weakening of
    /// either constant is caught beside the constants rather than only at whatever
    /// composition root happens to run first. This provider's own completeness
    /// (`Assurance::Unknown`) is genuinely weaker than the ceiling's (`Assurance::Sound`),
    /// the identical reason `nomos-lang-rust-clippy`'s own test gives for why the reversed
    /// direction would give a false pass here only by coincidence.
    #[test]
    fn Test_The_Ceiling_Should_Satisfy_The_Declared_Guarantee()
    {
        assert!(Ceiling().Satisfies(&Declared_Guarantee()));
    }

    /// The negative control: this guarantee does not satisfy the ceiling back, because its
    /// completeness is genuinely weaker. If it did, the test above would pass for two
    /// equal guarantees and prove nothing about ranking.
    #[test]
    fn Test_The_Declared_Guarantee_Should_Not_Satisfy_The_Ceiling()
    {
        assert!(!Declared_Guarantee().Satisfies(&Ceiling()));
    }

    /// The positive control against a real caller's floor: `nomos_rules::Dependency_
    /// Requirement`'s own floor is `Assurance::Sound` on both axes — this provider's
    /// `Assurance::Unknown` completeness does not clear it, honestly. A provider that
    /// cannot meet the one real requirement in this workspace is not a defect; it is what
    /// this crate's own module doc already names as the `replace`-directive gap, reported
    /// rather than hidden.
    #[test]
    fn Test_This_Should_Not_Satisfy_A_Requirement_For_Sound_Completeness()
    {
        let needs_sound_completeness = Guarantee::New(
            FactVariant::SemanticallyResolved,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::Project,
        );

        assert!(!Declared_Guarantee().Satisfies(&needs_sound_completeness));
    }

    /// The negative control for the test above: this guarantee does satisfy a requirement
    /// that only asks for what it actually claims, so the refusal above is about the
    /// requirement's floor and not about this guarantee failing to satisfy anything at all.
    #[test]
    fn Test_This_Should_Satisfy_A_Requirement_That_Accepts_Unknown_Completeness()
    {
        let accepts_unknown_completeness = Guarantee::New(
            FactVariant::SemanticallyResolved,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::Project,
        );

        assert!(Declared_Guarantee().Satisfies(&accepts_unknown_completeness));
    }
}
