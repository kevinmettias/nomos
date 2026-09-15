//! The two choices one agent dispatch carries -- how much effort it is allowed and which
//! backend answers it -- grouped into a single value.

use crate::Backend;
use nomos_model_package::EffortLevel;

/// The effort and backend a dispatch carries, together -- grouped because every caller of
/// [`crate::Run_Agent_Execute`]/[`crate::Run_Agent_Judgment`] threads them together, never one
/// without the other, the same reason `nomos_cli::agent::DispatchConfig` grouped them before
/// this crate existed. Also what keeps `Run_Agent_Judgment` inside this crate's own
/// `parameter-count` limit: `pair`, `finding`, `environment` and this one value are four, not
/// five.
#[derive(Clone, Copy)]
pub struct DispatchConfig
{
    pub effort: EffortLevel,
    pub backend: Backend,
}
