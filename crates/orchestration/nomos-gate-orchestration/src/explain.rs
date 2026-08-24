//! `nomos gate explain` -- what one named finding looks like right now, and whether it
//! would keep a real run from passing.

use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::{Finding, RuleId};
use nomos_platform::ProcessLauncher;
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;
use std::path::PathBuf;

use crate::composition::Registered;
use crate::run_gate::Judged;
use crate::{AdoptionPolicy, BaselineDebt, BaselinePolicy, GateCommand, RuleCalibration, Suppression, SuppressionPolicy};

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
        /// into -- `Finding::Can_Fail_A_Build` and no [`RuleCalibration`] or [`Suppression`]
        /// matched it.
        would_block: bool,
        /// The calibration that kept it from blocking, when `would_block` is `false` because
        /// of one -- checked first, the same order [`crate::Run_Gate`] reduces by, since
        /// calibration is a coarser, rule-wide override.
        calibrated_by: Option<RuleCalibration>,
        /// The suppression that kept it from blocking, when `would_block` is `false`,
        /// `calibrated_by` is `None`, and a suppression matched.
        suppressed_by: Option<Suppression>,
        /// The baseline debt entry that kept it from blocking, when `would_block` is
        /// `false` and both `calibrated_by` and `suppressed_by` are `None` -- checked only
        /// once calibration and suppression are both ruled out, the same order
        /// [`crate::Run_Gate`] reduces by.
        baselined_by: Option<BaselineDebt>,
        /// The governing record `query.rule`'s implementation cites, and the version of
        /// that record it was written against -- `AGT-008`'s "rule version" clause,
        /// [`nomos_rules::RuleOffer`]'s own two fields, looked up from the same registry
        /// `nomos gate plan` already exposes through `RuleOfferResponse`. `None` only if
        /// the registry itself is contradictory (`Registered`'s own doc: "not reachable
        /// today") or `query.rule` names a rule this build does not register at all --
        /// never the ordinary "no `CONTRACT_RECORD`" case, which `Check_Naming_
        /// Convention` represents as a real, present citation to `README.md` version 0
        /// rather than an absence.
        contract: Option<(String, u32)>,
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
/// rule-narrowed `run` currently see". `command.adoption`, `command.suppressions` and
/// `command.baseline` are the three fields this does consult, because whether any applies
/// is part of the finding's own explanation, not part of narrowing which findings a run
/// counts.
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
    let explanation = Explained(&check_outcome, query, &command.adoption, &command.suppressions, &command.baseline);

    return GateExplainResult { root: command.root.clone(), check_outcome, explanation };
}

fn Explained(
    outcome: &CheckOutcome,
    query: &FindingQuery,
    adoption: &AdoptionPolicy,
    suppressions: &SuppressionPolicy,
    baseline: &BaselinePolicy,
) -> Explanation
{
    let CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        return Explanation::NotFound;
    };

    return Named(findings, query).map_or(Explanation::NotFound, |finding| return Disposed(finding, adoption, suppressions, baseline));
}

/// `query.rule`'s contract citation, from the same registry `nomos gate plan` builds --
/// `None` only for a registry `Registered` itself refuses, or a rule that registry does
/// not hold at all.
fn Contract_Of(rule: &RuleId) -> Option<(String, u32)>
{
    let registry = Registered().ok()?;
    let offer = registry.Offered(rule)?;

    return Some((offer.contract_record.clone(), offer.contract_record_version));
}

/// The one finding `query` names among `findings`, if any.
fn Named<'a>(findings: &'a [Finding], query: &FindingQuery) -> Option<&'a Finding>
{
    return findings
        .iter()
        .find(|finding| return finding.rule == query.rule && finding.locations.iter().any(|location| return location == &query.location));
}

/// `finding`, reduced to what a real run would do with it -- blocked, calibrated,
/// suppressed, or baselined, checked in that order, the same order [`crate::Run_Gate`]
/// reduces by.
fn Disposed(finding: &Finding, adoption: &AdoptionPolicy, suppressions: &SuppressionPolicy, baseline: &BaselinePolicy) -> Explanation
{
    let calibrated_by = adoption.Calibrating(finding).cloned();
    let suppressed_by = calibrated_by.is_none().then(|| suppressions.Suppressing(finding).cloned()).flatten();
    let baselined_by = (calibrated_by.is_none() && suppressed_by.is_none())
        .then(|| baseline.Tolerating(finding).cloned())
        .flatten();
    let would_block = finding.Can_Fail_A_Build() && calibrated_by.is_none() && suppressed_by.is_none() && baselined_by.is_none();
    let contract = Contract_Of(&finding.rule);

    return Explanation::Found {
        finding: Box::new(finding.clone()),
        would_block,
        calibrated_by,
        suppressed_by,
        baselined_by,
        contract,
    };
}
