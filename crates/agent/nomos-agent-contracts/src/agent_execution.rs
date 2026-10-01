//! What an `AgentExecutorPackage` establishes by running one task.

use crate::WorkResult;
use xvpe_agent_execution::MicroDollars;

/// What one [`crate::AgentExecutor`] dispatch established.
///
/// The shape belongs to the *package kind*, not to a vendor. `OD-EXECUTOR-001` decided that
/// an agent executor's dispatch is bounded by a spend ceiling, a wall bound, an allow-list
/// and schema-validated output, and these five fields are what a caller can read back off
/// that boundary afterwards: what the model judged ([`WorkResult`]), what its own permission
/// system structurally refused, whether it reported failure, what it spent, and how long it
/// took.
///
/// [`crate::ModelAnswer`] is deliberately *not* this type with some fields left empty. A
/// model backend establishes none of `denied_tool_uses`, `is_error`, `spend` or
/// `duration_ms` -- it has no tool subsystem to deny into and no metered charge to report --
/// and a shared shape carrying them would make an absent measurement indistinguishable from
/// a measured zero. The two ports return two types for that reason, so the question "what
/// did this model backend cost" has no expression rather than a fabricated answer.
#[derive(Clone, Debug, PartialEq)]
pub struct AgentExecution
{
    /// What the executor judged, read only from schema-validated structured output.
    pub result: WorkResult,
    /// Every tool name the invocation's own permission system actually refused.
    pub denied_tool_uses: Vec<String>,
    /// Whether the executor itself reported the turn as failed.
    pub is_error: bool,
    /// What the dispatch cost, exactly as the engine measured it.
    ///
    /// [`MicroDollars`] rather than a dollar figure, so a reported cost and the ceiling it
    /// is bounded by are the same kind of number. The float belongs to the wire types that
    /// publish a `cost_usd`, and is made there.
    pub spend: MicroDollars,
    /// How long the dispatch took, as the engine measured it.
    pub duration_ms: u64,
}
