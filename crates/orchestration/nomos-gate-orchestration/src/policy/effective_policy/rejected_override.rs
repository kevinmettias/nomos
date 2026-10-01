//! An override a higher layer's lock refused, kept visible rather than dropped.

use super::FieldProvenance;

/// A statement a lock refused, with the reason it was refused.
///
/// `CONFIG-003`: "security and organization policy may prohibit lower-level overrides, and
/// rejected overrides shall remain visible with reason." `OD-POLICY-001` lists the locked
/// override among its three refusal cases and gives that same sentence as its treatment --
/// "the override is rejected and kept visible with its reason" -- which is why this one of
/// the three does not stop the resolution the way a same-layer contradiction and an
/// unreadable artifact do. The refusal is still typed: the reason here is written by
/// [`super::PolicyRefusal::LockedOverride`], which names the field, the artifact that locked
/// it and the artifact that tried to state it.
///
/// A rejected override is not an overridden one. An overridden contribution lost to a higher
/// layer that stated the same field; a rejected one was forbidden from stating it at all, and
/// a report that folded the two together would tell an author their value was outranked when
/// it was in fact prohibited.
///
/// Nothing produces one from a real source today, because no layer that could carry a lock
/// has a source on this host -- `OD-POLICY-001`'s own table. The slot is here because the
/// alternative is a resolver that silently drops a refused statement, and the tests beside
/// this module build a locking contribution directly rather than leaving the path
/// unexercised.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RejectedOverride
{
    /// The contribution that tried to state the locked field.
    pub offered_by: FieldProvenance,
    /// Why it was refused, naming the layer and artifact that locked the field.
    pub reason: String,
}
