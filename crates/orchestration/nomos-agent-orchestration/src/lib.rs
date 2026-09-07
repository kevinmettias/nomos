//! Zone: Application Service — the seam for dispatching one `TaskEnvelope` to a chosen
//! backend both hosts call.
//!
//! Every other verb family this workspace runs through more than one composition root
//! already has an orchestration crate between it and `nomos-cli`: `nomos-work-
//! orchestration`, `nomos-spec-orchestration`, `nomos-check-orchestration`, `nomos-gate-
//! orchestration`, `nomos-correction-orchestration` and `nomos-workflow-orchestration`.
//! `nomos agent`'s own dispatch -- choosing between `nomos-agent-executor-claude-code` and
//! `nomos-model-backend-ollama`, and building the two `TaskEnvelope` shapes this workspace
//! actually constructs -- lived only in `nomos-cli`'s own `agent.rs` and its `agent/`
//! directory, so `nomos-api` had no way to dispatch an agent task, choose a backend, or
//! read a result at all. `P43-AGENT-CANONICAL-SEAM-2` is that gap closed, the identical
//! shape `P40-CORRECTIONS-CANONICAL-SEAM` already used for `correct.rs`.
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
//! private dispatch, [`run::Dispatched`] (not exported: neither `nomos-cli` nor `nomos-api`
//! needs a bare `TaskEnvelope` in without one of the two shapes above building it first).
//!
//! [`run`]'s own module doc states the one deliberate departure from what it replaces:
//! `nomos-cli`'s former `Dispatch_Task` was fixed to `nomos_platform_std::
//! StdProcessLauncher` rather than generic, correct advice while this dispatch had exactly
//! one caller. [`Run_Agent_Execute`] and [`Run_Agent_Judgment`] are generic over
//! [`nomos_platform::ProcessLauncher`] instead, the identical reason
//! `nomos_correction_orchestration::Run_Correction` already is -- so `nomos-api` supplies
//! its own launcher at its own call site rather than depending on `nomos-cli`'s choice, or
//! on `nomos-cli` at all -- and, as a direct consequence, this crate's own tests reach both
//! backends with a scripted launcher instead of needing the `// check-test-coverage:
//! allow-untested` exclusion `dispatch.rs`'s own two match arms carried for exactly that
//! reason.
//!
//! Does not invent a shared trait between `nomos-agent-executor-claude-code` and
//! `nomos-model-backend-ollama`: `OD-EXECUTOR-001`/`OD-EXECUTOR-004`/`OD-EXECUTOR-005` all
//! decline one ahead of a real second `AgentExecutor`, and [`run::Dispatched`]'s own match
//! is that restraint carried forward unchanged, not a stand-in for a trait neither crate
//! needs yet.
//!
//! `nomos-cli`'s `agent` module and `nomos-api`'s own agent surface both call
//! [`Run_Agent_Execute`]/[`Run_Agent_Judgment`] now; neither owns this dispatch any more.
//!
//! [`validated_correction`] is a second, independent seam this crate owns: carrying a
//! [`nomos_agent_contracts::WorkResult`]'s own `plan`, once dispatch has produced one,
//! through `nomos-corrections`' `Preview -> Stage -> Validate -> Commit` lifecycle. It does
//! not call [`run`] and [`run`] does not call it -- a caller that dispatches a task and
//! then wants its plan carried through calls both seams itself, in that order.

#![forbid(unsafe_code)]

mod agent_dispatch_outcome;
mod backend;
mod run;
mod validated_correction;

pub use agent_dispatch_outcome::AgentDispatchOutcome;
pub use backend::Backend;
pub use run::{AgentEnvironment, DispatchConfig, Run_Agent_Execute, Run_Agent_Judgment};
pub use validated_correction::{Run_Validated_Correction, ValidatedCorrectionOutcome};
