//! One canonical capability a rule's judgment implementation needs.

use nomos_contracts::{CapabilityId, Guarantee};

/// One entry of `ARCH-002`'s "required canonical capabilities".
///
/// `OD-PACKAGE-008`'s four-rule measurement found this field genuinely plural, not
/// singular: `Check_Completeness_Mirrors` and `Check_Naming_Convention` both require
/// `nomos_cap_syntax`, `Check_Dependency_Direction` requires `nomos_cap_dependency`, and
/// `Check_Unread_Reaches_A_Finding` requires `nomos_cap_controlflow` — three distinct
/// [`Guarantee`] shapes across four rules, with nothing yet saying three is the ceiling.
/// [`RulePackage::required_capabilities`](crate::RulePackage::required_capabilities) is a
/// list for exactly that reason.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityRequirement
{
    /// Which capability contract this rule's judgment implementation needs.
    pub capability: CapabilityId,
    /// The guarantee this rule needs from whatever provider answers that capability —
    /// the same [`Guarantee`] shape `nomos_capability::Requirement` already uses, reused
    /// unchanged rather than redefined for a manifest-declared context.
    pub minimum: Guarantee,
}

impl CapabilityRequirement
{
    /// Constructs a capability requirement.
    #[must_use]
    pub const fn New(capability: CapabilityId, minimum: Guarantee) -> Self
    {
        return Self { capability, minimum };
    }
}
