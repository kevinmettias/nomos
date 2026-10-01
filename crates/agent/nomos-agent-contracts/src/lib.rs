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
//!
//! # The two ports, and why there are two
//!
//! `OD-ROADMAP-005` decision 2 put a port between the generic agent path and its concrete
//! backends, and this crate holds it: [`AgentExecutor`] is the port an
//! `AgentExecutorPackage` answers through, [`ModelBackend`] the port a
//! `ModelBackendPackage` answers through, and a [`DeclaredTarget`] a composition root
//! declares carries whichever [`DispatchPort`] answers it. Before that decision,
//! `nomos-agent-orchestration` named `nomos-agent-executor-claude-code` and
//! `nomos-model-backend-ollama` directly and matched an enum of the two onto their
//! functions.
//!
//! **One port per `PackageKind`, not one port over both.** `OD-EXECUTOR-005` measured that
//! an agent executor and a model backend answer different questions, which is why
//! `--executor` and `--model-backend` are separate flags, and `OD-PACKAGE-013` named the
//! two kinds. Their answers are correspondingly different: [`AgentExecution`] carries what
//! a bounded, tool-aware turn established, and [`ModelAnswer`] carries a response and
//! nothing else. Flattening them into one shape would have made an absent measurement read
//! as a measured zero, so the ports keep them apart and
//! `model_answer.rs`'s own test makes that structural rather than editorial.

#![forbid(unsafe_code)]

mod agent_execution;
mod agent_executor;
mod declared_target;
mod dispatch_port;
mod dispatch_refusal;
mod isolated_working_directory_error;
mod model_answer;
mod model_backend;
mod nomos_resolved_change_context;
mod portion_substantiation;
mod substantiation;
mod task_envelope;
mod unsubstantiated_reason;
mod work_result;

pub use agent_execution::AgentExecution;
pub use agent_executor::AgentExecutor;
pub use declared_target::DeclaredTarget;
pub use dispatch_port::DispatchPort;
pub use dispatch_refusal::DispatchRefusal;
pub use isolated_working_directory_error::{Isolated_Working_Directory, IsolatedWorkingDirectoryError};
pub use model_answer::ModelAnswer;
pub use model_backend::ModelBackend;
pub use nomos_resolved_change_context::{ChangeContextSubject, NomosResolvedChangeContext};
pub use portion_substantiation::PortionSubstantiation;
pub use substantiation::Substantiation;
pub use task_envelope::TaskEnvelope;
pub use unsubstantiated_reason::UnsubstantiatedReason;
pub use work_result::WorkResult;
// The engine's exact money, re-exported so a caller reading `AgentExecution::spend` or
// comparing one against an adapter's own ceiling can name its type without taking a
// dependency on the engine crate that declares it. It was re-exported from
// `nomos-agent-executor-claude-code` while that crate owned the only outcome carrying a
// spend; the port owns that shape now, so the re-export moved with it and that crate
// re-exports this one rather than declaring a second origin for the same type.
pub use xvpe_agent_execution::MicroDollars;
