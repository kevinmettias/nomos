//! What this crate's sites offer promises -- the split `crate::guarantee` draws for the items offer.

use nomos_cap_syntax::{SITES_CONTRACT_VERSION, Sites_Capability};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

/// What this offer claims, on every axis -- the claim [`crate::Declared_Guarantee`] makes for the
/// items offer, for the same reasons, over a different reading of the same tree.
///
/// # Syntactic
///
/// A labeled jump is resolved against the loops, switches and selects written around it, which is
/// lexical scope on the face of the file. No name is resolved and no type is read.
///
/// # Sound, on both axes
///
/// Every site reported was in the tree tree-sitter built from the file's bytes. And Go has no
/// construct where the grammar hands back an opaque token stream in place of statements -- no macro
/// system, the argument [`crate::Declared_Guarantee`] gives in full -- so a `break` or `continue`
/// naming a label is a node this walk reaches wherever it is written, a func literal in a
/// package-level `var` included.
///
/// # File
///
/// A Go file parses without reference to any other file.
///
/// Exercised by `Test_Soundness_Should_Hold_Every_Reported_Jump_Is_Written_On_Its_Line`, in
/// `tests/sites_capability.rs` beside the corpus's own cases.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Sound, IncrementalGranularity::File);
}

/// This provider's offer against [`nomos_cap_syntax::Sites_Capability_Contract`], under the identity
/// its items offer already carries, so a rule narrowing a Go file toward the provider that
/// recognized it finds this offer.
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
