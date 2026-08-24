//! `CorrectionCandidate`, `CorrectionPlan`, and the deterministic preview, stage, validate,
//! commit and rollback lifecycle over a workspace change.
//!
//! No agent and no model backend decides anything in this crate. Every step is a pure
//! function of a plan and a [`nomos_workspace::Workspace`]'s current state: what a
//! candidate would do is decided before this crate exists to run it, and the lifecycle
//! here only ever checks that state has not moved and submits through
//! [`nomos_workspace::Workspace`]'s one door.
//!
//! `ValidatedPlan::Commit`'s [`nomos_model::Evidence`] parameter, `OD-CORRECTIONS-002`,
//! does not change that. This crate does not judge the evidence it is handed -- it does
//! not decide what counts as evidence, and does not refuse a commit for the evidence being
//! weak, only for the workspace having moved. It only stops being silent about how the
//! caller says it knows, floored at [`nomos_model::EvidenceClass::AgentJudged`] when
//! nothing stronger is offered.
#![forbid(unsafe_code)]

mod candidate;
mod candidate_label;
mod change_set;
mod committed;
mod correction_choice;
mod correction_class;
mod correction_decision;
mod determinism;
mod edit;
mod error;
mod plan;
mod preview;
mod ranking_criterion;
mod staged;
mod validated;

pub use candidate::{CorrectionCandidate, CorrectionId};
pub use candidate_label::CandidateLabel;
pub use change_set::ChangeSet;
pub use correction_choice::CorrectionChoice;
pub use correction_class::CorrectionClass;
pub use correction_decision::{CorrectionDecision, ReviewReason};
pub use committed::CommittedPlan;
pub use determinism::CorrectionStaging;
pub use edit::Edit;
pub use error::CorrectionError;
pub use plan::CorrectionPlan;
pub use preview::Preview;
pub use ranking_criterion::RankingCriterion;
pub use staged::StagedPlan;
pub use validated::ValidatedPlan;
