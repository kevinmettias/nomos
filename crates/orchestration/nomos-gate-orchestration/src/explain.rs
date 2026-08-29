//! `nomos gate explain` -- what one named finding looks like right now, and whether it
//! would keep a real run from passing.

mod explanation;
mod gate_explain_result;

pub use explanation::Explanation;
pub use gate_explain_result::GateExplainResult;

use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::{Finding, RuleId};
use nomos_platform::ProcessLauncher;
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;

use crate::composition::Registered;
use crate::run_gate::{JudgeContext, Judged};
use crate::{AdoptionPolicy, BaselinePolicy, GateCommand, SuppressionPolicy};

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

/// Judges `walked` exactly as [`crate::Run_Gate`] would, then answers `query` against what
/// was judged.
///
/// Deliberately independent of `command.scope` and `command.rules`: those narrow a real
/// run's *disposition* over many findings, and this answers a question about one named
/// finding as check would produce it right now -- not "what would a scope- or
/// rule-narrowed `run` currently see". `command.adoption`, `command.suppressions` and
/// `command.baseline` are the three fields this does consult, because whether any applies
/// is part of the finding's own explanation, not part of narrowing which findings a run
/// counts. `Judged` is passed an empty rule selection here, not `command.rules.include`, for
/// the same reason: since `OD-GATE-017`, a non-empty selection also narrows what
/// [`nomos_check_orchestration::Run`] computes at all, and a query about a rule
/// `command.rules` excludes must still be answerable.
#[must_use]
pub fn Explain_Gate<P: ProcessLauncher>(
    walked: Option<Vec<SourceFile>>,
    variant: BuildVariant,
    command: &GateCommand,
    query: &FindingQuery,
    launcher: &P,
) -> GateExplainResult
{
    let check_outcome = Judged(walked, launcher, JudgeContext { variant, root: &command.root, selected: &[] });
    let explanation = Explained(
        &check_outcome,
        query,
        DispositionPolicies { adoption: &command.adoption, suppressions: &command.suppressions, baseline: &command.baseline },
    );

    return GateExplainResult { root: command.root.clone(), check_outcome, explanation };
}

/// The three per-finding overrides [`Explained`] and [`Disposed`] check, grouped into one
/// value so [`Explained`] stays within this crate's own parameter-count limit -- `adoption`
/// checked first (a coarser, rule-wide override), then `suppressions`, then `baseline`, the
/// same order [`crate::Run_Gate`] reduces by.
#[derive(Clone, Copy)]
struct DispositionPolicies<'a>
{
    adoption: &'a AdoptionPolicy,
    suppressions: &'a SuppressionPolicy,
    baseline: &'a BaselinePolicy,
}

fn Explained(outcome: &CheckOutcome, query: &FindingQuery, policies: DispositionPolicies<'_>) -> Explanation
{
    let CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        return Explanation::NotFound;
    };

    return Named(findings, query)
        .map_or(Explanation::NotFound, |finding| return Disposed(finding, policies.adoption, policies.suppressions, policies.baseline));
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

/// `query.rule`'s contract citation, from the same registry `nomos gate plan` builds --
/// `None` only for a registry `Registered` itself refuses, or a rule that registry does
/// not hold at all.
fn Contract_Of(rule: &RuleId) -> Option<(String, u32)>
{
    let registry = Registered().ok()?;
    let offer = registry.Offered(rule)?;

    return Some((offer.contract_record.clone(), offer.contract_record_version));
}
