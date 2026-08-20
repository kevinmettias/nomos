//! A rule package's claim that it offers one rule.

use nomos_contracts::RuleId;

/// A rule package's claim that it offers one rule, at the contract record and version its
/// implementation was written against.
///
/// Shaped after [`nomos_capability::ProviderOffer`] closely enough that a rule package can
/// hand one of these to a [`super::RuleRegistry`] without either party naming the other's
/// crate — the same seam `ProviderId`/`ProviderOffer` give `nomos_capability::Registry`. A
/// capability and a guarantee have no rule-side equivalent yet, because nothing here ranks
/// or selects between offers the way a capability registry does; a rule's own record
/// citation (`CONTRACT_RECORD`/`CONTRACT_RECORD_VERSION`, `mirror.rs`'s own worked example)
/// is the closer analogue to what a provider's `version` means.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleOffer
{
    /// The rule's own identity, the same [`RuleId`] a [`nomos_contracts::Finding`] carries.
    pub rule: RuleId,
    /// The governing record this rule's implementation cites, e.g. `"D-134"`.
    pub contract_record: String,
    /// The version of `contract_record` this implementation was written against.
    pub contract_record_version: u32,
}
