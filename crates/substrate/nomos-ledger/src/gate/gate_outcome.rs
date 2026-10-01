//! What the shared gate said, if it was asked.

use serde::Deserialize;
use serde::Serialize;
/// What the derived gate step did, alongside the item's own predicate.
///
/// Recorded rather than merely run. Without it a reader cannot tell an item finished
/// under the gate from one finished before the gate was derived at all, and every
/// `verified` block written earlier would silently read as though it had been checked.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateOutcome
{
    /// What was run, as derived from the workflow.
    pub argv: Vec<String>,
    /// What it exited with.
    pub exit_code: i32,
}
