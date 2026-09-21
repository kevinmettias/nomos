//! What dispatching one `TaskEnvelope` to a chosen [`crate::Backend`] produced.

/// What one [`crate::Run_Agent_Execute`] or [`crate::Run_Agent_Judgment`] call reported.
///
/// Names each backend's own outcome type directly, the same no-shared-trait shape
/// `nomos_workflow_orchestration::StepOutcome` already uses for the identical reason: the
/// two crates' outcome shapes are not interchangeable. `ClaudeCode` carries
/// `denied_tool_uses`, `is_error`, `cost` and `duration_ms`; `Ollama` honestly does
/// not have any of those -- `nomos-model-backend-ollama`'s own module doc says why, and
/// this type does not fabricate them to make the two variants look more alike than they
/// are. `Unavailable` folds both backends' own error type down to the text either one's
/// `Display` already produces, the identical collapse `nomos-cli`'s own former
/// `Backend_Unavailable` made: a caller that reached this variant already knows which
/// backend it asked for, and the old rendering never repeated that choice back either.
#[derive(Clone, Debug, PartialEq)]
pub enum AgentDispatchOutcome
{
    /// What `nomos-agent-executor-claude-code::Execute_Task` reported.
    ClaudeCode(nomos_agent_executor_claude_code::AgentExecutionOutcome),
    /// What `nomos-model-backend-ollama::Execute_Task` reported.
    Ollama(nomos_model_backend_ollama::AgentExecutionOutcome),
    /// The chosen backend could not be started, exited non-zero, timed out, stalled, or
    /// (Claude Code only) answered with something other than the JSON `--output-format
    /// json` promises.
    Unavailable(String),
    /// No backend was selected, so nothing ran.
    ///
    /// Deliberately not [`Self::Unavailable`]. That one reports a backend that was chosen and
    /// then did not answer; this one reports that nothing was chosen at all, and the two
    /// have different remedies -- one is a backend to fix, the other is a declaration to
    /// change. Folding them together is how "nothing could look" comes to read as "nothing
    /// was wrong", which `OD-GATE-001` is about one layer out.
    NotSelected(crate::BackendAbsence),
}
