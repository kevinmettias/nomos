//! `nomos gate explain` -- what one named finding looks like right now, and whether it
//! would keep a real run from passing.

use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::{Finding, RuleId};
use nomos_platform::ProcessLauncher;
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;
use std::path::PathBuf;

use crate::run_gate::Judged;
use crate::{GateCommand, Suppression, SuppressionPolicy};

/// Which finding to explain: the rule that produced it, and one of the locations it names --
/// the same human-visible `Finding::locations` a reader of `nomos gate run`'s own output
/// already sees, not a digest nothing prints today.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindingQuery
{
    /// The rule the finding to explain was produced by.
    pub rule: RuleId,
    /// One location the finding names.
    pub location: String,
}

/// What `explain` answered about a [`FindingQuery`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Explanation
{
    /// No finding from `query.rule` names `query.location` among its locations, in this
    /// judgment.
    NotFound,
    /// The finding `query` names, and what it would do to a real run's disposition.
    Found
    {
        /// The finding itself, in full. Boxed: `NotFound` carries nothing, and a `Finding`
        /// inline here would make every `Explanation` pay `Found`'s size regardless of
        /// which variant it holds.
        finding: Box<Finding>,
        /// Whether this finding, on its own, could fail a build a real `run` reduces it
        /// into -- `Finding::Can_Fail_A_Build` and no [`Suppression`] matched it.
        would_block: bool,
        /// The suppression that kept it from blocking, when `would_block` is `false`
        /// because of one rather than because the finding cannot fail a build at all.
        suppressed_by: Option<Suppression>,
    },
}

/// What a real `nomos gate explain` produced.
pub struct GateExplainResult
{
    /// The tree this judgment was over.
    pub root: PathBuf,
    /// What [`nomos_check_orchestration::Run`] (or the walk decision made before it was
    /// ever called) produced.
    pub check_outcome: CheckOutcome,
    /// The answer to `query`.
    pub explanation: Explanation,
}

/// Judges `walked` exactly as [`crate::Run_Gate`] would, then answers `query` against what
/// was judged.
///
/// Deliberately independent of `command.scope` and `command.rules`: those narrow a real
/// run's *disposition* over many findings, and this answers a question about one named
/// finding as check would produce it right now -- not "what would a scope- or
/// rule-narrowed `run` currently see". `command.suppressions` is the one field this does
/// consult, because whether a suppression applies is part of the finding's own
/// explanation, not part of narrowing which findings a run counts.
#[must_use]
pub fn Explain_Gate<P: ProcessLauncher>(
    walked: Option<Vec<SourceFile>>,
    variant: BuildVariant,
    command: &GateCommand,
    query: &FindingQuery,
    launcher: &P,
) -> GateExplainResult
{
    let check_outcome = Judged(walked, variant, &command.root, launcher);
    let explanation = Explained(&check_outcome, query, &command.suppressions);

    return GateExplainResult { root: command.root.clone(), check_outcome, explanation };
}

fn Explained(outcome: &CheckOutcome, query: &FindingQuery, suppressions: &SuppressionPolicy) -> Explanation
{
    let CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        return Explanation::NotFound;
    };

    let Some(finding) = findings
        .iter()
        .find(|finding| return finding.rule == query.rule && finding.locations.iter().any(|location| return location == &query.location))
    else
    {
        return Explanation::NotFound;
    };

    let suppressed_by = suppressions.Suppressing(finding).cloned();
    let would_block = finding.Can_Fail_A_Build() && suppressed_by.is_none();

    return Explanation::Found { finding: Box::new(finding.clone()), would_block, suppressed_by };
}
