//! JSON-serializable projections of what `nomos-gate-orchestration` produces for `run`,
//! `plan` and `explain`.
//!
//! `GateRunResult`, `GateRunOutcome`, `GateOutcome`, `Explanation` and its own
//! `RuleCalibration`/`Suppression`/`SuppressionDisposition`/`BaselineDebt`, and
//! `nomos_rules::RuleOffer`, do not derive `Serialize` -- nothing needed a wire format for
//! any of them before this crate existed. Adding it to them directly would grow
//! `nomos-gate-orchestration`'s or `nomos-rules`' own public surface on behalf of one
//! caller's shape, before a second transport exists to check that shape against -- the same
//! premature-surface caution this crate's own module doc names. The types here are that
//! shape, owned in this crate and built by conversion from the borrowed or owned result, so
//! neither orchestration crate changes.
//!
//! Every handler and its own response type share one file below -- [`gate_run_response`] pairs
//! [`Handle_Gate_Run`] with [`GateRunResponse`], [`gate_plan_response`] pairs [`Handle_Gate_Plan`] with
//! [`GatePlanResponse`], [`gate_explain_response`] pairs [`Handle_Gate_Explain`] with
//! [`GateExplainResponse`] -- and every twin type that exists only to serialize one of their
//! own fields gets a file of its own beside them.

mod baseline_debt_response;
mod disposition;
mod gate_explain_response;
mod gate_findings;
mod gate_plan_response;
mod gate_run_response;
mod rule_calibration_response;
mod rule_offer_response;
mod suppression_disposition_response;
mod suppression_response;

pub use baseline_debt_response::BaselineDebtResponse;
pub use disposition::Disposition;
pub use gate_explain_response::{GateExplainResponse, Handle_Gate_Explain};
pub use gate_findings::GateFindings;
pub use gate_plan_response::{GatePlanResponse, Handle_Gate_Plan};
pub use gate_run_response::{GateRunResponse, Handle_Gate_Run};
pub use rule_calibration_response::RuleCalibrationResponse;
pub use rule_offer_response::RuleOfferResponse;
pub use suppression_disposition_response::SuppressionDispositionResponse;
pub use suppression_response::SuppressionResponse;
