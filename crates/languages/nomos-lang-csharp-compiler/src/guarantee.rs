//! What this offer promises, stated as a value and checked in both directions.
//!
//! Downward, `nomos_capability::Registry` refuses [`Provider_Offer`] if it claims more than
//! `nomos_cap_csharp_semantics::Ceiling` permits. Upward, `tests/guarantee.rs` exercises every
//! axis against what the provider emits, and against the C# compiler itself, which is what
//! `OD-CAPABILITY-016` requires of a declaring crate.

use nomos_cap_csharp_semantics::{Capability, CONTRACT_VERSION};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

/// This implementation, named for the tool that resolves its answer: `MSBuild`, asked through the
/// .NET SDK.
pub const PROVIDER: &str = "nomos.lang.csharp.msbuild";

/// What this offer claims, on every axis.
///
/// Exercised by `Test_Every_Branch_State_Should_Be_The_One_The_Compiler_Takes` for soundness and
/// completeness, against the C# compiler itself; by
/// `Test_The_Variant_Should_Be_Resolved_Because_The_Answer_Turns_On_The_Builds_Symbols` for the
/// variant; and by `Test_The_Granularity_Should_Be_File_Because_A_Reading_Carries_Nothing_Across`
/// for the granularity -- all three in `tests/guarantee.rs`.
///
/// # `SemanticallyResolved`
///
/// Every symbol a condition names is resolved against the set the build hands its compiler,
/// which `MSBuild` evaluates from the project, its imports, the SDK's defaults for the
/// configuration and the target framework's implicit symbols. A reading of the file alone cannot
/// know that set; this is the one step that makes the answer a fact about a compilation rather
/// than about text.
///
/// # Sound
///
/// Every branch reported is one the file has, and its state is the one the specification's rules
/// give for that set and the file's own `#define` and `#undef`. A condition that is not a
/// well-formed pre-processing expression, or that reads a symbol a `#define` in an unevaluated
/// branch may have changed, is reported `Unevaluated` rather than guessed. `tests/guarantee.rs`
/// builds a fixture with the real compiler, with every branch this provider calls skipped made
/// invalid C# and every branch it calls compiled made necessary, so a single wrong state fails
/// the build.
///
/// # Complete
///
/// Every branch of every chain is reported, including chains nested inside skipped branches,
/// which the compiler never evaluates and this provider reports as skipped. A directive is
/// recognized wherever the compiler recognizes one: at the start of a line in code, and never
/// inside a comment, verbatim string, raw string or interpolation that spans lines -- the
/// constructs `crate::lexer` tracks, each with its own test. The same compiler-backed test holds
/// this axis: a branch the provider missed would keep its placeholder, which is valid only where
/// the branch is compiled.
///
/// # File
///
/// One file's answer is computed from its text and the build's definition set, and a change to
/// one file re-derives that file alone.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(FactVariant::SemanticallyResolved, Assurance::Sound, Assurance::Sound, IncrementalGranularity::File);
}

/// This crate's offer against `nomos_cap_csharp_semantics::Capability_Contract`.
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
    use nomos_cap_csharp_semantics::Capability_Contract;
    use nomos_capability::{OfferRefusal, Registry, RegistryError, RegistryErrorKind, Requirement};

    fn Declared_Registry() -> Registry
    {
        let mut registry = Registry::New();
        registry.Declare(Capability_Contract()).expect("the contract is the first declaration in a fresh registry");
        return registry;
    }

    #[test]
    fn Test_Provider_Offer_Should_Be_Accepted_Under_The_Capabilitys_Contract()
    {
        let mut registry = Declared_Registry();

        assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
    }

    #[test]
    fn Test_Claiming_Runtime_Observation_Should_Be_Refused()
    {
        let mut registry = Declared_Registry();
        let overclaim = ProviderOffer {
            guarantee: Guarantee::New(FactVariant::RuntimeObserved, Assurance::Sound, Assurance::Sound, IncrementalGranularity::File),
            ..Provider_Offer()
        };

        assert_eq!(
            registry.Offer(overclaim),
            Err(RegistryError {
                capability: Capability(),
                kind: RegistryErrorKind::Offer { provider: ProviderId::New(PROVIDER), refusal: OfferRefusal::ExceedsCeiling },
            })
        );
    }

    /// A caller asking for the ceiling itself resolves to this offer: it claims the whole of what
    /// the capability promises.
    #[test]
    fn Test_A_Caller_Needing_The_Whole_Ceiling_Should_Resolve_To_This_Offer()
    {
        let mut registry = Declared_Registry();
        registry.Offer(Provider_Offer()).expect("within the ceiling");

        let resolved = registry.Resolve(&Requirement::New(Capability(), CONTRACT_VERSION, nomos_cap_csharp_semantics::Ceiling()));

        assert_eq!(resolved.Offer().map(|offer| return offer.provider.clone()), Some(ProviderId::New(PROVIDER)));
    }
}
