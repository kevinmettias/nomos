//! What this provider promises, stated as a value — checked downward, checked upward, and
//! weaker than its Go peer on exactly one axis.
//!
//! Downward, [`nomos_capability::Registry`] refuses [`Provider_Offer`] if it claims more than
//! the capability's ceiling permits. `Test_Claiming_Resolution_Should_Be_Refused` below
//! exercises that directly, offering a fabricated offer claiming
//! `FactVariant::SemanticallyResolved` and asserting the registry turns it down.
//!
//! Upward, the completeness axis is the one this provider does not inherit from
//! `nomos-lang-go`, so it is the one whose consequence is exercised rather than asserted: a
//! caller that requires every declaration must not resolve to this provider, and a caller
//! that will accept a bounded gap must. `nomos-lang-rust-scan`'s own guarantee tests are the
//! worked example of that pair, and the reason both halves are here: the negative on its own
//! would pass over a provider that satisfies nothing at all.
//!
//! What the other three axes have is stated plainly rather than left silent, the way
//! `OD-CAPABILITY-016` requires. `FactVariant::Syntactic` is exercised only as a value —
//! what it does and does not satisfy as a requirement — never against emitted output;
//! soundness is exercised against emitted output by `crate::syntax`'s own
//! `Test_A_Declaration_Inside_A_Conditional_Region_Should_Not_Be_Reported` and
//! `Test_Broken_Source_Should_Be_Unparseable_Rather_Than_Empty`, which together are the two
//! ways an unsound item could reach a caller; and nothing demonstrates that a reading carries
//! nothing across file boundaries, which is what `File` granularity claims.

use nomos_cap_syntax::{Capability, CONTRACT_VERSION};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

/// This implementation.
pub const PROVIDER: &str = "nomos.lang.csharp.tree-sitter";

/// The language this provider reads, as `OD-RULES-014` has a rule name it.
///
/// Deliberately not derived from [`PROVIDER`], which names the tool rather than the
/// language — the two Rust providers declare one shared value beside two different
/// identities for exactly that reason.
pub const LANGUAGE: &str = "csharp";

/// What this provider claims, on every axis.
///
/// Exercised by `Test_A_Declaration_Inside_A_Conditional_Region_Should_Not_Be_Reported`, which is
/// the soundness axis measured against emitted output rather than asserted as a value:
/// `OD-CAPABILITY-016` requires the declaration to name what holds it to itself, and this test
/// is killed both by deleting the guard and by inverting it into a descent.
///
/// # Syntactic
///
/// Every answer is a function of one file's bytes. Nothing resolves a `using` to the assembly
/// it names, decides whether a `partial class` has a second half elsewhere, or reads a project
/// file to learn which symbols a build defines — the same ceiling every other provider of this
/// capability states, for the same reason.
///
/// # Sound
///
/// Two conditions, and both are needed here where one was enough for Go.
///
/// Every item reported came from a parse tree with no `ERROR` or `MISSING` node anywhere in
/// it — [`crate::Read_Source`] refuses the whole file rather than report the part before a
/// syntax error, because a node inside `tree-sitter`'s own error-recovery region is the
/// parser's guess about what the author meant.
///
/// And every item reported sits outside every preprocessor conditional region. In this grammar
/// a `#if`/`#elif`/`#else` chain is one subtree carrying *every* branch's declarations at once,
/// and at most one of those branches is in any compilation. A provider that reads no project
/// file cannot know which, so reporting all of them would report declarations that no single
/// compilation contains — which is precisely what an [`Assurance::Sound`] soundness claim
/// forbids. `#region`, `#pragma`, `#nullable` and `#define` are not conditional and gate
/// nothing: the grammar leaves declarations around them as ordinary siblings, confirmed
/// against the real tree, so they are passed over and cost nothing.
///
/// # Unknown completeness, and this is the real divergence from `nomos-lang-go`
///
/// Go's provider claims soundness *and* completeness because Go has no construct where the
/// parse tree hands back something a declaration might have been. C# has one, and it is the
/// same shape as the Rust macro gap that keeps `nomos-lang-rust`'s own completeness at
/// [`Assurance::Unknown`]: a conditional region is a place where declarations exist that this
/// provider declines to read, and the payload header carries a count of the regions rather
/// than of the declarations inside them. That is a lower bound that cannot be tightened into a
/// real count without a definition set from outside the file, which is exactly the condition
/// `Unknown` describes. [`Assurance::Unsound`] would be wrong: nothing here establishes that
/// declarations *are* being missed in a given file, only that this provider cannot say they
/// are not.
///
/// Four constructs that look like they should weaken this axis and do not, each checked
/// against the real grammar before this claim was written:
///
/// A generic type or method is the same `class_declaration` or `method_declaration` node with
/// an added `type_parameter_list` field, not a new, less-transparent form.
///
/// A nested type is an ordinary child of its parent's `declaration_list`, read like any other
/// declaration and qualified by what encloses it.
///
/// A `partial` type is a full declaration in each file that carries a piece of it, and this is
/// a per-file fact: a file declaring one half of a type declares exactly that half, which is
/// what `File` granularity below already says. Nothing here claims to know a type's whole
/// member set, and the crate doc's item boundary is what completeness is evaluated against.
///
/// A file-scoped namespace (`namespace X;`) is its own node kind rather than a namespace whose
/// body went missing, and is read as one.
///
/// # File
///
/// A C# file parses without reference to any other file, and this provider's answer depends on
/// nothing outside the file's own bytes — including no definition set, since it reads no
/// conditional region. So a change to one file requires reparsing exactly that file, the same
/// reasoning `nomos-lang-go` gives for `File` over `Symbol`.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );
}

/// This provider's offer against [`nomos_cap_syntax::Capability_Contract`].
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
    use nomos_cap_syntax::Capability_Contract;
    use nomos_capability::{OfferRefusal, Registry, RegistryError, RegistryErrorKind, Requirement};

    #[test]
    fn Test_Declared_Guarantee_Should_State_A_Sound_Syntactic_File_Granular_Claim()
    {
        assert_eq!(
            Declared_Guarantee(),
            Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File)
        );
    }

    #[test]
    fn Test_Provider_Offer_Should_Be_Accepted_Under_The_Capabilitys_Contract()
    {
        let mut registry = Registry::New();
        registry
            .Declare(Capability_Contract())
            .expect("the contract is the first declaration in a fresh registry");

        assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
    }

    #[test]
    fn Test_Claiming_Resolution_Should_Be_Refused()
    {
        let mut registry = Registry::New();
        registry
            .Declare(Capability_Contract())
            .expect("the contract is the first declaration in a fresh registry");

        let refused = registry.Offer(Claiming_Resolution());

        assert_eq!(
            refused,
            Err(RegistryError {
                capability: Capability(),
                kind: RegistryErrorKind::Offer {
                    provider: ProviderId::New(PROVIDER),
                    refusal: OfferRefusal::ExceedsCeiling,
                },
            })
        );
    }

    fn Claiming_Resolution() -> ProviderOffer
    {
        return ProviderOffer {
            guarantee: Guarantee::New(
                FactVariant::SemanticallyResolved,
                Assurance::Sound,
                Assurance::Unknown,
                IncrementalGranularity::File,
            ),
            ..Provider_Offer()
        };
    }

    /// The consequence of declining `Sound` completeness, made visible at the resolution
    /// site: a caller that needs every declaration cannot be served this provider, which is
    /// the whole reason the axis is stated as a value rather than described in prose.
    #[test]
    fn Test_A_Caller_Needing_Every_Declaration_Should_Not_Resolve_To_This_Provider()
    {
        let resolved = Resolved_Against(Needs_Every_Declaration());

        assert_eq!(
            resolved, None,
            "a provider that declines to read conditional regions must not satisfy a \
             requirement for every declaration"
        );
    }

    /// The positive control. Without it the assertion above would pass over a provider that
    /// satisfies nothing at all, which is the vacuity `nomos-lang-rust-scan`'s own guarantee
    /// tests exist to refuse.
    #[test]
    fn Test_A_Caller_Accepting_A_Bounded_Gap_Should_Resolve_To_This_Provider()
    {
        let resolved = Resolved_Against(Accepts_A_Bounded_Gap());

        assert_eq!(
            resolved,
            Some(ProviderId::New(PROVIDER)),
            "a requirement asking only for a sound syntactic reading is one this provider meets"
        );
    }

    /// `Unknown` is not `Unsound`. One says nobody has bounded the gap, the other that the
    /// property is known to fail, and a consumer's response to them differs.
    #[test]
    fn Test_Unknown_Completeness_Should_Not_Be_Spelled_Unsound()
    {
        assert_ne!(Declared_Guarantee().completeness, Assurance::Unsound);
        assert!(!Declared_Guarantee().completeness.Satisfies_Requirement());
    }

    fn Needs_Every_Declaration() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Unknown,
            Assurance::Sound,
            IncrementalGranularity::None,
        );
    }

    fn Accepts_A_Bounded_Gap() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::None,
        );
    }

    /// The provider a registry holding only this one resolves to for `required`, if any.
    fn Resolved_Against(required: Guarantee) -> Option<ProviderId>
    {
        let mut registry = Registry::New();
        registry
            .Declare(Capability_Contract())
            .expect("the contract is the first declaration in a fresh registry");
        registry.Offer(Provider_Offer()).expect("the offer is within the ceiling");

        let requirement = Requirement::New(Capability(), CONTRACT_VERSION, required);

        return registry
            .Resolve(&requirement)
            .Offer()
            .map(|offer| return offer.provider.clone());
    }
}
