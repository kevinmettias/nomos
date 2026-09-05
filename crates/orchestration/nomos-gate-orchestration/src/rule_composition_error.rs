//! Why this crate's own rule composition would be a lie if it returned a registry.

use nomos_check_orchestration::RuleResolutionError;
use nomos_rules::RuleRegistryError;

/// A defect in this crate's own composition, not in anything a caller supplied.
///
/// Two shapes, because `Registered` now answers two questions rather than one. It builds a
/// registry, which can refuse a repeated identity; and since `P52` it also resolves the
/// declared rule set against the implementations `nomos-check-orchestration` composes, which
/// can refuse a disagreement. Collapsing them into one string would lose which of the two
/// happened, and they have different repairs: a duplicate identity is a typo in one table, a
/// disagreement is a rule declared and not implemented or implemented and not declared.
///
/// `Debug` only, no `Display`, matching `RuleRegistryError` and
/// `nomos_check_orchestration::RuleResolutionError` — both consumers of
/// [`crate::GateOutcome::Contradictory`] format this with `{error:?}` today.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuleCompositionError
{
    /// One rule identity was offered to the registry twice.
    Registration(RuleRegistryError),
    /// What this build declares and what it composes are not the same set.
    ///
    /// `OD-RULES-022`'s refusal, reaching a caller. The registry is not returned in this
    /// state on purpose: a plan built from a declaration the run does not implement is
    /// exactly the divergence `nomos gate plan` describing a smaller gate than `nomos gate
    /// run` performs — which happened on 2026-09-05 and was caught by a test rather than
    /// prevented by the composition.
    Resolution(RuleResolutionError),
}

impl From<RuleRegistryError> for RuleCompositionError
{
    fn from(error: RuleRegistryError) -> Self
    {
        return Self::Registration(error);
    }
}

impl From<RuleResolutionError> for RuleCompositionError
{
    fn from(error: RuleResolutionError) -> Self
    {
        return Self::Resolution(error);
    }
}
