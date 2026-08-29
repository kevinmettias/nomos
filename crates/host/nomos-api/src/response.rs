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
//! Every handler and its own response type share one file below -- [`run`] pairs
//! [`Handle_Gate_Run`] with [`GateRunResponse`], [`plan`] pairs [`Handle_Gate_Plan`] with
//! [`GatePlanResponse`], [`explain`] pairs [`Handle_Gate_Explain`] with
//! [`GateExplainResponse`] -- and every twin type that exists only to serialize one of their
//! own fields gets a file of its own beside them.

mod baseline_debt;
mod disposition;
mod explain;
mod gate_findings;
mod plan;
mod rule_calibration;
mod rule_offer;
mod run;
mod suppression;
mod suppression_disposition;

pub use baseline_debt::BaselineDebtResponse;
pub use disposition::Disposition;
pub use explain::{GateExplainResponse, Handle_Gate_Explain};
pub use gate_findings::GateFindings;
pub use plan::{GatePlanResponse, Handle_Gate_Plan};
pub use rule_calibration::RuleCalibrationResponse;
pub use rule_offer::RuleOfferResponse;
pub use run::{GateRunResponse, Handle_Gate_Run};
pub use suppression::SuppressionResponse;
pub use suppression_disposition::SuppressionDispositionResponse;
