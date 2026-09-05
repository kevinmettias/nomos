//! What a `nomos check` run produced.

use nomos_capability::RegistryError;
use nomos_contracts::Finding;

use crate::examined::{Claim, Examined};

/// What a `nomos check` run produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckOutcome
{
    /// The root does not exist or is not a directory, or the walked source could not be
    /// ingested as a workspace state. Nothing was judged.
    Unreadable,
    /// This build's own capability registry is self-contradictory -- a defect in the
    /// composition, not in the tree being checked.
    Contradictory(RegistryError),
    /// The walk found no source under the root.
    NoSource,
    /// Source was found but no syntax fact was materialized for any of it, so no mirror
    /// claim could be resolved. The same lie as [`CheckOutcome::NoSource`], one layer in.
    NoFacts
    {
        /// Files the walk read, none of which produced a fact.
        files: usize,
    },
    /// The rules ran over a real fact store.
    Judged
    {
        /// What the rules found.
        findings: Vec<Finding>,
        /// How much of the world this run actually saw.
        examined: Examined,
        /// Whether this run reached a judgment about everything it touched.
        claim: Claim,
    },
}
