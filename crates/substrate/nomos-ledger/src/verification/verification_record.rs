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
    #[serde(with = "nomos_platform::timestamp_serde")]
    pub verified_at: Timestamp,
    /// The gate step that ran first, when one could be derived.
    ///
    /// `None` on every record written before the gate was part of finishing. That is a
    /// fact about those records and is left visible rather than backfilled.
    #[serde(default)]
    pub gate: Option<GateOutcome>,
    /// The tree this predicate ran against, as `HEAD` resolved at `verified_at`.
    ///
    /// Same shape as `gate`, deliberately: `None` on every record written before this field
    /// existed, and `None` again when `HEAD` could not be resolved -- no `.git` at the tree
    /// this ran in, a `HEAD` naming a packed ref this build does not chase, or any other read
    /// failure. A reader cannot tell those cases apart from a `None`, and that is deliberate --
    /// none of them is a case this field claims to answer, and inventing a value for any of
    /// them would be worse than admitting it does not have one. Never backfilled, for the
    /// reason `gate` above is not: doing so would manufacture the exact claim this field
    /// exists to stop -- that a tree nobody measured was.
    ///
    /// Identifies the tree by its last commit, not by its content. `OD-LEDGER-027` chose this
    /// over a digest of the item's own territory and over a digest of the whole working tree,
    /// and says there what each of those two would have given up instead.
    #[serde(default)]
    pub revision: Option<String>,
}
