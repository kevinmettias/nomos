//! What one layer's one artifact said about the gate policy.

use nomos_contracts::ConfigurationLayer;

use super::PolicyField;
use crate::policy::{AdoptionPolicy, BaselinePolicy, CoveragePolicy, EvidenceFloor, SuppressionPolicy};
use crate::{GatePhase, PhaseApproval};

/// One layer's statement about the gate policy, and the artifact that made it.
///
/// Every field is optional, and that is the whole point of the type: `OD-POLICY-001` decides
/// that "a layer that does not state it contributes nothing, and a sentinel is not a
/// statement". `None` is a layer saying nothing about a field; `Some` is a statement, even
/// when what it states is empty. A layer with no source at all contributes no
/// `PolicyContribution` -- it is reported absent by
/// [`super::EffectivePolicy::absent_layers`], which is a different fact from a source that
/// stated nothing.
///
/// A composition root says which layer it is speaking for. `OD-POLICY-001` measured that "the
/// caller" is three layers wearing one coat -- the CLI, a workflow step's own body, and a
/// policy a caller built in code -- and this type is the shape that makes them tell
/// themselves apart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyContribution
{
    /// Which layer is speaking.
    pub layer: ConfigurationLayer,
    /// Which source within that layer: a repository-relative path for a file, the caller for
    /// a policy built in code, the build for a default.
    pub artifact: String,
    /// The dispositions this artifact states, if it states any.
    pub suppressions: Option<SuppressionPolicy>,
    /// The existing debt this artifact states, if it states any.
    pub baseline: Option<BaselinePolicy>,
    /// The rule calibrations this artifact states, if it states any.
    pub adoption: Option<AdoptionPolicy>,
    /// The coverage floor this artifact states, if it states one.
    pub coverage: Option<CoveragePolicy>,
    /// The lowest evidence class this artifact lets a finding block on, if it states one.
    ///
    /// `OD-GATE-034`'s floor, contributed like any other single value: the highest layer that
    /// states one decides it, and a layer that leaves it at [`EvidenceFloor::Unset`] has said
    /// nothing rather than stated no floor.
    pub evidence_floor: Option<EvidenceFloor>,
    /// The ordered stages this artifact states, if it states any. The deciding field of
    /// [`super::PHASE_POLICY_UNIT`].
    pub phases: Option<Vec<GatePhase>>,
    /// The approvals this artifact states, if it states any.
    ///
    /// A companion of [`super::PHASE_POLICY_UNIT`]: stating it without `phases` is refused,
    /// because an approval addresses a key only `phases` declares.
    pub approvals: Option<Vec<PhaseApproval>>,
    /// Fields this artifact forbids a *lower* layer from stating (`CONFIG-003`).
    ///
    /// Empty for every source this workspace has: no organization or security layer is
    /// observable on this host, which is `OD-POLICY-001`'s own measurement rather than a
    /// decision taken here.
    ///
    /// A lock takes effect by layer rank, which is worth naming because `CONFIG-003` speaks of
    /// "security and organization policy" and `OD-POLICY-001`'s order places `Organization`
    /// second from the bottom, below `Repository`. The rule that record decides is about rank
    /// -- "a lower layer stating a field a higher layer has locked" -- so an organization lock
    /// would forbid nothing a repository states until a record decides that authority and
    /// precedence are two axes. That is a corpus question and not one a resolver may answer for
    /// itself; the mechanism is built to the rule as written.
    pub locks: Vec<PolicyField>,
    /// Why this artifact is present and could not be read as its declared shape.
    ///
    /// `Some` is the third of `OD-POLICY-001`'s refusal cases: an unreadable artifact refuses
    /// rather than reading as empty, because a repository that meant to state a policy and
    /// could not would otherwise get a run that passed under policy nobody authored.
    pub unreadable: Option<String>,
}

impl PolicyContribution
{
    /// A contribution from `layer`'s `artifact` that states nothing.
    ///
    /// The starting point every builder fills in from, so a field added to this type is a
    /// field every builder has to decide about rather than one that silently defaults to
    /// "stated nothing" at a site nobody re-read.
    #[must_use]
    pub fn Silent(layer: ConfigurationLayer, artifact: &str) -> Self
    {
        return Self {
            layer,
            artifact: artifact.to_owned(),
            suppressions: None,
            baseline: None,
            adoption: None,
            coverage: None,
            evidence_floor: None,
            phases: None,
            approvals: None,
            locks: Vec::new(),
            unreadable: None,
        };
    }

    /// Whether this artifact states `field`.
    #[must_use]
    pub fn States(&self, field: PolicyField) -> bool
    {
        return match field
        {
            PolicyField::Suppressions => self.suppressions.is_some(),
            PolicyField::Baseline => self.baseline.is_some(),
            PolicyField::Adoption => self.adoption.is_some(),
            PolicyField::Coverage => self.coverage.is_some(),
            PolicyField::EvidenceFloor => self.evidence_floor.is_some(),
            PolicyField::Phases => self.phases.is_some(),
            PolicyField::Approvals => self.approvals.is_some(),
        };
    }
}
