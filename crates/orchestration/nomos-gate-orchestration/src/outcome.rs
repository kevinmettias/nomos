//! What a `nomos gate` run produced.
//!
//! One variant is real today. The others do not exist as variants at all, the same
//! "no invented shape ahead of a real body" choice [`crate::command`] documents for the verbs
//! themselves -- an outcome variant with nothing yet to carry would be exactly the empty seam
//! this workspace has learned not to build ahead of a second real case.

mod gate_outcome;
mod gate_run_outcome;

pub use gate_outcome::GateOutcome;
pub use gate_run_outcome::{Disposition, GateRunOutcome};

use nomos_rules::RuleOffer;

/// What a gate would evaluate, without evaluating it.
///
/// `rules` is every rule [`crate::composition::Registered`] holds, in [`nomos_rules::
/// RuleRegistry::Offers`]'s own order -- [`nomos_contracts::RuleId`] order, so two runs over
/// the same registration agree without depending on a hasher. It does not yet vary by
/// [`crate::GateCommand::root`] or filter by scope or rule: `ScopeSelector` and
/// `RuleSelector`, the two policy types `ARC-ROADMAP-001` names first, do not exist yet, and
/// this increment reports the one thing that is real without pretending to filter it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GatePlan
{
    /// Every rule this gate's registry holds.
    pub rules: Vec<RuleOffer>,
}
