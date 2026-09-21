//! Which real backend, or which canonical check seam, a workflow step's body dispatches
//! through.

mod agent_body;
mod check_body;
mod commit_intent;
mod correction_body;
mod gate_body;

pub use agent_body::AgentBody;
pub use check_body::CheckBody;
pub use commit_intent::CommitIntent;
pub use correction_body::CorrectionBody;
pub use gate_body::GateBody;

use nomos_agent_contracts::TaskEnvelope;

/// Which real backend a workflow step's body dispatches through, and the `TaskEnvelope`
/// it carries -- or, for [`Body::Check`], the tree and rule selection
/// `nomos_check_orchestration::Run` judges.
///
/// Names both of this workspace's real dispatch targets directly — `ClaudeCode`, the one
/// real `AgentExecutor`, and `Ollama`, the one real `ModelBackend` — the same shape
/// `nomos_cli::agent::Backend` already uses for a person's own single call, reused rather
/// than reinvented. Not a trait generic over either: `OD-EXECUTOR-001` and `OD-EXECUTOR-004`
/// both decline a shared dispatch trait ahead of a real need, and `OD-PACKAGE-013` settles
/// `Ollama` as a `ModelBackendPackage` instance dispatched through the same shape rather than
/// a second `AgentExecutor` — this crate does not reach past either restraint.
///
/// `Check` is the same restraint applied to a third dispatch target that shares no trait
/// with either of the first two: `nomos_check_orchestration::Run` is neither an
/// `AgentExecutor` nor a `ModelBackend`, so it gets its own variant rather than being forced
/// through a shape built for the other two. `OD-WORKFLOW-005` named this the real, heavier
/// next step and declined to build it in that increment; `P40-WORKFLOW-CHECK-BODY` is that
/// step.
///
/// `Correction` is the fourth, and the first that can change the tree rather than only
/// report on it: `nomos_correction_orchestration::Run_Correction` judges, plans, stages,
/// validates and -- only when its own body asks for it -- commits a fix, which is what
/// turns Check then Correction then Validate then Gate from a diagram into something this
/// crate can actually run. Sequenced after `Check` deliberately: the body seam should be
/// shaped by more than one case before a mutating body uses it, per
/// `P40-WORKFLOW-CORRECTION-BODY`'s own why.
///
/// `Gate` is the fifth, and the second half of Check then Correction then Validate then
/// Gate: `nomos_gate_orchestration::Run_Gate` is the seam both hosts already call for the
/// judgment this whole system exists to produce. Unlike every body above it, a failing
/// gate run does not merely complete as a `StepOutcome::Gate` -- `crate::run::Dispatch_Body`
/// reports it as a [`crate::DispatchError::Gate`] instead, ending the workflow the same
/// way a real dispatch failure already does, per `P40-WORKFLOW-GATE-BODY`'s own
/// done_when: "a failing gate ends the workflow rather than being reported as a step that
/// merely ran."
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Body
{
    /// Dispatches to whatever this step's declared profile resolves to.
    Agent(AgentBody),
    /// Dispatches through `nomos-check-orchestration::Run`.
    Check(CheckBody),
    /// Dispatches through `nomos-correction-orchestration::Run_Correction`.
    Correction(CorrectionBody),
    /// Dispatches through `nomos-gate-orchestration::Run_Gate`.
    Gate(GateBody),
}
