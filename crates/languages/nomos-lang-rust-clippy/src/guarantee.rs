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

// The seam this offer's own guarantee has with `nomos_cap_lint`'s ceiling and with
// `nomos_capability::Registry` is proven from outside this crate now, not inside it:
// `tests/capability_seam.rs` moved what used to be this module's own inline test out to
// where it can reach only the public API a real caller has — see that file's own module
// doc for why.
#[cfg(test)]
mod tests
{
    use super::*;

    /// `Declared_Guarantee` claims exactly these four axes — the offer this provider
    /// makes, pinned here in the same file check-test-coverage's own Rust strategy keys a
    /// unit test's companion function against, so a change to any one axis is a deliberate
    /// edit to this constant rather than a silent drift nothing in this file would catch.
    /// The seam this guarantee has with `nomos_cap_lint`'s ceiling is a different claim,
    /// proven in `tests/capability_seam.rs` instead.
    #[test]
    fn Test_Declared_Guarantee_Should_Claim_Sound_Semantically_Resolved_Diagnostics_At_Project_Granularity()
    {
        assert_eq!(
            Declared_Guarantee(),
            Guarantee::New(FactVariant::SemanticallyResolved, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::Project)
        );
    }

    /// `Provider_Offer` assembles this provider's own identity and its declared guarantee
    /// into one offer. Whether `nomos_capability::Registry` actually accepts it is a
    /// different claim, proven in `tests/capability_seam.rs`; this checks only that the
    /// offer itself is assembled from the right parts.
    ///
    /// Deliberately avoids the word pair "Declared Guarantee" in its own name:
    /// check-test-coverage's Rust strategy attributes a test to the *longest* declared
    /// function name it contains, and this unit also declares `Declared_Guarantee` — a name
    /// naming both would be attributed entirely to that longer one, leaving this function's
    /// own coverage unaddressed.
    #[test]
    fn Test_Provider_Offer_Should_Assemble_This_Providers_Own_Identity_And_Guarantee()
    {
        let offer = Provider_Offer();

        assert_eq!(offer.provider, ProviderId::New(PROVIDER));
        assert_eq!(offer.capability, Capability());
        assert_eq!(offer.version, CONTRACT_VERSION);
        assert_eq!(offer.guarantee, Declared_Guarantee());
    }
}
