//! Policy types [`crate::GateCommand`] carries, checked in a fixed order by
//! [`crate::Run_Gate`]: the evidence floor, then calibration, suppression, baseline, then
//! coverage. Grouped together because every one of them answers the same question -- what
//! should not fail a build despite `Finding::Can_Fail_A_Build` -- and none of them varies
//! independently of that question.
//!
//! [`EvidenceFloor`] is checked first and is the one of them that is not a disposition
//! somebody authored about a finding: `OD-GATE-034` puts it beside `Can_Fail_A_Build`'s own
//! two conditions, as a third fact about the finding itself, so the three matchers below it
//! never see a finding the floor already took.

mod baseline_debt;
mod coverage_policy;
mod effective_policy;
mod evidence_floor;
mod gate_policy_file;
mod rule_calibration;
mod rule_selector;
mod scope_selector;
mod suppression_disposition;

pub use baseline_debt::{BaselineAllowance, BaselineDebt, BaselinePolicy};
pub use coverage_policy::CoveragePolicy;
pub use effective_policy::{
    Effective_Gate_Policy, EffectivePolicy, FieldProvenance, PHASE_POLICY_UNIT, PolicyContribution, PolicyField, PolicyRefusal, PolicyUnit,
    RejectedOverride, ResolvedField,
};
pub(crate) use effective_policy::Resolved_Gate_Policy;
pub use evidence_floor::EvidenceFloor;
pub(crate) use gate_policy_file::{Resolve_Gate_Policy, GatePolicyFile};
pub use rule_calibration::{AdoptionPolicy, RuleCalibration};
pub use rule_selector::RuleSelector;
pub use scope_selector::ScopeSelector;
pub use suppression_disposition::{Suppression, SuppressionDisposition, SuppressionPolicy, SuppressionReason, SuppressionStatus};
