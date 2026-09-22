//! Policy types [`crate::GateCommand`] carries, checked in a fixed order by
//! [`crate::Run_Gate`]: calibration, suppression, baseline, then coverage. Grouped together
//! because every one of them answers the same question -- what should not fail a build
//! despite `Finding::Can_Fail_A_Build` -- and none of them varies independently of that
//! question.

mod baseline_debt;
mod coverage_policy;
mod effective_policy;
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
pub(crate) use gate_policy_file::{Resolve_Gate_Policy, GatePolicyFile};
pub use rule_calibration::{AdoptionPolicy, RuleCalibration};
pub use rule_selector::RuleSelector;
pub use scope_selector::ScopeSelector;
pub use suppression_disposition::{Suppression, SuppressionDisposition, SuppressionPolicy, SuppressionReason, SuppressionStatus};
