//! What a `nomos gate` run produced.
//!
//! One variant is real today. The others do not exist as variants at all, the same
//! "no invented shape ahead of a real body" choice [`crate::gate_command`] documents for the verbs
//! themselves -- an outcome variant with nothing yet to carry would be exactly the empty seam
//! this workspace has learned not to build ahead of a second real case.

mod gate_findings;
mod gate_outcome;
mod gate_run_outcome;
mod gate_run_result;

pub use gate_findings::GateFindings;
pub use gate_outcome::GateOutcome;
pub use gate_run_outcome::{Disposition_Of_Findings, GateRunOutcome};
pub use gate_run_result::{GateRunResult, NoVerdict};

use nomos_rules::RuleOffer;

/// What a gate would evaluate, without evaluating it.
///
/// `rules` is [`crate::composition::Registered`]'s own offers, filtered by
/// [`crate::GateCommand::rules`] the same way [`crate::Run_Gate`] narrows a real run's
/// findings -- an empty [`crate::RuleSelector::include`] plans every registered rule, the
/// same "select everything" default every existing caller already has, so two invocations
/// differing only in which rules are selected produce different plans (`P41-GATE-PLAN-IS-A-
/// PLAN-3`). Registered offers are read in [`nomos_rules::RuleRegistry::Offers`]'s own order
/// -- [`nomos_contracts::RuleId`] order, so two runs over the same registration and the same
/// selection agree without depending on a hasher. It still does not vary by
/// [`crate::GateCommand::root`] or `scope`: `Plan` reports the registry, not a walk, so
/// `ScopeSelector` -- real since `P13-GATE-014-SCOPE-RULE-SELECTORS`, and consulted by
/// [`crate::Run_Gate`] -- has no file here to narrow against.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GatePlan
{
    /// Every rule this gate's registry holds.
    pub rules: Vec<RuleOffer>,
}
