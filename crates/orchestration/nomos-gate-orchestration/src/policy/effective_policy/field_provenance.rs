//! Where one field's value came from.

use nomos_contracts::ConfigurationLayer;

use super::PolicyUnit;

/// The layer and the artifact behind one statement about one field.
///
/// `CONFIG-001` names source layer and source artifact as two fields, and `OD-POLICY-001`
/// keeps them apart for the reason its measurement gives: the `Repository` layer has four
/// artifacts today and `TemporaryRunOverride` has as many as there are callers, so a layer
/// alone does not say which source spoke.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldProvenance
{
    /// Which layer stated it.
    pub layer: ConfigurationLayer,
    /// Which source within that layer: a repository-relative path for a file, the caller for
    /// a policy built in code, the build for a default.
    pub artifact: String,
    /// The unit whose deciding field carried this statement, when a unit decided it.
    ///
    /// `OD-POLICY-001` version 2: every field of a resolved unit carries the deciding field's
    /// provenance *and says that it did*, because a provenance naming only its own field
    /// would report `approvals` as though its source had written approvals when that source
    /// may have written none -- the difference between a value and a statement this whole
    /// resolution turns on. So an effective policy reads "approvals: `Repository`,
    /// `nomos-gate.json`, decided with `phases` as the phase policy" and never
    /// "approvals: `Repository`, `nomos-gate.json`" alone.
    ///
    /// `None` for every field that resolved on its own.
    pub decided_by_unit: Option<PolicyUnit>,
}

impl FieldProvenance
{
    /// This provenance as one line of prose a refusal or a report can carry.
    #[must_use]
    pub fn Sentence(&self) -> String
    {
        let Some(unit) = self.decided_by_unit
        else
        {
            return format!("{} ({})", self.layer.Label(), self.artifact);
        };

        return format!("{} ({}), decided with '{}' as {}", self.layer.Label(), self.artifact, unit.deciding_field.Label(), unit.name);
    }
}
