//! `CorrectionCandidate`, `CorrectionPlan`, and the deterministic preview, stage, validate,
//! commit and rollback lifecycle over a workspace change.
//!
//! No agent and no model backend appears anywhere in this crate. Every step is a pure
//! function of a plan and a [`nomos_workspace::Workspace`]'s current state: what a
//! candidate would do is decided before this crate exists to run it, and the lifecycle
//! here only ever checks that state has not moved and submits through
//! [`nomos_workspace::Workspace`]'s one door.
#![forbid(unsafe_code)]
// PROBE

mod candidate;
mod change_set;
mod committed;
mod determinism;
mod edit;
mod error;
mod plan;
mod preview;
mod staged;
mod validated;

pub use candidate::{CorrectionCandidate, CorrectionId};
pub use change_set::ChangeSet;
pub use committed::CommittedPlan;
pub use determinism::CorrectionStaging;
pub use edit::Edit;
pub use error::CorrectionError;
pub use plan::CorrectionPlan;
pub use preview::Preview;
pub use staged::StagedPlan;
pub use validated::ValidatedPlan;
