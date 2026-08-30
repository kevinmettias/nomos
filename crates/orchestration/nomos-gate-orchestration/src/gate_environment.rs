//! Composing an already-walked tree into a real `nomos gate run`, apart from choosing a
//! platform, walking a tree or rendering the answer.

use nomos_check_orchestration::{CheckOutcome, Claim, Claim_Of};
use nomos_contracts::{Finding, RuleId, RunId};
use nomos_platform::ProcessLauncher;
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;
use std::path::Path;

use crate::{
    AdoptionPolicy, BaselinePolicy, CoveragePolicy, Disposition_Of_Findings, GateCommand, GateFindings, GateRunOutcome, GateRunResult, RuleSelector,
    ScopeSelector, SuppressionPolicy,
};

/// Judges `walked` exactly as `nomos check` would.
///
/// `walked` is the walk, already done and already decided by the composition root, the same
/// reason `nomos_check_orchestration::Run` takes `sources` rather than a root to read:
/// `None` for a root that was not a directory, `Some(sources)` otherwise -- including the
/// empty case, so the no-source-found decision stays visible to a caller rather than
/// collapsing into `Some` versus `None`. `context.variant` and `launcher` cross to
/// [`nomos_check_orchestration::Run`] unchanged; see its own documentation for why each is a
/// composition-root value this crate cannot compute for itself.
///
/// Shared by [`Run_Gate`] (over a `command.scope`-narrowed walk, and `command.rules`-selected
/// per `OD-GATE-017`) and [`crate::Explain_Gate`] (over the whole one and every rule, since
/// explain answers a question about one named finding, not a scope- or rule-narrowed
/// disposition) — factored out so the two do not duplicate this match.
///
/// `context.selected` names which rules [`nomos_check_orchestration::Run`] should compute
/// at all -- empty for every rule, the same default `RuleSelector::include` already has. A
/// caller that must see every rule's findings regardless of `command.rules` (`Explain_Gate`)
/// passes an empty slice here rather than `command.rules.include`.
pub(crate) fn Judged_Sources<Launcher: ProcessLauncher>(walked: Option<Vec<SourceFile>>, launcher: &Launcher, context: JudgeContext<'_>) -> CheckOutcome
{
    return match walked
    {
        None => CheckOutcome::Unreadable,
        Some(sources) if sources.is_empty() => CheckOutcome::NoSource,
        Some(sources) => nomos_check_orchestration::Run(
            &sources,
            nomos_check_orchestration::RunContext { variant: context.variant, root: context.root, launcher },
            context.selected,
        ),
    };
}

/// The build variant and process launcher [`Run_Gate`] and [`crate::Explain_Gate`] both need
/// but neither computes -- grouped into one value so each stays within this crate's own
/// parameter-count limit. `command` and `walked`/`query`/`run` stay separate parameters:
/// this groups only the two values every gate entry point shares.
pub struct GateEnvironment<'a, Launcher: ProcessLauncher>
{
    pub variant: BuildVariant,
    pub launcher: &'a Launcher,
}

/// What [`Judged_Sources`] judges a walked tree against, apart from the walk itself and the
/// platform used to run it -- grouped into one value so [`Judged_Sources`] stays within this
/// crate's own parameter-count limit.
pub(crate) struct JudgeContext<'a>
{
    /// Which [`BuildVariant`] to judge as.
    pub(crate) variant: BuildVariant,
    /// The tree this judgment is over.
    pub(crate) root: &'a Path,
    /// Which rules [`nomos_check_orchestration::Run`] should compute at all -- empty for
    /// every rule.
    pub(crate) selected: &'a [RuleId],
}

/// Judges `walked` exactly as `nomos check` would, and reduces the result to a
/// [`GateRunResult`].
///
/// `command.scope` narrows `walked` before [`Judged_Sources`] runs; a walk that becomes empty after
/// scoping is `CheckOutcome::NoSource`, the same state an empty walk already was, because
/// both mean "nothing was judged" to a caller.
///
/// `run` identifies this execution and is not computed here -- `OD-WORKFLOW-001`'s amendment
/// decided a `RunId` identifies one execution, not one configuration, so this function must
/// not derive it from `command` or `variant` the way everything else it composes is derived.
/// The composition root supplies one, typically [`crate::Fresh_Run_Id`] over a real clock
/// reading.
#[must_use]
pub fn Run_Gate<Launcher: ProcessLauncher>(
    walked: Option<Vec<SourceFile>>,
    environment: GateEnvironment<'_, Launcher>,
    command: &GateCommand,
    run: RunId,
) -> GateRunResult
{
    let GateEnvironment { variant, launcher } = environment;
    let scoped = walked.map(|sources| return Scoped_Sources(sources, &command.scope));
    let outcome = Judged_Sources(scoped, launcher, JudgeContext { variant, root: &command.root, selected: &command.rules.include });

    let reduced = Reduced_Findings(
        &outcome,
        &command.rules,
        DispositionPolicies { adoption: &command.adoption, suppressions: &command.suppressions, baseline: &command.baseline },
        command.coverage,
    );

    return GateRunResult {
        root: command.root.clone(),
        run,
        check_outcome: outcome,
        findings: reduced.findings,
        disposition: reduced.disposition,
    };
}

/// `sources` narrowed to what `scope` admits.
fn Scoped_Sources(sources: Vec<SourceFile>, scope: &ScopeSelector) -> Vec<SourceFile>
{
    return sources.into_iter().filter(|source| return scope.Is_In_Scope(&source.path)).collect();
}

/// [`Reduced_Findings`]'s own result -- named so its caller assigns [`GateFindings`] and the
/// disposition by field rather than by position.
struct Reduction
{
    findings: GateFindings,
    disposition: GateRunOutcome,
}

/// The blocking findings, the findings an `AdoptionPolicy` calibration kept from blocking,
/// the findings a `Suppression` kept from blocking, the findings a `BaselineDebt` kept from
/// blocking, and the disposition they imply, read off a [`CheckOutcome`] this function does
/// not own and must not consume -- `check_outcome` still has to end up in [`GateRunResult`]
/// afterward. `rules` narrows which findings count before any list is computed; a finding
/// whose rule is not selected can be neither blocking, calibrated, suppressed nor baselined,
/// but it still exists in `check_outcome` untouched. `policies.adoption` splits what remains
/// first -- a coarser, rule-wide override rather than a per-finding one -- then
/// `policies.suppressions` splits what calibration did not match, then `policies.baseline`
/// splits what neither matched: a finding matched by more than one reports as calibrated,
/// not counted twice. `coverage` is consulted last, over `rules`' own selection rather than
/// any of the four lists it splits into -- calibration, suppression and baseline each answer
/// "does this blocking finding still block," a question about one finding at a time, while
/// `coverage` answers "did this run reach a judgment about everything it selected," a
/// question about the run as a whole.
fn Reduced_Findings(
    outcome: &CheckOutcome,
    rules: &RuleSelector,
    policies: DispositionPolicies<'_>,
    coverage: CoveragePolicy,
) -> Reduction
{
    let CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        return Unjudged();
    };

    let selected: Vec<Finding> = findings.iter().filter(|finding| return rules.Is_Included(&finding.rule)).cloned().collect();
    let findings = Partitioned_Findings(&selected, policies);
    let disposition = Reduced_With_Coverage(Disposition_Of_Findings(&findings.blocking_findings), coverage, &selected);

    return Reduction { findings, disposition };
}

/// [`Reduced_Findings`]'s own result when `outcome` was never judged -- nothing was found, so
/// nothing can block, calibrate, suppress or baseline, and [`GateRunOutcome::Indeterminate`]
/// is the only disposition an unjudged run can support.
fn Unjudged() -> Reduction
{
    return Reduction {
        findings: GateFindings {
            blocking_findings: Vec::new(),
            calibrated_findings: Vec::new(),
            suppressed_findings: Vec::new(),
            baselined_findings: Vec::new(),
        },
        disposition: GateRunOutcome::Indeterminate,
    };
}

/// `selected`, split into the calibrated, suppressed, baselined and still-blocking findings
/// `policies` implies -- [`Reduced_Findings`]'s own middle section, named so that function
/// reads as one decision per line.
fn Partitioned_Findings(selected: &[Finding], policies: DispositionPolicies<'_>) -> GateFindings
{
    let blockable: Vec<Finding> = selected.iter().filter(|finding| return finding.Can_Fail_A_Build()).cloned().collect();
    let (calibrated_findings, uncalibrated): (Vec<Finding>, Vec<Finding>) =
        blockable.into_iter().partition(|finding| return policies.adoption.Calibrating(finding).is_some());
    let (suppressed_findings, remaining): (Vec<Finding>, Vec<Finding>) =
        uncalibrated.into_iter().partition(|finding| return policies.suppressions.Suppressing(finding).is_some());
    let (baselined_findings, blocking_findings): (Vec<Finding>, Vec<Finding>) =
        remaining.into_iter().partition(|finding| return policies.baseline.Tolerating(finding).is_some());

    return GateFindings { blocking_findings, calibrated_findings, suppressed_findings, baselined_findings };
}

/// The three per-finding overrides [`Reduced_Findings`] checks, grouped into one value so
/// [`Reduced_Findings`] stays within this crate's own parameter-count limit -- `adoption`
/// checked first (a coarser, rule-wide override), then `suppressions`, then `baseline`.
#[derive(Clone, Copy)]
struct DispositionPolicies<'a>
{
    adoption: &'a AdoptionPolicy,
    suppressions: &'a SuppressionPolicy,
    baseline: &'a BaselinePolicy,
}

/// `outcome`, downgraded from [`GateRunOutcome::Passed`] to [`GateRunOutcome::Indeterminate`]
/// when `coverage` requires completeness and `Claim_Of(selected)` is [`Claim::Incomplete`] --
/// `OD-GATE-016`'s own decision. Leaves every other `outcome` untouched: unset, this is the
/// identity function, and [`CoveragePolicy::RequireCompleteness`]'s own doc says why a
/// `Failed` outcome is left alone rather than downgraded the same way.
fn Reduced_With_Coverage(outcome: GateRunOutcome, coverage: CoveragePolicy, selected: &[Finding]) -> GateRunOutcome
{
    return match (coverage, outcome)
    {
        (CoveragePolicy::RequireCompleteness, GateRunOutcome::Passed) if Claim_Of(selected) == Claim::Incomplete => GateRunOutcome::Indeterminate,
        (_, outcome) => outcome,
    };
}

#[cfg(test)]
mod tests
{
    use super::{JudgeContext, Judged_Sources, Run_Gate};
    use crate::GateCommand;
    use nomos_check_orchestration::CheckOutcome;
    use nomos_contracts::{Digest128, RunId};
    use nomos_model::Subject_Of_Path;
    use nomos_platform_std::StdProcessLauncher;
    use nomos_rules::SourceFile;
    use nomos_workspace::BuildVariant;
    use std::path::PathBuf;

    fn Test_Variant() -> BuildVariant
    {
        return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
    }

    /// This repository's own real root -- [`Judged_Sources`]'s dependency step, through
    /// `nomos_check_orchestration::Run`, runs `cargo metadata` against it regardless of what
    /// sources a test hands in.
    fn Repository_Root() -> PathBuf
    {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        return manifest.parent().and_then(std::path::Path::parent).and_then(std::path::Path::parent).map(PathBuf::from).expect("this crate sits three levels below the workspace root");
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        return SourceFile::New(path, Subject_Of_Path(path), text);
    }

    #[test]
    fn Test_Judged_Sources_Should_Report_Unreadable_For_An_Unwalked_Root()
    {
        let root = Repository_Root();
        let outcome = Judged_Sources(None, &StdProcessLauncher, JudgeContext { variant: Test_Variant(), root: &root, selected: &[] });

        assert!(matches!(outcome, CheckOutcome::Unreadable));
    }

    #[test]
    fn Test_Judged_Sources_Should_Report_No_Source_For_An_Empty_Walk()
    {
        let root = Repository_Root();
        let outcome = Judged_Sources(Some(Vec::new()), &StdProcessLauncher, JudgeContext { variant: Test_Variant(), root: &root, selected: &[] });

        assert!(matches!(outcome, CheckOutcome::NoSource));
    }

    #[test]
    fn Test_Run_Gate_Should_Fail_On_A_Blocking_Finding()
    {
        let root = Repository_Root();
        let sources = vec![Source(
            "a.rs",
            "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n",
        )];
        let command = GateCommand { root: root.clone(), ..Default::default() };
        let run = RunId::From_Digest(Digest128::From_Bytes([0; Digest128::BYTE_LENGTH]));

        let result = Run_Gate(Some(sources), super::GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher }, &command, run);

        assert!(!result.findings.blocking_findings.is_empty());
    }
}
