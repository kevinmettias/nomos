//! Which real backend, or which canonical check seam, a workflow step's body dispatches
//! through.

mod check_body;
mod correction_body;

pub use check_body::CheckBody;
pub use correction_body::CorrectionBody;

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Body
{
    /// Dispatches through `nomos-agent-executor-claude-code`.
    ClaudeCode(TaskEnvelope),
    /// Dispatches through `nomos-model-backend-ollama`.
    Ollama(TaskEnvelope),
    /// Dispatches through `nomos-check-orchestration::Run`.
    Check(CheckBody),
    /// Dispatches through `nomos-correction-orchestration::Run_Correction`.
    Correction(CorrectionBody),
}
