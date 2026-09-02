//! What this provider promises, and what it deliberately does not declare.
//!
//! # Why there is no `Capability_Contract` here
//!
//! `nomos.cap.syntax.items` is not this crate's capability, and it is not
//! `nomos-lang-rust`'s either — a capability contract is the agreed meaning of a question
//! and the ceiling on what any answer may claim, and an agreement is not the property of one
//! party to it. It lives in `nomos-cap-syntax`, below both providers, and both offer against
//! it.
//!
//! It used to live in `nomos-lang-rust`, because that was the only provider when it was
//! written. This crate then agreed with its peer by retyping the peer's string constants —
//! two providers at one band cannot name each other, so there was no other way — and nothing
//! would have noticed the day one of them was retyped differently. What held the invariant
//! was `Registry::Declare` refusing a second contract for one capability, which is the
//! registry compensating for the layering rather than the layering being right.
//!
//! What this crate does is *offer*. That is exactly the relationship a second provider is
//! supposed to have, and now the first one has it too.

use nomos_cap_syntax::{Capability, CONTRACT_VERSION};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

pub const PROVIDER: &str = "nomos.lang.rust.scan";

/// The language this provider reads, as `OD-RULES-014` has a rule name it.
///
/// The same value `nomos-lang-rust` declares, and that is the point: one language, two
/// providers of it, so a caller asking which language a file is cannot be answered with
/// either provider identity.
pub const LANGUAGE: &str = "rust";

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
        capability: Capability(),
        version: CONTRACT_VERSION,
        guarantee: Declared_Guarantee(),
    };
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
