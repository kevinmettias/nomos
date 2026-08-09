//! What this provider promises, and what it deliberately does not declare.
//!
//! # Why there is no `Capability_Contract` here
//!
//! `nomos.cap.syntax.items` is not this crate's capability. It is not `nomos-lang-rust`'s
//! either — a capability contract is the agreed meaning of a question and the ceiling on
//! what any answer may claim, and an agreement is not the property of one party to it.
//!
//! Today the contract is authored in `nomos-lang-rust` because it was the only provider,
//! and `Registry::Declare` refuses a second contract for one capability, so nothing here
//! could declare it even if this crate wanted to. That is the registry protecting the
//! invariant rather than the layering being right: the contract's home should be below both
//! providers, and there is no crate there yet. P8-CONTRACT-HOME carries it.
//!
//! What this crate does is *offer*, against a capability it names by string and does not
//! own. That is exactly the relationship a second provider is supposed to have.

use nomos_capability::ProviderOffer;
use nomos_contracts::{
    Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity,
    ProviderId, SchemaId,
};

/// The capability answered, named by string.
///
/// The same string `nomos-lang-rust` uses, written out again rather than imported. Two
/// providers at one band cannot name each other — `tests/contract` forbids the edge — and
/// that is the design: what they share is a name and a payload format, which is an
/// interface, and nothing else.
pub const CAPABILITY: &str = "nomos.cap.syntax.items";

pub const PROVIDER: &str = "nomos.lang.rust.scan";

/// The same payload schema, for the same reason.
///
/// A schema is the shape of an answer, not a claim about its accuracy — that is what the
/// guarantee is for. Two providers of one capability that wrote different shapes would
/// force every consumer to know which one answered, and the point of resolving through a
/// registry is that it does not have to.
pub const SCHEMA: &str = "nomos.syntax.items.v1";

pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// What a line-reader can promise.
///
/// Three of the four axes differ from `nomos-lang-rust`'s, and each difference is a fact
/// about the method rather than a hedge:
///
/// [`FactVariant::Approximate`] — established by a method that trades accuracy for cost,
/// which is the definition of reading lines instead of parsing them.
///
/// [`Assurance::Unsound`] for soundness, not `Unknown`. `Unknown` means nobody has
/// established it either way; this is established, and it is false. `crate::Scan`'s own
/// tests enumerate the cases where it reports a declaration that is not there.
///
/// Completeness stays [`Assurance::Unknown`]: it misses declarations written across two
/// lines and every item nested on one, and nobody has bounded how many that is.
///
/// [`IncrementalGranularity::File`] is the one axis that matches, and honestly so — the
/// scan reads one file and nothing else.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Approximate,
        Assurance::Unsound,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );
}

#[must_use]
pub fn Provider_Offer() -> ProviderOffer
{
    return ProviderOffer {
        provider: ProviderId::New(PROVIDER),
        capability: CapabilityId::New(CAPABILITY),
        version: CONTRACT_VERSION,
        guarantee: Declared_Guarantee(),
    };
}

#[must_use]
pub fn Payload_Schema() -> SchemaId
{
    return SchemaId::New(SCHEMA);
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The declaration is weaker than a parser's on the axes it should be, and equal on the
    /// one it should be. Written as a comparison rather than as four constants, because
    /// what matters is the difference and a constant restated is not a check.
    #[test]
    fn Test_The_Guarantee_Should_Be_Weaker_Than_A_Parsers_On_Three_Axes()
    {
        let parser = Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::File,
        );
        let scan = Declared_Guarantee();

        assert!(scan.variant < parser.variant, "approximate is weaker than syntactic");
        assert_ne!(scan.soundness, parser.soundness);
        assert_eq!(scan.completeness, parser.completeness, "neither claims completeness");
        assert_eq!(
            scan.incremental, parser.incremental,
            "both read one file, so both refresh at file granularity"
        );
    }

    /// The load-bearing one. A caller that needs a sound syntactic answer must not be
    /// served this, and the whole registry mechanism rests on that comparison.
    #[test]
    fn Test_This_Should_Not_Satisfy_A_Requirement_For_A_Parse()
    {
        let needs_a_parse = Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::File,
        );

        assert!(!Declared_Guarantee().Satisfies(&needs_a_parse));
    }

    /// The positive control. If this guarantee satisfied nothing, the test above would pass
    /// over a provider that can never be selected.
    #[test]
    fn Test_This_Should_Satisfy_A_Requirement_That_Accepts_An_Approximation()
    {
        let will_take_an_approximation = Guarantee::New(
            FactVariant::Approximate,
            Assurance::Unknown,
            Assurance::Unknown,
            IncrementalGranularity::File,
        );

        assert!(Declared_Guarantee().Satisfies(&will_take_an_approximation));
    }

    /// Unsound is not Unknown. One says the property is known not to hold, the other that
    /// nobody has looked, and a consumer's response to them differs.
    #[test]
    fn Test_Unsound_Should_Not_Be_Spelled_Unknown()
    {
        assert_ne!(Declared_Guarantee().soundness, Assurance::Unknown);
        assert!(!Declared_Guarantee().soundness.Satisfies_Requirement());
    }
}
