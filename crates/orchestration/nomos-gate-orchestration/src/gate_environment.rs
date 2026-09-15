//! Composing an already-walked tree into a real `nomos gate run`, apart from choosing a
//! platform, walking a tree or rendering the answer.

use nomos_check_orchestration::{CheckOutcome, Claim, Claim_Of};
use nomos_contracts::{Finding, RuleId, RunId, SubjectId};
use std::collections::BTreeMap;
use nomos_platform::{Environment, FileSystem, ProcessLauncher, Timestamp};
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;
use std::path::Path;

use crate::policy::{GatePolicyFile, Resolve_Gate_Policy};
use crate::{
    AdoptionPolicy, BaselinePolicy, CoveragePolicy, Disposition_Of_Findings, Evaluated_Phases, GateCommand, GateFindings, GateRunOutcome, GateRunResult,
    NoVerdict, Phased_Disposition, RuleSelector, ScopeSelector, SuppressionPolicy, SuppressionReason,
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
pub(crate) fn Judged_Sources<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(
    walked: Option<Vec<SourceFile>>,
    context: JudgeContext<'_, Launcher, Fs, Env>,
) -> CheckOutcome
{
    return match walked
    {
        None => CheckOutcome::Unreadable,
        Some(sources) if sources.is_empty() => CheckOutcome::NoSource,
        Some(sources) => nomos_check_orchestration::Run(
            &sources,
            nomos_check_orchestration::RunContext {
                variant: context.variant,
                root: context.root,
                launcher: context.launcher,
                filesystem: context.filesystem,
                environment: context.environment,
                workspace: &mut None,
                store: &mut nomos_analysis::MemoryFactStore::New(),
            },
            context.selected,
        ),
    };
}

/// The build variant, process launcher and filesystem [`Run_Gate`] and [`crate::Explain_Gate`]
/// both need but neither computes -- grouped into one value so each stays within this crate's
/// own parameter-count limit. `command` and `walked`/`query`/`run` stay separate parameters:
/// this groups only the three values every gate entry point shares.
pub struct GateEnvironment<'a, Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>
{
    pub variant: BuildVariant,
    pub launcher: &'a Launcher,
    pub filesystem: &'a Fs,
    /// Where a provider reads `CARGO` and the working directory from, rather than from this
    /// process's own ambient state. `OD-HOST-001`: the composition root chooses it.
    pub environment: &'a Env,
    /// The moment this run is judged against.
    ///
    /// Supplied by whoever composed the run rather than read from a clock inside policy
    /// logic, so a replay of a past run answers as that run did instead of as today would.
    /// It rides here for the reason `variant` does: the composing function is already at this
    /// workspace's own parameter-count limit, and an execution fact belongs with the other
    /// execution facts rather than in the caller-authored `GateCommand`, which is policy.
    pub now: Timestamp,
}

/// What [`Judged_Sources`] judges a walked tree against, apart from the walk itself and the
/// platform used to run it -- grouped into one value so [`Judged_Sources`] stays within this
/// crate's own parameter-count limit.
pub(crate) struct JudgeContext<'a, Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>
{
    /// The process launcher a provider's subprocess runs through.
    pub(crate) launcher: &'a Launcher,
    /// The filesystem a provider reads through.
    pub(crate) filesystem: &'a Fs,
    /// Where a provider reads `CARGO` and the working directory from.
    pub(crate) environment: &'a Env,
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
pub fn Run_Gate<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(
    walked: Option<Vec<SourceFile>>,
    environment: GateEnvironment<'_, Launcher, Fs, Env>,
    command: &GateCommand,
    run: RunId,
) -> GateRunResult
{
    let GateEnvironment { variant, launcher, filesystem, environment, now } = environment;
    let declared = Resolve_Gate_Policy(&command.root, filesystem);
    let effective = match &declared
    {
        Ok(Some(from_file)) => from_file.Resolved_Over(command),
        // No file, or one that could not be read: the command's own policies stand alone,
        // which for every caller that states none is today's behavior exactly.
        Ok(None) | Err(_) => GatePolicyFile::default().Resolved_Over(command),
    };

    // `OD-GATE-025`: the scope never reaches the judging. Every walked file is judged, and
    // the scope narrows the findings afterwards -- a rule answering a cross-file question
    // must see the whole world or it answers a different question and labels it the same.
    //
    // A scope admitting no walked source at all is still `NoSource`, which is what it was
    // before the scope moved. The judging happened and is simply discarded: what a caller is
    // told is that nothing it asked about was there, and a mistyped `--include` must not read
    // as a repository with nothing to say.
    let admits_a_source = walked.as_ref().is_none_or(|sources| {
        return sources.iter().any(|source| return command.scope.Is_In_Scope(&source.path));
    });
    let judged = Judged_Sources(walked, JudgeContext { launcher, filesystem, environment, variant, root: &command.root, selected: &command.rules.include });
    let outcome = if admits_a_source { Scoped_Findings(judged, &command.scope) } else { CheckOutcome::NoSource };

    let reduced = Reduced_Findings(
        &outcome,
        &command.rules,
        DispositionPolicies { adoption: &effective.adoption, suppressions: &effective.suppressions, baseline: &effective.baseline, now },
        effective.coverage,
    );

    let phase_outcomes = Evaluated_Phases(&command.phases, &reduced.findings.blocking_findings, &command.approvals);
    let disposition = Phased_Disposition(reduced.disposition, &command.phases, &phase_outcomes, &reduced.findings.blocking_findings);

    // A policy file that exists and could not be turned into a policy refuses the run rather
    // than letting it report a disposition reached under policy nobody authored. The judgment
    // above still happens and `check_outcome` still carries it in full, so a caller sees
    // exactly what the check found; what it does not get is a verdict, because the rules for
    // turning findings into one were unreadable. Reported after judging rather than instead of
    // it so the answer stays as informative as it honestly can be.
    //
    // The cause travels with the refusal. It used to be read as `declared.is_err()` and
    // dropped, which left every consumer able to say that there was no verdict and unable to
    // say why -- including for the reader's own message naming the key it refused.
    let unusable_policy = declared.as_ref().err().map(|error| return error.As_No_Verdict());

    return GateRunResult {
        root: command.root.clone(),
        run,
        check_outcome: outcome,
        findings: reduced.findings,
        unmatched_policy: reduced.unmatched_policy,
        disposition: if unusable_policy.is_some() { GateRunOutcome::Indeterminate } else { disposition },
        // The policy failure wins when both could apply. It cannot: an unreadable file falls
        // back to a default policy whose coverage is Unset, so Reduced_With_Coverage never
        // downgrades under one. Written as a preference anyway rather than as an assumption,
        // because the fallback is in a different function than this line.
        no_verdict: unusable_policy.or(reduced.no_verdict),
    };
}

/// `outcome`'s findings narrowed to what `scope` admits, the judging behind them untouched.
///
/// `OD-GATE-025` decided this is where a scope belongs. Narrowing the *source* set instead
/// handed a rule answering a cross-file question a truncated world, which is how
/// `--include <one file>` came to report `no-orphan-modules` against a file its own `lib.rs`
/// declares: the file was collected and its declaring root was not.
///
/// A finding is admitted when any of its locations is, and a finding carrying no location at
/// all is admitted unchanged -- a path filter has nothing to say about a finding that names
/// no path, which is every `dependency-policy` advisory about the workspace as a whole.
fn Scoped_Findings(outcome: CheckOutcome, scope: &ScopeSelector) -> CheckOutcome
{
    let CheckOutcome::Judged { findings, examined, claim } = outcome
    else
    {
        return outcome;
    };

    let admitted = findings
        .into_iter()
        .filter(|finding| return Is_Admitted(finding, scope))
        .collect();

    return CheckOutcome::Judged { findings: admitted, examined, claim };
}

/// Whether `scope` admits `finding`, by the places it names.
fn Is_Admitted(finding: &Finding, scope: &ScopeSelector) -> bool
{
    if finding.locations.is_empty()
    {
        return true;
    }

    return finding.locations.iter().any(|location| return scope.Is_In_Scope(location));
}

/// [`Reduced_Findings`]'s own result -- named so its caller assigns [`GateFindings`] and the
/// disposition by field rather than by position.
struct Reduction
{
    findings: GateFindings,
    disposition: GateRunOutcome,
    unmatched_policy: Vec<String>,
    /// Why this reduction reached no verdict, when it judged findings and reached none.
    /// Only [`Reduced_With_Coverage`] can produce one here; the policy file is read before
    /// any of this runs and [`Run_Gate`] carries that cause itself.
    no_verdict: Option<NoVerdict>,
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
/// Every declared entry that no finding in `selected` matched, described for a reader.
///
/// `OD-GATE-024`'s one clause that survived its own retraction: an entry matching nothing is
/// reported rather than silently ignored. Against the findings a run actually *selected*,
/// not every finding it judged, because an entry for a rule the caller deselected did not
/// fail to match -- it was never asked.
fn Unmatched_Entries(selected: &[Finding], policies: DispositionPolicies<'_>) -> Vec<String>
{
    let mut unmatched = Vec::new();

    for suppression in &policies.suppressions.suppressions
    {
        if !selected.iter().any(|finding| return suppression.Is_Applicable_To(finding))
        {
            unmatched.push(format!("suppression for `{}` matched nothing", suppression.rule));
        }
    }
    for debt in &policies.baseline.debt
    {
        if !selected.iter().any(|finding| return debt.Is_Applicable_To(finding))
        {
            unmatched.push(format!("baseline entry for `{}` matched nothing", debt.rule));
        }
    }
    for calibration in &policies.adoption.calibrated
    {
        if !selected.iter().any(|finding| return calibration.Is_Applicable_To(finding))
        {
            unmatched.push(format!("calibration for `{}` matched nothing", calibration.rule));
        }
    }

    return unmatched;
}

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

    let unmatched_policy = Unmatched_Entries(&selected, policies);

    // Disposition_Of_Findings answers Passed or Failed and never Indeterminate, so an
    // Indeterminate here is Reduced_With_Coverage's own downgrade and nothing else. Read off
    // the result rather than recomputing the claim, so the two can never disagree about why.
    let no_verdict = (disposition == GateRunOutcome::Indeterminate).then_some(NoVerdict::IncompleteCoverage);

    return Reduction { findings, disposition, unmatched_policy, no_verdict };
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
            suppression_reasons: std::collections::BTreeMap::new(),
        },
        disposition: GateRunOutcome::Indeterminate,
        // Nothing was judged, so no entry failed to match -- none was asked.
        unmatched_policy: Vec::new(),
        // Nothing was judged, so check_outcome is already the reason and this does not
        // restate it. See GateRunResult::no_verdict's own doc.
        no_verdict: None,
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
        uncalibrated.into_iter().partition(|finding| return policies.suppressions.Suppressing(finding, policies.now).is_some());
    let (baselined_findings, blocking_findings): (Vec<Finding>, Vec<Finding>) =
        remaining.into_iter().partition(|finding| return policies.baseline.Tolerating(finding).is_some());
    let suppression_reasons = Recorded_Reasons(selected, policies);

    return GateFindings { blocking_findings, calibrated_findings, suppressed_findings, baselined_findings, suppression_reasons };
}

/// Why a disposition applied to each finding one names, recorded at the moment it was decided.
///
/// Both halves, deliberately. An active disposition explains why a finding is in
/// `suppressed_findings`. A lapsed one explains why a finding is *not*: under `P103` an expired
/// waiver does not suppress, so the finding falls through to baseline or blocking, and without
/// this a reader comparing two runs would see it arrive in `blocking_findings` with nothing
/// saying a tolerance came due rather than a new violation appearing. That distinction is the
/// whole reason expiry was made visible in the run model, and it would be lost again in
/// comparison if only suppressing dispositions were recorded.
fn Recorded_Reasons(selected: &[Finding], policies: DispositionPolicies<'_>) -> BTreeMap<(RuleId, SubjectId), SuppressionReason>
{
    let mut reasons = BTreeMap::new();

    for finding in selected
    {
        let key = (finding.rule.clone(), finding.subject);

        if let Some(suppression) = policies.suppressions.Suppressing(finding, policies.now)
        {
            reasons.insert(key, SuppressionReason { disposition: suppression.disposition, status: suppression.Status_At(policies.now) });
            continue;
        }

        if let Some(lapsed) = policies.suppressions.Lapsed(finding, policies.now).first()
        {
            reasons.insert(key, SuppressionReason { disposition: lapsed.disposition, status: lapsed.Status_At(policies.now) });
        }
    }

    return reasons;
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
    now: Timestamp,
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
    use crate::{CoveragePolicy, GateCommand, GatePhase, GateRunOutcome, NoVerdict, PhaseApproval, PhaseThreshold};
    use nomos_check_orchestration::{CheckOutcome, Claim};
    use nomos_contracts::{Digest128, RuleId, RunId};
    use nomos_model::Subject_Of_Path;
    use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProcessLauncher};
    use nomos_rules::SourceFile;
    use nomos_workspace::BuildVariant;
    use std::path::PathBuf;

    fn Source(path: &str, text: &str) -> SourceFile
    {
        return SourceFile::New(path, Subject_Of_Path(path), text);
    }

    /// One source guaranteed to produce a real `completeness-mirror` blocking finding --
    /// the same fixture [`Test_Run_Gate_Should_Fail_On_A_Blocking_Finding`] already uses,
    /// named so the phase tests below do not repeat its literal text.
    fn Blocking_Sources() -> Vec<SourceFile>
    {
        return vec![Source(
            "a.rs",
            "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n",
        )];
    }


    /// A scratch root carrying `policy` as its own `nomos-gate.json`.
    ///
    /// A third private copy of a helper `gate_policy_file.rs` and `tests.rs` each keep one of
    /// already. Reaching across for either would make it public for a caller that wants three
    /// lines, which costs this crate's surface more than the repetition costs a reader.
    fn Root_With_Policy(name: &str, policy: &str) -> PathBuf
    {
        let root = std::env::temp_dir().join(format!("nomos-gate-environment-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&root).expect("creatable");
        std::fs::write(root.join("nomos-gate.json"), policy).expect("writable");

        return root;
    }

    /// A policy file whose key is mis-spelled leaves the run without a verdict, and the run
    /// says so *and* says which key was refused.
    ///
    /// The likeliest operator error there is, in the one file a repository adopting this tool
    /// writes by hand, and `DeclaredPolicy` refuses it under `deny_unknown_fields` on purpose.
    /// The reader's message naming the offending key was computed and discarded until
    /// `no_verdict` existed, so this asserts the key itself reaches a caller rather than only
    /// that something went wrong.
    #[test]
    fn Test_Run_Gate_Should_Name_The_Key_A_Malformed_Policy_Was_Refused_For()
    {
        let root = Root_With_Policy("malformed", r#"{ "basline": [] }"#);
        let command = GateCommand { root: root.clone(), ..Default::default() };
        let run = RunId::From_Digest(Digest128::From_Bytes([0; Digest128::BYTE_LENGTH]));

        let result = Run_Gate(Some(Blocking_Sources()), super::GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, now: nomos_platform::Timestamp::From_Unix_Seconds(0) }, &command, run);

        assert_eq!(result.disposition, GateRunOutcome::Indeterminate);
        let Some(NoVerdict::MalformedPolicy(detail)) = result.no_verdict
        else
        {
            panic!("expected a malformed policy, got {:?}", result.no_verdict);
        };
        assert!(detail.contains("basline"), "{detail}");
        // The judging still happened and is still reported in full. Refusing the verdict is
        // not refusing the answer, which is what Run_Gate's own comment promises.
        assert!(matches!(result.check_outcome, CheckOutcome::Judged { .. }));
    }

    /// A run that reaches a real verdict records no reason for one, so a caller reading
    /// `no_verdict` on an ordinary run is told nothing rather than something empty.
    #[test]
    fn Test_Run_Gate_Should_Record_No_Reason_When_It_Reached_A_Verdict()
    {
        let root = Repository_Root();
        let command = GateCommand { root: root.clone(), ..Default::default() };
        let run = RunId::From_Digest(Digest128::From_Bytes([0; Digest128::BYTE_LENGTH]));

        let result = Run_Gate(Some(Blocking_Sources()), super::GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, now: nomos_platform::Timestamp::From_Unix_Seconds(0) }, &command, run);

        assert_eq!(result.disposition, GateRunOutcome::Failed);
        assert_eq!(result.no_verdict, None);
    }

    /// The coverage floor downgrading an otherwise-passing run names itself, and is therefore
    /// tellable apart from a broken policy file -- the distinction that matters most in this
    /// enum, because this one is not a fault and the other two are.
    ///
    /// Driven through `Reduced_Findings` rather than `Run_Gate`: this is the exact mechanism
    /// `OD-GATE-016` decided, and reaching it through a real walk would make the test depend
    /// on which capabilities happen to be materializable on the machine running it.
    #[test]
    fn Test_Reduced_Findings_Should_Name_The_Coverage_Floor_That_Downgraded_A_Pass()
    {
        let outcome = CheckOutcome::Judged {
            findings: vec![Unjudgeable_Finding()],
            examined: nomos_check_orchestration::Examined { files: 1, facts: 1 },
            claim: Claim::Incomplete,
        };
        let (adoption, suppressions, baseline) = (Default::default(), Default::default(), Default::default());
        let policies = super::DispositionPolicies { adoption: &adoption, suppressions: &suppressions, baseline: &baseline, now: nomos_platform::Timestamp::From_Unix_Seconds(0) };

        let reduced = super::Reduced_Findings(&outcome, &Default::default(), policies, CoveragePolicy::RequireCompleteness);

        assert_eq!(reduced.disposition, GateRunOutcome::Indeterminate);
        assert_eq!(reduced.no_verdict, Some(NoVerdict::IncompleteCoverage));
        assert!(reduced.findings.blocking_findings.is_empty(), "nothing here can fail a build; the floor is the whole reason");
    }

    /// The same findings without a declared floor reach a verdict and name no reason, which is
    /// what makes the assertion above about the floor rather than about the findings.
    #[test]
    fn Test_Reduced_Findings_Should_Pass_The_Same_Findings_With_No_Declared_Floor()
    {
        let outcome = CheckOutcome::Judged {
            findings: vec![Unjudgeable_Finding()],
            examined: nomos_check_orchestration::Examined { files: 1, facts: 1 },
            claim: Claim::Incomplete,
        };
        let (adoption, suppressions, baseline) = (Default::default(), Default::default(), Default::default());
        let policies = super::DispositionPolicies { adoption: &adoption, suppressions: &suppressions, baseline: &baseline, now: nomos_platform::Timestamp::From_Unix_Seconds(0) };

        let reduced = super::Reduced_Findings(&outcome, &Default::default(), policies, CoveragePolicy::Unset);

        assert_eq!(reduced.disposition, GateRunOutcome::Passed);
        assert_eq!(reduced.no_verdict, None);
    }

    /// One finding a rule could not judge, which is what makes a claim `Incomplete`.
    fn Unjudgeable_Finding() -> nomos_contracts::Finding
    {
        return nomos_contracts::Finding {
            rule: RuleId::New("dependency-policy"),
            subject: Subject_Of_Path("a.rs"),
            subject_name: "a.rs".to_owned(),
            applicability: nomos_contracts::Applicability::MissingCapability,
            evidence: nomos_contracts::EvidenceClass::Derived,
            gate: nomos_contracts::GateCategory::Advisory,
            summary: "no provider offered the capability this rule requires".to_owned(),
            locations: vec!["a.rs".to_owned()],
        };
    }

    #[test]
    fn Test_Judged_Sources_Should_Report_Unreadable_For_An_Unwalked_Root()
    {
        let root = Repository_Root();
        let outcome = Judged_Sources(None, JudgeContext { launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, variant: Test_Variant(), root: &root, selected: &[] });

        assert!(matches!(outcome, CheckOutcome::Unreadable));
    }

    #[test]
    fn Test_Judged_Sources_Should_Report_No_Source_For_An_Empty_Walk()
    {
        let root = Repository_Root();
        let outcome = Judged_Sources(Some(Vec::new()), JudgeContext { launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, variant: Test_Variant(), root: &root, selected: &[] });

        assert!(matches!(outcome, CheckOutcome::NoSource));
    }

    #[test]
    fn Test_Run_Gate_Should_Fail_On_A_Blocking_Finding()
    {
        let root = Repository_Root();
        let command = GateCommand { root: root.clone(), ..Default::default() };
        let run = RunId::From_Digest(Digest128::From_Bytes([0; Digest128::BYTE_LENGTH]));

        let result =
            Run_Gate(Some(Blocking_Sources()), super::GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, now: nomos_platform::Timestamp::From_Unix_Seconds(0) }, &command, run);

        assert!(!result.findings.blocking_findings.is_empty());
    }

    #[test]
    fn Test_Run_Gate_Should_Pass_When_A_Phase_Approval_Covers_Every_Blocking_Finding()
    {
        let root = Repository_Root();
        let unphased = GateCommand { root: root.clone(), ..Default::default() };
        let baseline = Run_Gate(
            Some(Blocking_Sources()),
            super::GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, now: nomos_platform::Timestamp::From_Unix_Seconds(0) },
            &unphased,
            RunId::From_Digest(Digest128::From_Bytes([9; Digest128::BYTE_LENGTH])),
        );
        // Every rule the fixture's own blocking findings actually name, read off a real run
        // rather than hard-coded -- this fixture is shared with `Test_Run_Gate_Should_Fail_
        // On_A_Blocking_Finding` and may trip more than one rule (`completeness-mirror` and
        // `single-letter-names` both plausibly apply to `pub const T`), and a phase that
        // named only one of them would leave the other unphased, which is a different test.
        let rules: Vec<RuleId> = baseline.findings.blocking_findings.iter().map(|finding| return finding.rule.clone()).collect();
        assert!(!rules.is_empty(), "the fixture must produce at least one blocking finding for this test to mean anything");

        let phase = GatePhase { name: "completeness".to_owned(), rules, threshold: PhaseThreshold::AnyBlockingFinding };
        let approval = PhaseApproval { phase: "completeness".to_owned(), rationale: "reviewed and accepted".to_owned() };
        let command = GateCommand { root: root.clone(), phases: vec![phase], approvals: vec![approval], ..Default::default() };
        let run = RunId::From_Digest(Digest128::From_Bytes([1; Digest128::BYTE_LENGTH]));

        let result =
            Run_Gate(Some(Blocking_Sources()), super::GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, now: nomos_platform::Timestamp::From_Unix_Seconds(0) }, &command, run);

        assert!(!result.findings.blocking_findings.is_empty(), "the finding must still be real and reported, not hidden");
        assert!(matches!(result.disposition, GateRunOutcome::Passed), "an approved phase covering every blocking finding must pass the run");
    }

    #[test]
    fn Test_Run_Gate_Should_Stay_Failed_When_A_Blocking_Finding_Belongs_To_No_Declared_Phase()
    {
        let root = Repository_Root();
        let phase = GatePhase { name: "unrelated".to_owned(), rules: vec![RuleId::New("naming-convention")], threshold: PhaseThreshold::AnyBlockingFinding };
        let command = GateCommand { root: root.clone(), phases: vec![phase], ..Default::default() };
        let run = RunId::From_Digest(Digest128::From_Bytes([2; Digest128::BYTE_LENGTH]));

        let result =
            Run_Gate(Some(Blocking_Sources()), super::GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, now: nomos_platform::Timestamp::From_Unix_Seconds(0) }, &command, run);

        assert!(matches!(result.disposition, GateRunOutcome::Failed), "a phase policy must not let a finding outside its own scope silently stop blocking");
    }

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
}

#[cfg(test)]
mod reason_recording_tests
{
    use super::{DispositionPolicies, Partitioned_Findings};
    use crate::{AdoptionPolicy, BaselinePolicy, Suppression, SuppressionDisposition, SuppressionPolicy, SuppressionStatus};
    use nomos_contracts::{Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};
    use nomos_platform::Timestamp;

    const EXPIRY: i64 = 1_000;

    fn Finding_Here() -> Finding
    {
        return Finding {
            rule: RuleId::New("naming-convention"),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([3; Digest128::BYTE_LENGTH])),
            subject_name: "src/lib.rs".to_string(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: "a name".to_string(),
            locations: Vec::new(),
        };
    }

    fn Waiver() -> Suppression
    {
        let finding = Finding_Here();

        return Suppression {
            rule: finding.rule,
            subject: finding.subject,
            disposition: SuppressionDisposition::TemporaryWaiver,
            rationale: "bounded".to_string(),
            owner: "someone".to_string(),
            expiry: Some(Timestamp::From_Unix_Seconds(EXPIRY)),
        };
    }

    fn Partitioned_At(seconds: i64) -> crate::GateFindings
    {
        let adoption = AdoptionPolicy::default();
        let baseline = BaselinePolicy::default();
        let suppressions = SuppressionPolicy { suppressions: vec![Waiver()] };

        return Partitioned_Findings(
            &[Finding_Here()],
            DispositionPolicies {
                adoption: &adoption,
                suppressions: &suppressions,
                baseline: &baseline,
                now: Timestamp::From_Unix_Seconds(seconds),
            },
        );
    }

    /// A live waiver suppresses, and its reason is recorded as active.
    ///
    /// The forward direction: a suppressed finding always has a reason, so a reader is never
    /// told a finding did not block without being told why.
    #[test]
    fn Test_A_Suppressed_Finding_Should_Carry_An_Active_Reason()
    {
        let findings = Partitioned_At(EXPIRY - 1);
        let finding = findings.suppressed_findings.first().expect("a live waiver suppresses");
        let reason = findings
            .suppression_reasons
            .get(&(finding.rule.clone(), finding.subject))
            .expect("a suppressed finding must say why");

        assert_eq!(reason.disposition, SuppressionDisposition::TemporaryWaiver);
        assert_eq!(reason.status, SuppressionStatus::Active);
    }

    /// A lapsed waiver does not suppress, and its reason is still recorded as expired.
    ///
    /// The direction the narrower invariant would have forbidden. Recording only suppressing
    /// dispositions would leave this finding in `blocking_findings` with nothing saying a
    /// tolerance came due rather than a violation appearing -- which is exactly the
    /// distinction expiry was made visible for, lost again at the moment a reader looks for
    /// it. So the recorded set is not the suppressed set, deliberately, and this is the case
    /// that shows why.
    #[test]
    fn Test_A_Lapsed_Waiver_Should_Be_Recorded_Though_It_Did_Not_Suppress()
    {
        let findings = Partitioned_At(EXPIRY);
        let finding = findings.blocking_findings.first().expect("a lapsed waiver does not suppress");

        assert!(findings.suppressed_findings.is_empty(), "an expired waiver must not suppress");

        let reason = findings
            .suppression_reasons
            .get(&(finding.rule.clone(), finding.subject))
            .expect("a finding blocking because a waiver lapsed must say so");

        assert_eq!(reason.status, SuppressionStatus::Expired);
    }

    /// A finding no disposition names carries no reason.
    ///
    /// The converse control. Without it a recorder that attached a reason to everything would
    /// satisfy both cases above and make the field meaningless.
    #[test]
    fn Test_A_Finding_No_Disposition_Names_Should_Carry_No_Reason()
    {
        let adoption = AdoptionPolicy::default();
        let baseline = BaselinePolicy::default();
        let suppressions = SuppressionPolicy::default();

        let findings = Partitioned_Findings(
            &[Finding_Here()],
            DispositionPolicies {
                adoption: &adoption,
                suppressions: &suppressions,
                baseline: &baseline,
                now: Timestamp::From_Unix_Seconds(0),
            },
        );

        assert!(
            findings.suppression_reasons.is_empty(),
            "a finding nothing suppressed carried a reason: {:?}",
            findings.suppression_reasons
        );
    }
}
