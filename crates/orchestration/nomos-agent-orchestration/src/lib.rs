//! Zone: Application Service — the seam for dispatching one `TaskEnvelope` to a resolved
//! dispatch target both hosts call.
//!
//! Every other verb family this workspace runs through more than one composition root
//! already has an orchestration crate between it and `nomos-cli`: `nomos-work-
//! orchestration`, `nomos-spec-orchestration`, `nomos-check-orchestration`, `nomos-gate-
//! orchestration`, `nomos-correction-orchestration` and `nomos-workflow-orchestration`.
//! `nomos agent`'s own dispatch -- and building the two `TaskEnvelope` shapes this workspace
//! actually constructs -- lived only in `nomos-cli`'s own `agent.rs` and its `agent/`
//! directory, so `nomos-api` had no way to dispatch an agent task, choose a backend, or
//! read a result at all. `P43-AGENT-CANONICAL-SEAM-2` is that gap closed, the identical
//! shape `P40-CORRECTIONS-CANONICAL-SEAM` already used for `correct.rs`.
//!
//! # This crate names a port, and no backend
//!
//! It used to name two. `nomos-agent-executor-claude-code` and `nomos-model-backend-ollama`
//! were both manifest dependencies here; `run::Dispatched_Task` matched a two-variant
//! `Backend` enum declared here onto their `Execute_Task` functions, `AgentDispatchOutcome`
//! carried each crate's own outcome type as a variant named for its vendor, and
//! `Declared_Targets` listed exactly those two. An external architecture review called that a
//! plugin-boundary leak -- the generic path knew Claude Code and Ollama rather than knowing an
//! executor -- and `OD-ROADMAP-005` decision 2 authorized closing it.
//!
//! So: `nomos-agent-contracts` holds two ports, one per `PackageKind`
//! (`nomos_agent_contracts::AgentExecutor` and `nomos_agent_contracts::ModelBackend`); each
//! adapter implements the one that matches its own mechanism and declares its own
//! `nomos_agent_contracts::DeclaredTarget`; and a composition root offers those declarations
//! to [`BackendSelection`]. This crate resolves and dispatches, and constructs neither adapter.
//!
//! **Two ports rather than one, because the two answers are not interchangeable.**
//! `OD-EXECUTOR-005` measured that an agent executor and a model backend answer different
//! questions -- which is why `--executor` and `--model-backend` are separate flags -- and
//! their answers differ accordingly: an execution carries a denial list, an error flag, a
//! spend and a duration; a model answer carries a response and none of those, because the
//! mechanism establishes none of them. A single port would have had to return one shape, and
//! any shape wide enough for both would report an absent measurement as a measured zero.
//! Keeping two return types is what makes that impossible rather than merely discouraged.
//!
//! [`Run_Agent_Execute`] moved from `nomos-cli`'s own `agent/dispatch.rs`
//! (`Execute_Goal`/`Dispatch_Task`/`Execute_Task`): a person's own direct question, bounded
//! and tool-free, with no `RuleId` or `SubjectId` to report a `Finding` against.
//! [`Run_Agent_Judgment`] moved from `agent/judge_role.rs` (`Judgment_Task` plus the same
//! shared dispatch): the question `nomos_rules::Check_Declared_Role_Matches_Surface`
//! cannot answer for itself, already judged and already paired by its own caller before
//! this seam ever sees it -- reading a crate's `README.md` row and its committed surface
//! snapshot stays a composition root's own concern (`OD-HOST-002`), the identical division
//! `nomos-correction-orchestration` draws around walking a tree. Both end at the same
//! private dispatch, `run::Dispatched_Task` (not exported: neither `nomos-cli` nor `nomos-api`
//! needs a bare `TaskEnvelope` in without one of the shapes above building it first).
//!
//! [`run`]'s own module doc states why no function here is generic over a platform any more:
//! an adapter behind a port already holds whatever launcher a composition root bound into it,
//! so `AgentEnvironment` -- whose one field was that launcher -- went with the parameter.
//!
//! `nomos-cli`'s `agent` module and `nomos-api`'s own agent surface both call
//! [`Run_Agent_Execute`]/[`Run_Agent_Judgment`]; neither owns this dispatch any more.
//!
//! [`Resolve_Profile`] is a third, independent seam: which declared target a declared
//! [`nomos_model_package::ModelExecutionProfile`] resolves to, against a package set the caller
//! declares. It chooses; it dispatches nothing, so it neither calls [`Run_Agent_Execute`] nor is
//! called by it -- a composition root that has a profile resolves it and then dispatches the
//! [`DispatchConfig`] it produced. `OD-PACKAGE-016` decides it and
//! `profile_resolution`'s own module doc states what resolves and what deliberately does not.
//!
//! [`validated_correction`] is a second, independent seam this crate owns: carrying a
//! [`nomos_agent_contracts::WorkResult`]'s own `plan`, once dispatch has produced one,
//! through `nomos-corrections`' `Preview -> Stage -> Validate -> Commit` lifecycle. It does
//! not call [`run`] and [`run`] does not call it -- a caller that dispatches a task and
//! then wants its plan carried through calls both seams itself, in that order.

#![forbid(unsafe_code)]

mod agent_dispatch_outcome;
mod backend_absence;
mod backend_selection;
mod dispatch_config;
mod profile_resolution;
mod run;
mod validated_correction;
mod validated_correction_outcome;

pub use agent_dispatch_outcome::AgentDispatchOutcome;
pub use backend_absence::BackendAbsence;
pub use backend_selection::{BackendSelection, Selected_Dispatch};
pub use dispatch_config::DispatchConfig;
pub use profile_resolution::{ProfileAbsence, ProfileResolution, Resolve_Profile};
pub use run::{Run_Agent_Execute, Run_Agent_Judgment, Run_Agent_Task};
pub use validated_correction::Run_Validated_Correction;
pub use validated_correction_outcome::ValidatedCorrectionOutcome;

#[cfg(test)]
mod test_support;
