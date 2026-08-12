//! What was run to finish an item, and what it answered.

use serde::Deserialize;
use serde::Serialize;
use crate::GateOutcome;
use nomos_platform::Timestamp;
/// What happened when the predicate was run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationRecord
{
    /// What was run.
    pub argv: Vec<String>,
    /// What it exited with.
    pub exit_code: i32,
    /// The tail of its output, for a human reading the ledger later.
    pub output_tail: String,
    /// When it ran.
    pub verified_at: Timestamp,
    /// The gate step that ran first, when one could be derived.
    ///
    /// `None` on every record written before the gate was part of finishing. That is a
    /// fact about those records and is left visible rather than backfilled.
    #[serde(default)]
    pub gate: Option<GateOutcome>,
}
