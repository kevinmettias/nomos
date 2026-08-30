//! Band 36 — the typed envelope an agent-assisted operation carries in, and the typed
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

#![forbid(unsafe_code)]

mod isolated_working_directory_error;
mod nomos_resolved_change_context;
mod task_envelope;
mod work_result;

pub use isolated_working_directory_error::{Isolated_Working_Directory, IsolatedWorkingDirectoryError};
pub use nomos_resolved_change_context::{ChangeContextSubject, NomosResolvedChangeContext};
pub use task_envelope::TaskEnvelope;
pub use work_result::WorkResult;
