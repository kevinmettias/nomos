//! The declared units: sets of fields no layer can state apart, and the field that decides
//! each one.
//!
//! `OD-POLICY-001` version 2 decides that a unit is **declared, never inferred**. That an
//! approval names a phase is a fact about what the two fields mean, not one any shape can
//! compute, so the unit and its deciding field are named here and pinned by a test rather
//! than derived from the values at resolution time.
//!
//! One unit exists. `OD-POLICY-001` measured, at `54e88f78`, that nothing else in the tree
//! resolves two policy fields from one statement, and states that a resolver may not promote
//! a coupling it notices into a unit of its own: naming the fields and the deciding field is
//! a decision, and it belongs to the record that measures it.

use super::PolicyField;

/// A declared set of fields no layer can state apart, with the one that decides them.
///
/// `OD-POLICY-001`: "a *unit* is a declared set of two or more fields of one policy in which
/// a value of one field can only address something another field of the set declares, and one
/// field of the unit is designated its *deciding field*. The highest layer that states the
/// deciding field decides every field of the unit, and no other layer contributes to any
/// field in it."
///
/// Overridden whole, even where a companion has the shape of a keyed set, because a set
/// assembled from two layers is a set neither author wrote -- the silent winner `ARCH-008`
/// prohibits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PolicyUnit
{
    /// What a report calls this unit when it says which statement decided a field.
    pub name: &'static str,
    /// The field whose statement decides every field of the unit.
    pub deciding_field: PolicyField,
    /// The fields that address keys only `deciding_field` declares.
    ///
    /// A contribution stating one of these without the deciding field is refused, because the
    /// one way it could take effect is by being paired with another layer's declaration.
    pub companion_fields: &'static [PolicyField],
}

impl PolicyUnit
{
    /// Whether `field` is one of this unit's fields, deciding or companion.
    #[must_use]
    pub fn Contains(&self, field: PolicyField) -> bool
    {
        return field == self.deciding_field || self.companion_fields.contains(&field);
    }
}

/// The one unit this workspace declares: `phases` decides, `approvals` follows.
///
/// The reason is quoted rather than re-derived, from what `Preferred_Phase_Policy` said in
/// code before this resolver took the decision over: "`phases` alone decides it, because
/// approvals are read only through them -- [`crate::Evaluated_Phases`] iterates the phases
/// and asks each whether an approval names it, so a source declaring approvals and no phase
/// has declared nothing a run can act on." [`crate::GateCommand::approvals`] states the
/// consequence: an approval "names the phase it covers, so a caller's phases paired with a
/// file's approvals would let an approval address a stage its own source never declared."
pub const PHASE_POLICY_UNIT: PolicyUnit =
    PolicyUnit { name: "the phase policy", deciding_field: PolicyField::Phases, companion_fields: &[PolicyField::Approvals] };
