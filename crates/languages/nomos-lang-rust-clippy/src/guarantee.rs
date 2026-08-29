//! What this provider offers, and at what guarantee.

use nomos_cap_lint::{Capability, CONTRACT_VERSION};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

/// This provider's own name.
///
/// Named for the tool it actually runs, the same way `nomos.lang.rust.cargo` is: a future
/// provider answering the same capability from a different tool (`rustc`'s own warnings
/// with no clippy pass, a second-language linter) would need to disagree with this name
/// honestly.
pub const PROVIDER: &str = "nomos.lang.rust.clippy";

/// What this provider claims, on every axis.
///
/// [`FactVariant::SemanticallyResolved`] at the ceiling: `cargo clippy` diagnoses over
/// fully type-checked code, not text alone.
///
/// Soundness [`Assurance::Sound`]: every diagnostic this provider reports is a diagnostic
/// `cargo clippy` itself emitted; nothing here infers or synthesizes one.
///
/// Completeness [`Assurance::Unknown`], deliberately not `Sound` — `OD-RULES-010`'s own
/// text already measured this: clippy does not claim to catch every possible issue its
/// own lints describe, many of its lints are allow-by-default, and no formal completeness
/// bound exists for what it does and does not catch. A floor no provider can honestly meet
/// is not caution, it is a declared need with nothing behind it — the same reasoning
/// `nomos-rules::Syntax_Requirement` already gives for accepting `Assurance::Unknown` on
/// this identical axis for a parser.
///
/// [`IncrementalGranularity::Project`]: this provider materializes one fact per workspace
/// member, bundling every diagnostic that member's own files produced in one `cargo
/// clippy` invocation over the whole workspace — there is no slice below "the member's own
/// diagnostics" this provider's own materialization exposes.
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

/// This provider's offer against [`nomos_cap_lint::Capability_Contract`].
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

    /// The offer must satisfy the contract's own ceiling — `contract.ceiling.Satisfies(&
    /// offer.guarantee)` is the exact check `Registry::Offer` runs at composition time
    /// (`nomos_capability::registry::declaring::Offer`), asserted here in the same
    /// direction so a future weakening of either constant is caught beside the constants
    /// rather than only at whatever composition root happens to run first. This
    /// provider's own completeness (`Assurance::Unknown`) is genuinely weaker than the
    /// ceiling's (`Assurance::Sound`), so the reversed direction `Declared_Guarantee().
    /// Satisfies(&Ceiling())` would give a false pass here only by coincidence elsewhere
    /// in this workspace, where every axis happens to already match the ceiling exactly.
    #[test]
    fn Test_The_Ceiling_Should_Satisfy_The_Declared_Guarantee()
    {
        use nomos_cap_lint::Ceiling;

        assert!(Ceiling().Satisfies(&Declared_Guarantee()));
    }
}
