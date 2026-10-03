//! What this crate's sites offer promises, stated as a value and checked in two directions -- the
//! split `crate::guarantee` draws for the items offer.

use nomos_cap_syntax::{SITES_CONTRACT_VERSION, Sites_Capability};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

/// What this offer claims, on every axis -- the same claim [`crate::Declared_Guarantee`] makes,
/// for the same reasons, over a different reading of the same parse.
///
/// # Syntactic
///
/// A labeled jump is resolved against the loops and blocks written around it, which is lexical
/// scope on the face of the file. No name is resolved and no type is read.
///
/// # Sound
///
/// Every site reported was in the token stream: `syn` parsed the jump and the loop it names, and
/// the walk has no path by which to report a jump the file does not contain.
///
/// # Completeness Unknown
///
/// A macro invocation is where the parse ends and an unexpanded token stream begins, and a jump
/// inside one is invisible here -- `inner!(break 'outer)` names a label this walk never sees named.
/// How many such places a file holds is not bounded, so completeness may not be claimed, and a
/// rule judging a label over the jumps it can see says it judged part of the file.
///
/// # File
///
/// `syn` parses one file's bytes or fails, with no partial reparse to offer.
///
/// Exercised by `Test_Soundness_Should_Hold_Every_Reported_Jump_Is_Written_On_Its_Line`, in
/// `tests/sites_capability.rs` beside the corpus's own cases.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File);
}

/// This provider's offer against [`nomos_cap_syntax::Sites_Capability_Contract`].
///
/// [`crate::PROVIDER`] is reused rather than a second identity declared: the tool that reads the
/// file is the same `syn` parse, and a rule narrowing a file toward the provider that recognized it
/// finds this offer under the identity its items offer already carries.
#[must_use]
pub fn Provider_Offer() -> ProviderOffer
{
    return ProviderOffer {
        provider: ProviderId::New(crate::PROVIDER),
        capability: Sites_Capability(),
        version: SITES_CONTRACT_VERSION,
        guarantee: Declared_Guarantee(),
    };
}
