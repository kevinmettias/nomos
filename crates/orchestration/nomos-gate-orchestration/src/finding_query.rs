//! `nomos gate explain` -- what one named finding looks like right now, and whether it
//! would keep a real run from passing.

mod explanation;
mod gate_explain_result;

pub use explanation::Explanation;
pub use gate_explain_result::GateExplainResult;

use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::{Finding, RuleId};
use nomos_platform::{Environment, FileSystem, ProcessLauncher, Timestamp};
use nomos_rules::SourceFile;

use crate::gate_environment::{GateEnvironment, JudgeContext, Judged_Sources};
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
pub fn Explain_Gate<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(
    walked: Option<Vec<SourceFile>>,
    environment: GateEnvironment<'_, Launcher, Fs, Env>,
    command: &GateCommand,
    query: &FindingQuery,
) -> GateExplainResult
{
    let GateEnvironment { variant, launcher, filesystem, environment, now } = environment;
    let check_outcome = Judged_Sources(walked, JudgeContext { launcher, filesystem, environment, variant, root: &command.root, selected: &[] });
    let explanation = Explained_Query(
        &check_outcome,
        query,
        DispositionPolicies { adoption: &command.adoption, suppressions: &command.suppressions, baseline: &command.baseline, now },
    );

    return GateExplainResult { root: command.root.clone(), check_outcome, explanation };
}

/// The three per-finding overrides [`Explained_Query`] and [`Disposed_Finding`] check,
/// grouped into one value so [`Explained_Query`] stays within this crate's own
/// parameter-count limit -- `adoption` checked first (a coarser, rule-wide override), then
/// `suppressions`, then `baseline`, the same order [`crate::Run_Gate`] reduces by.
#[derive(Clone, Copy)]
struct DispositionPolicies<'a>
{
    adoption: &'a AdoptionPolicy,
    suppressions: &'a SuppressionPolicy,
    baseline: &'a BaselinePolicy,
    now: Timestamp,
}

fn Explained_Query(outcome: &CheckOutcome, query: &FindingQuery, policies: DispositionPolicies<'_>) -> Explanation
{
    let CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        return Explanation::NotFound;
    };

    return Named_Finding(findings, query)
        .map_or(Explanation::NotFound, |finding| return Disposed_Finding(finding, policies));
}

/// The one finding `query` names among `findings`, if any.
fn Named_Finding<'a>(findings: &'a [Finding], query: &FindingQuery) -> Option<&'a Finding>
{
    return findings
        .iter()
        .find(|finding| return finding.rule == query.rule && finding.locations.iter().any(|location| return location == &query.location));
}

/// `finding`, reduced to what a real run would do with it -- blocked, calibrated,
/// suppressed, or baselined, checked in that order, the same order [`crate::Run_Gate`]
/// reduces by.
fn Disposed_Finding(finding: &Finding, policies: DispositionPolicies<'_>) -> Explanation
{
    let DispositionPolicies { adoption, suppressions, baseline, now } = policies;
    let calibrated_by = adoption.Calibrating(finding).cloned();
    let suppressed_by = calibrated_by.is_none().then(|| suppressions.Suppressing(finding, now).cloned()).flatten();
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
    use crate::Registered;

    let registry = Registered().ok()?;
    let offer = registry.Offered(rule)?;

    return Some((offer.contract_record.clone(), offer.contract_record_version));
}

#[cfg(test)]
mod tests
{
    use super::{Explain_Gate, FindingQuery};
    use crate::{Explanation, GateCommand, GateEnvironment};
    use nomos_model::Subject_Of_Path;
    use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProcessLauncher};
    use nomos_rules::{SourceFile, COMPLETENESS_MIRROR};
    use nomos_workspace::BuildVariant;
    use std::path::PathBuf;

    fn Source(path: &str, text: &str) -> SourceFile
    {
        return SourceFile::New(path, Subject_Of_Path(path), text);
    }

    /// A real query naming the one blocking finding a phantom mirror produces answers `Found`,
    /// with `would_block` true -- proof this crate's own `Explain_Gate` (not a rename of the
    /// higher-level `Run_Gate` fixture) actually judges the source and answers the query.
    #[test]
    fn Test_Explain_Gate_Should_Find_A_Real_Blocking_Finding()
    {
        let sources = vec![Source(
            "a.rs",
            "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n",
        )];
        let query = FindingQuery { rule: nomos_contracts::RuleId::New(COMPLETENESS_MIRROR), location: "a.rs".to_owned() };
        let command = GateCommand { root: Repository_Root(), ..Default::default() };

        let result = Explain_Gate(Some(sources), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, now: nomos_platform::Timestamp::From_Unix_Seconds(0) }, &command, &query);

        let Explanation::Found { would_block, .. } = result.explanation
        else
        {
            // this fixture's own source matches the rule it names; a refusal here is a bug
            // in the fixture, not a caller-facing failure to route through Result.
            panic!("this fixture must produce the finding the query names");
        };
        assert!(would_block);
    }

    fn Repository_Root() -> PathBuf
    {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        return manifest.parent().and_then(std::path::Path::parent).and_then(std::path::Path::parent).map(PathBuf::from).expect("this crate sits three levels below the workspace root");
    }

    fn Test_Variant() -> BuildVariant
    {
        return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
    }
}
