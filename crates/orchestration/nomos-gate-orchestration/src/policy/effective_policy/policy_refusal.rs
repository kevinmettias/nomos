//! Why a set of contributions could not be resolved into an effective policy.

use nomos_contracts::ConfigurationLayer;

use super::{FieldProvenance, PolicyField, PolicyUnit};

/// A statement this resolution refused, in `OD-POLICY-001`'s own four cases.
///
/// Three of them are that record's "refuse" family and the fourth is the unit rule's own.
/// Every one of them names the offending key or artifact, because a refusal an author cannot
/// locate in their own file is a refusal they cannot act on -- the same standard
/// `crate::policy::gate_policy_file`'s reader already holds itself to one level in.
///
/// **Declaration order resolves none of them.** A contradiction within one layer is refused
/// rather than settled by whichever contribution was passed first: `MODEL-ROUTE-023` forbids
/// declaration order, client preference, package load order and hidden last-write-wins as
/// ways to resolve an equal-specificity conflict, and `OD-POLICY-001` applies that rule to
/// every field.
///
/// [`Self::LockedOverride`] is the one case that does not stop the resolution. `CONFIG-003`
/// requires a rejected override to remain visible with its reason, so it is written into
/// [`super::ResolvedField::rejected`] instead of returned -- see
/// [`super::RejectedOverride`], which carries the reason this type writes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyRefusal
{
    /// One key stated twice within one layer with different values, by two artifacts of that
    /// layer or by one artifact twice.
    ContradictionWithinALayer
    {
        layer: ConfigurationLayer,
        field: PolicyField,
        /// The key both statements address, as an author wrote it.
        key: String,
        /// The artifacts of that layer that disagreed.
        artifacts: Vec<String>,
    },
    /// A lower layer stated a field a higher layer locked (`CONFIG-003`).
    LockedOverride
    {
        field: PolicyField,
        /// The layer and artifact that locked the field.
        locked_by: FieldProvenance,
        /// The layer and artifact that tried to state it anyway.
        offered_by: FieldProvenance,
    },
    /// An artifact is present and cannot be read as its declared shape.
    UnreadableArtifact
    {
        layer: ConfigurationLayer,
        artifact: String,
        /// What the reader said was wrong with it.
        detail: String,
    },
    /// A contribution stated a companion field of a unit without that unit's deciding field.
    CompanionWithoutADecidingField
    {
        unit: PolicyUnit,
        /// The companion field that was stated.
        field: PolicyField,
        layer: ConfigurationLayer,
        artifact: String,
    },
}

impl PolicyRefusal
{
    /// The sentence this refusal hands whoever wrote the statement it refused.
    ///
    /// Single-spaced prose, the same standard
    /// `crate::policy::gate_policy_file::tests::Test_Every_Refusal_Sentence_Should_Render_As_Single_Spaced_Prose`
    /// holds the reader's own refusals to: a sentence wrapped across source lines renders
    /// with a run of spaces where each break was, which reads as a formatting accident at the
    /// exact moment an author is being told how to repair their policy.
    #[must_use]
    pub fn Sentence(&self) -> String
    {
        return match self
        {
            Self::ContradictionWithinALayer { layer, field, key, artifacts } => Contradiction_Sentence(*layer, *field, key, artifacts),
            Self::LockedOverride { field, locked_by, offered_by } => Locked_Sentence(*field, locked_by, offered_by),
            Self::UnreadableArtifact { layer, artifact, detail } => Unreadable_Sentence(*layer, artifact, detail),
            Self::CompanionWithoutADecidingField { unit, field, layer, artifact } => Orphan_Sentence(*unit, *field, *layer, artifact),
        };
    }
}

/// What a same-layer contradiction says to the author of either statement.
fn Contradiction_Sentence(layer: ConfigurationLayer, field: PolicyField, key: &str, artifacts: &[String]) -> String
{
    return format!(
        "'{key}' is stated twice at the {} layer with different values, under '{}', by {}. Two statements of equal authority cannot be settled by which was read first, so this is refused rather than resolved: make the two agree, or state the key at one layer only.",
        layer.Label(),
        field.Label(),
        Listed(artifacts)
    );
}

/// What a locked field says to the author who tried to override it.
fn Locked_Sentence(field: PolicyField, locked_by: &FieldProvenance, offered_by: &FieldProvenance) -> String
{
    return format!(
        "'{}' is locked by {} and {} states it anyway. A locked field may not be overridden from a lower layer, and the statement is kept visible here rather than dropped so that the author can see it was refused rather than outranked.",
        field.Label(),
        locked_by.Sentence(),
        offered_by.Sentence()
    );
}

/// What an unreadable artifact says to the author of the file.
fn Unreadable_Sentence(layer: ConfigurationLayer, artifact: &str, detail: &str) -> String
{
    return format!(
        "'{artifact}' at the {} layer is present and could not be read as the shape it declares: {detail}. A policy artifact that cannot be read is not an empty one, because a run that passed under it would be passing under policy nobody authored.",
        layer.Label()
    );
}

/// What a companion stated without its deciding field says to whoever stated it.
fn Orphan_Sentence(unit: PolicyUnit, field: PolicyField, layer: ConfigurationLayer, artifact: &str) -> String
{
    return format!(
        "'{}' is stated by '{artifact}' at the {} layer and '{}' is not, and the two are {}. A '{}' entry addresses a key only '{}' declares, so the one way this could take effect is by pairing with another source's declaration -- which would let it address a stage its own source never declared: state both, or neither.",
        field.Label(),
        layer.Label(),
        unit.deciding_field.Label(),
        unit.name,
        field.Label(),
        unit.deciding_field.Label()
    );
}

/// Artifacts as one comma-separated phrase, so a refusal names every source that disagreed.
fn Listed(artifacts: &[String]) -> String
{
    return artifacts.iter().map(|artifact| return format!("'{artifact}'")).collect::<Vec<String>>().join(" and ");
}
