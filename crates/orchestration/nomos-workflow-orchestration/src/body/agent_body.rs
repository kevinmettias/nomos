//! A workflow step that runs an agent, and the profile that decides which one.

use nomos_agent_contracts::TaskEnvelope;
use nomos_model_package::ModelExecutionProfile;

/// The task a step runs, and the execution profile that selects what runs it.
///
/// Replaces the two per-backend variants this enum used to carry. Those made the variant a
/// step was written as its backend choice: dispatch matched the variant straight to an
/// adapter crate, so nothing resolved anything and a step could not express "whatever
/// answers to this family" at all.
///
/// `OD-PACKAGE-016` decision 9's wiring is that a step declares what it wants and the
/// declared set decides what answers. So the profile rides here, beside the task, and
/// `nomos_agent_orchestration::Run_Agent_Task` is what turns it into a backend -- or into an
/// absence, if the declared set offers nothing that answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentBody
{
    /// What the step asks the agent to do.
    pub task: TaskEnvelope,
    /// What the step declares about which model execution should answer it.
    pub profile: ModelExecutionProfile,
}
