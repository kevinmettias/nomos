//! What a bounded Claude Code invocation reported, kept apart from what it claimed.

use nomos_agent_contracts::WorkResult;
use xvpe_agent_execution::MicroDollars;

/// What one `Execute` call reported.
///
/// `result` is a real [`WorkResult`], built only from Claude Code's own schema-validated
/// `structured_output` and never from its free-text `result` field --
/// `OD-EXECUTOR-008`'s own decision, and the reason this field is not named `response`
/// carrying prose the way it once did: `OD-EXECUTOR-001`'s own amendment measured a run
/// whose free text falsely claimed a denied write had succeeded, and a schema-validated
/// judgment field is not immune to that same risk if it were read as evidence of what
/// happened rather than what the model judged. `denied_tool_uses` is the structural
/// signal for whether the boundary held: every tool name the invocation's own permission
/// system actually refused, read off `permission_denials` rather than inferred from
/// prose or from `result`.
#[derive(Clone, Debug, PartialEq)]
pub struct AgentExecutionOutcome
{
    pub result: WorkResult,
    pub denied_tool_uses: Vec<String>,
    pub is_error: bool,
    /// What the dispatch cost, exactly as the engine measured it.
    ///
    /// [`MicroDollars`] rather than a dollar figure, and that is the whole of the
    /// difference: the engine measures money as an integer and this crate's own
    /// [`crate::MAXIMUM_SPEND`] is one, so a float here would make the reported cost and
    /// the cap it is bounded by two incomparable kinds of number. The float belongs to the
    /// wire types that publish a `cost_usd`, and is made there.
    pub cost: MicroDollars,
    pub duration_ms: u64,
}
