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
//!
//! The same boundary holds one level up, between plans. [`Compatibility`] and
//! [`WavePartition`] answer whether several plans may be staged together and in what
//! order, computed only from the read and write sets the plans' candidates declare
//! (`COR-EXEC-001`, `COR-EXEC-003`) -- never by opening a file or diffing content. A plan
//! whose declared sets cannot be trusted to be complete is refused as
//! [`UnresolvedAccess`] rather than placed, because unknown independence is not safe
//! parallelism. Nothing here runs a wave: that is orchestration, and this crate stays the
//! substrate it computes from.
//!
//! [`StateSignature`] and [`ConvergenceTrail`] are that same substrate for `COR-005`'s
//! "compare state signatures" and `COR-006`'s repeated-state detection: a signature
//! reduces one observation of an affected scope to a comparable value, and a trail says
//! whether the state just observed is one this run has already been in. Neither reruns
//! anything or decides what an observation is -- a caller that reran the scope hands in
//! what it saw, exactly as a caller that resolved a read/write set hands in what it
//! declared.
#![forbid(unsafe_code)]

mod correction_id;
mod candidate_label;
mod change_set;
mod choice_record;
mod committed_plan;
mod compatibility;
mod convergence_trail;
mod correction_choice;
mod correction_class;
mod correction_decision;
mod correction_staging;
mod edit;
mod correction_error;
mod correction_plan;
mod plan_access;
mod preview;
mod ranking_criterion;
mod read_write_resolution;
mod rollback_boundary;
mod staged_plan;
mod state_signature;
#[cfg(test)]
mod test_support;
mod validated_plan;
mod wave_partition;

pub use correction_id::{CorrectionCandidate, CorrectionId};
pub use candidate_label::CandidateLabel;
pub use change_set::ChangeSet;
pub use choice_record::ChoiceRecord;
pub use compatibility::{Compatibility, Overlap, OverlapClass};
pub use convergence_trail::ConvergenceTrail;
pub use correction_choice::CorrectionChoice;
pub use correction_class::CorrectionClass;
pub use correction_decision::{CorrectionDecision, ReviewReason};
pub use committed_plan::CommittedPlan;
pub use correction_staging::CorrectionStaging;
pub use edit::Edit;
pub use correction_error::CorrectionError;
pub use correction_plan::CorrectionPlan;
pub use plan_access::UnresolvedAccess;
pub use preview::Preview;
pub use ranking_criterion::RankingCriterion;
pub use read_write_resolution::{DerivedProvenance, ReadWriteResolution, ReadWriteSet};
pub use rollback_boundary::RollbackBoundary;
pub use staged_plan::StagedPlan;
pub use state_signature::StateSignature;
pub use validated_plan::ValidatedPlan;
pub use wave_partition::WavePartition;
