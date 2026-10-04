//! What a `nomos check` run produced.

use nomos_capability::RegistryError;
use nomos_contracts::Finding;

use crate::examined::{Claim, Examined, Populations, SupportingFactTrail, UndeclaredValues};

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
        /// How many sources each selected rule was judged over, in its declared population.
        ///
        /// Beside [`Self::Judged::claim`] and never in it, as `OD-ANALYSIS-012` version 2
        /// decided: an empty population is reported here, and the claim is what the findings
        /// alone support. [`Populations::Empty`] names the rules that judged nothing.
        populations: Populations,
        /// Every value a selected rule judged over a nonempty population read that the repository
        /// did not declare, and what the rule did with it.
        ///
        /// Beside [`Self::Judged::claim`] and [`Self::Judged::populations`] and apart from both, as
        /// `OD-RULES-011` version 3 decided: no finding carries it and the claim does not read it.
        undeclared: UndeclaredValues,
    },
}
