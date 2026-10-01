//! What dispatching one `TaskEnvelope` through a resolved port produced.

use nomos_agent_contracts::{AgentExecution, ModelAnswer};

/// What one [`crate::Run_Agent_Execute`], [`crate::Run_Agent_Judgment`] or
/// [`crate::Run_Agent_Task`] call reported.
///
/// The two answering variants are named for the **package kind** that answered, not for the
/// vendor that implements it. There used to be one variant per adapter crate, each carrying
/// that adapter's own outcome type, which is the plugin-boundary leak `OD-ROADMAP-005`
/// decision 2 closes: this crate named two vendors and carried their own types, so the generic
/// path knew which two products existed rather than knowing an executor and a model backend.
///
/// **The two shapes are still not interchangeable, and that is deliberate.**
/// [`AgentExecution`] carries `denied_tool_uses`, `is_error`, `spend` and `duration_ms`;
/// [`ModelAnswer`] honestly carries none of those, because the mechanism `OD-EXECUTOR-004`
/// measured establishes none of them. Nothing here fabricates them to make the two variants
/// look more alike than they are -- and since `OD-ROADMAP-005` that is a property of the port
/// types rather than of this enum's restraint: a caller cannot ask a [`ModelAnswer`] for a
/// spend, because the type has no such field and
/// `nomos-agent-contracts`' own test stops compiling if one is added.
///
/// Every variant that reached a backend names the family that answered. Before the port, the
/// variant *was* the vendor, so a caller always knew; now that a profile can resolve to a
/// family nobody typed, "which backend answered" is an answer the resolution has and the
/// caller does not.
#[derive(Clone, Debug, PartialEq)]
pub enum AgentDispatchOutcome
{
    /// An `AgentExecutorPackage` answered, and what it established.
    Executed
    {
        /// The family label of the target that answered.
        family: String,
        /// What the dispatch established.
        execution: AgentExecution,
    },
    /// A `ModelBackendPackage` answered, and what it said.
    Answered
    {
        /// The family label of the target that answered.
        family: String,
        /// What the model said, which is not a report of what it did.
        answer: ModelAnswer,
    },
    /// The resolved backend could not be started, exited non-zero, timed out, stalled, or
    /// answered with something its own reader refused.
    ///
    /// Folds the refusing adapter's own error type down to the text its `Display` produces,
    /// the same collapse this variant has always made. It names the family as well, which it
    /// did not before: the old doc comment's reason -- that a caller reaching this variant
    /// already knows which backend it asked for -- stopped being true once a dispatch could
    /// resolve a backend nobody named.
    Unavailable
    {
        /// The family label of the target that was reached and did not answer.
        family: String,
        /// The reason, as the refusing adapter stated it.
        reason: String,
    },
    /// No backend was selected, so nothing ran.
    ///
    /// Deliberately not [`Self::Unavailable`]. That one reports a backend that was chosen and
    /// then did not answer; this one reports that nothing was chosen at all, and the two
    /// have different remedies -- one is a backend to fix, the other is a declaration to
    /// change. Folding them together is how "nothing could look" comes to read as "nothing
    /// was wrong", which `OD-GATE-001` is about one layer out. It carries no family, because
    /// there is no target this could be about.
    NotSelected(crate::BackendAbsence),
}
