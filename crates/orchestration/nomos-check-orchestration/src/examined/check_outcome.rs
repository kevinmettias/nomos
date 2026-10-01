//! What a `nomos check` run produced.

use nomos_capability::RegistryError;
use nomos_contracts::Finding;

use crate::examined::{Claim, Examined, SupportingFactTrail};

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
        /// Which facts each rule read in the call that produced its findings.
        ///
        /// A field here rather than a second, richer return shape, which is
        /// `OD-HOST-016`'s decision 4 and `OD-HOST-002`'s reason: two shapes make the
        /// richer one the only complete one and every caller taking the thinner one sees a
        /// subset. Walked from a finding through its rule with
        /// [`SupportingFactTrail::Facts_For`], never carried on the finding itself -- a
        /// finding is what a rule says about a subject, and how the rule came to say it is
        /// a property of the run.
        supporting_facts: SupportingFactTrail,
    },
}
