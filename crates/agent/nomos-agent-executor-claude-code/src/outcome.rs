//! What a bounded Claude Code invocation reported, kept apart from what it claimed.

/// What one `Execute` call reported.
///
/// `response` is free text and is never evidence of what happened, only of what the
/// process said — `OD-EXECUTOR-001`'s own amendment measured a run whose `response`
/// falsely claimed a denied write had succeeded. `denied_tool_uses` is the structural
/// signal instead: every tool name the invocation's own permission system actually
/// refused, read off `permission_denials` rather than inferred from prose. A caller
/// that wants to know whether the boundary held checks this field, never `response`.
#[derive(Clone, Debug, PartialEq)]
pub struct AgentExecutionOutcome
{
    pub response: String,
    pub denied_tool_uses: Vec<String>,
    pub is_error: bool,
    pub cost_usd: f64,
    pub duration_ms: u64,
}
