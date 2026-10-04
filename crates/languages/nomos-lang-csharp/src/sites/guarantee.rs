//! What this crate's sites offer promises -- the split `crate::guarantee` draws for the items offer.

use nomos_cap_syntax::{SITES_CONTRACT_VERSION, Sites_Capability};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

/// What this offer claims, on every axis.
///
/// # Syntactic
///
/// What it states is a fact about C#'s grammar, which is the face of every C# file.
///
/// # Sound, on both axes
///
/// It offers no kind, so it reports no site and cannot report a false one; and it declines the
/// labeled jump for every C# file, which is complete because the construct exists in none of them.
/// A declined kind with a site would be refused by the family's reader, so the claim cannot be
/// quietly broken by the payload either.
///
/// # File
///
/// The answer is a function of one file's text and of nothing another file says.
///
/// Exercised by `Test_Every_Csharp_File_Should_Decline_The_Labeled_Jump_With_Its_Reason`, in
/// `tests/sites_capability.rs`.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Sound, IncrementalGranularity::File);
}

/// This provider's offer against [`nomos_cap_syntax::Sites_Capability_Contract`], under the identity
/// its items offer already carries, so a rule narrowing a C# file toward the provider that
/// recognized it finds this offer and reads the decline.
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
