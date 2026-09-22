//! The three things one agent dispatch carries once a profile has resolved -- how much
//! effort it is allowed, which family answers it, and the port that answers -- grouped into a
//! single value.

use nomos_agent_contracts::DispatchPort;
use nomos_model_package::EffortLevel;

/// What a resolved dispatch runs with: the effort the profile asked for, carried unchanged,
/// the family label of the target that answers, and the port it answers through.
///
/// `port` replaces the two-variant `Backend` enum this type used to carry. That enum named
/// `nomos-agent-executor-claude-code` and `nomos-model-backend-ollama` in its own variants,
/// so a resolved choice was a vendor and the generic path held the list of them.
/// `OD-ROADMAP-005` decision 2: a resolved choice is now the thing that answers, supplied by
/// a composition root, and `family` is the label the declaration gave it.
///
/// Grouped because every caller threads these together, never one without the others -- the
/// same reason `nomos_cli::agent::DispatchConfig` grouped effort and backend before this
/// crate existed. Also what keeps [`crate::Run_Agent_Judgment`] inside this workspace's
/// `parameter-count` limit.
#[derive(Clone, Copy)]
pub struct DispatchConfig<'port>
{
    pub effort: EffortLevel,
    pub family: &'port str,
    pub port: DispatchPort<'port>,
}
