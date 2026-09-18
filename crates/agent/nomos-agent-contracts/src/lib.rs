//! Zone: Agent — the typed envelope an agent-assisted operation carries in, and the typed
//! result it carries out.
//!
//! `AGT-001` and `AGT-002` each name their own artifact (`TaskEnvelope`, `WorkResult`)
//! and each already has real, scattered analogs across `nomos-ledger`, `nomos-contracts`
//! and `nomos-corrections` -- `tests/contract/requirements/AGT-001.assessment` and
//! `AGT-002.assessment` both confirm the pieces exist and that nothing bundles them into
//! one artifact. This crate is that bundling, and nothing else: every field here is a
//! direct reuse of a type that already means what the corpus word means, and neither
//! type computes anything -- an agent's own caller fills a `TaskEnvelope` in, and an
//! agent's own response fills a `WorkResult` in.
//!
//! The one exception to "nothing else" is [`Substantiation`] and the two types it is built
//! from, which are this crate's own vocabulary rather than a reuse: `OD-EXECUTOR-011` found
//! that a `WorkResult` could not say which of its empty portions meant the task produced none
//! and which meant the producing executor could never have grounded it, so the distinction is
//! now a value the result carries rather than a sentence in the executor's doc comment.

#![forbid(unsafe_code)]

mod isolated_working_directory_error;
mod nomos_resolved_change_context;
mod portion_substantiation;
mod substantiation;
mod task_envelope;
mod unsubstantiated_reason;
mod work_result;

pub use isolated_working_directory_error::{Isolated_Working_Directory, IsolatedWorkingDirectoryError};
pub use nomos_resolved_change_context::{ChangeContextSubject, NomosResolvedChangeContext};
pub use portion_substantiation::PortionSubstantiation;
pub use substantiation::Substantiation;
pub use task_envelope::TaskEnvelope;
pub use unsubstantiated_reason::UnsubstantiatedReason;
pub use work_result::WorkResult;
