//! One field of the effective policy, with what decided it.

use super::{FieldProvenance, PolicyField, RejectedOverride};

/// What decided one field, what it overrode, and what it rejected.
///
/// `OD-POLICY-001`: "for every single-valued field, the effective policy carries the decided
/// value, the layer and artifact that decided it, every contribution it overrode with that
/// contribution's own layer and artifact, and every override it rejected with the reason."
/// The decided *value* is not here: it stays in the policy types a run judges with, because
/// duplicating it would give a run two answers to the same question and no rule for which
/// one it judges under. This is the provenance beside those values, keyed by field.
///
/// Provenance is per field rather than per entry, which is what this increment builds of the
/// record's fuller shape: for a merged field, `decided_by` names the highest layer that
/// stated the field at all and `overrode` names every lower contribution some entry of which
/// a higher layer superseded. Per-entry provenance -- "for every merged field it carries the
/// same per entry" -- is the rendering item's, and is named here rather than left to be
/// discovered as a gap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedField
{
    /// Which field this is.
    pub field: PolicyField,
    /// The layer and artifact whose statement decided it, and the unit when a unit did.
    pub decided_by: FieldProvenance,
    /// Every contribution this one outranked, lowest layer first.
    ///
    /// A unit overridden whole records the lower layer's contribution for *every* field of
    /// it, so a reader of `approvals` alone sees that a file's approvals lost to a caller's
    /// stages although only `phases` was compared.
    pub overrode: Vec<FieldProvenance>,
    /// Every statement a lock refused, kept visible with its reason.
    pub rejected: Vec<RejectedOverride>,
}
