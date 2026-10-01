//! What this provider promises, stated as a value — checked downward, and owed upward.
//!
//! Downward, [`nomos_capability::Registry`] refuses [`Provider_Offer`] if it claims more
//! than the capability's ceiling permits. `Test_Claiming_Resolution_Should_Be_Refused`
//! below exercises that directly, offering a fabricated offer claiming
//! `FactVariant::SemanticallyResolved` and asserting the registry turns it down.
//!
//! # The upward half is owed, and this doc used to say it was held
//!
//! It said "Upward, this crate's own tests assert what it actually emits against the claim".
//! They do not, and `OD-CAPABILITY-016` measured it: of eleven guarantee-declaring crates,
//! two hold that convention, and this crate — one of the two that *stated* it — is not among
//! them. Recorded here rather than quietly deleted, because a doc that overstated for months
//! is the evidence for why the record made the convention a requirement with a mechanism
//! instead of leaving it as prose two files had agreed on.
//!
//! What this crate does have is real and is a different thing.
//! `Test_Broken_Source_Should_Be_Unparseable_Rather_Than_Empty` (in `syntax/tests.rs`) holds
//! that a file `tree-sitter` could not parse cleanly yields `Reading::Unparseable` rather
//! than a `Parsed` carrying no items — refusal honesty, `OD-RULES-001`'s rule that an
//! absence must not read as a clean answer. It is the same class `OD-CAPABILITY-016` carved
//! out for `nomos-lang-rust-deny`, and it is not an axis of the guarantee below.
//!
//! So all four declared axes are unexercised against emitted output: no test asserts that
//! every item reported occurs in the source (`Sound`), that no name is resolved
//! (`Syntactic`), that the item boundary this crate declares is in fact fully covered
//! (`Complete`), or that a reading carries nothing across file boundaries (`File`).
//! `nomos-lang-rust`'s `tests/guarantee.rs` is the shape each would take, one property per
//! axis. Naming them is what this crate owes under that record; writing them is a separate
//! increment and no test is added here.

use nomos_cap_syntax::{Capability, CONTRACT_VERSION};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

/// This implementation.
pub const PROVIDER: &str = "nomos.lang.go.tree-sitter";

/// The language this provider reads, as `OD-RULES-014` has a rule name it.
///
/// Deliberately not derived from [`PROVIDER`], which names the tool rather than the
/// language — the two Rust providers declare one shared value beside two different
/// identities for exactly that reason.
pub const LANGUAGE: &str = "go";

/// What this provider claims, on every axis.
///
/// # Syntactic
///
/// Every answer is a function of one file's bytes. Nothing resolves an import path or knows
/// whether a `Writer` interface is satisfied by a given `File` — the same ceiling
/// `nomos-lang-rust` states for the same reason.
///
/// # Sound
///
/// Every item reported came from a parse tree with no `ERROR` or `MISSING` node anywhere in
/// it — [`crate::Read_Source`] refuses the whole file rather than report the part before a
/// syntax error, because a node inside `tree-sitter`'s own error-recovery region is the
/// parser's guess about what the author meant, not a fact the source contains.
///
/// # Complete, and this is the real divergence from `nomos-lang-rust`
///
/// Rust's own provider cannot claim this: a macro invocation is where its parse tree ends and
/// an unexpanded token stream begins, the count of such places is itself only a lower bound,
/// and there is no way to bound what was generated there without resolving names. Go has no
/// macro system — no construct of any kind where the grammar hands back an opaque token
/// stream in place of a declaration. Generics do not create one either: a generic function or
/// type is the same `function_declaration` / `type_spec` node with an added
/// `type_parameters` field, not a new, less-transparent form — confirmed against the real
/// grammar before this claim was written, not assumed from Go's reputation for having no
/// macros. Neither do build tags (`//go:build`) or `go:generate` directives: both are
/// ordinary comments that this file's own text still contains every declaration around,
/// whatever a build configuration elsewhere decides to do with the file as a whole.
///
/// So there is no gap this provider cannot bound, which is what [`Assurance::Sound`] on this
/// axis requires. It is *not* a claim that this provider reads everything Go has syntax
/// for — struct fields and embedded interfaces are deliberately outside the item boundary
/// this crate states in its own crate doc, the same way `nomos-lang-rust` excludes struct
/// fields and enum variants. Completeness is evaluated against that stated boundary, not
/// against an unstated one; a boundary that excludes a form is a scope decision, and a
/// provider that sees everything inside the boundary it declared is complete regardless of
/// how the boundary was drawn.
///
/// # File
///
/// A Go file parses without reference to any other file, so a change to one file requires
/// reparsing exactly that file — the same reasoning `nomos-lang-rust` gives for `File` over
/// `Symbol`.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Sound,
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
            Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Sound, IncrementalGranularity::File)
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
                Assurance::Sound,
                IncrementalGranularity::File,
            ),
            ..Provider_Offer()
        };
    }

    /// The consequence of claiming `Sound` completeness, made visible at the resolution
    /// site: a caller that needs every item can resolve to this provider, unlike
    /// `nomos-lang-rust`'s.
    #[test]
    fn Test_A_Caller_Needing_Completeness_Should_Resolve_To_This_Provider()
    {
        let mut registry = Registry::New();
        registry
            .Declare(Capability_Contract())
            .expect("the contract is the first declaration in a fresh registry");
        registry.Offer(Provider_Offer()).expect("the offer is within the ceiling");

        let every_item = Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Unknown,
            Assurance::Sound,
            IncrementalGranularity::None,
        );
        let needs_every_item = Requirement::New(Capability(), CONTRACT_VERSION, every_item);

        let resolved = registry.Resolve(&needs_every_item);

        assert_eq!(
            resolved.Offer().map(|offer| return offer.provider.clone()),
            Some(ProviderId::New(PROVIDER)),
            "a provider claiming Sound completeness must satisfy a requirement that only \
             needs Unknown-or-better"
        );
    }
}
