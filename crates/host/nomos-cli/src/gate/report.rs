//! Turning what [`nomos_gate_orchestration::Run`] or [`nomos_gate_orchestration::Run_Gate`]
//! answered into text and an [`ExitCode`].

// file-size: allow this file pairs its production code with its own inline #[cfg(test)]
// module; check-test-coverage keys a test's companion unit off the exact file it is
// textually written in, so these tests cannot move to a sibling file without losing
// their attribution to every function this file declares.
// responsibility: allow same reason -- the coupling that keeps this file whole is
// check-test-coverage's stem-based companion attribution, not a design choice.

use super::ExitCode;
use nomos_capability::RegistryError;
use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::Finding;
use nomos_gate_orchestration::{
    BaselineDebt, Explanation, GateExplainResult, GateOutcome, GateRunOutcome, GateRunResult, RuleCalibration, Suppression,
};
use std::io::Write;
use std::path::Path;

/// Renders what `nomos_gate_orchestration::Run` answered for `plan`.
pub(super) fn Render_Plan(outcome: &GateOutcome, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    return match outcome
    {
        GateOutcome::Planned(plan) =>
        {
            let _ = writeln!(stdout, "rules: {}", plan.rules.len());
            for offer in &plan.rules
            {
                let _ = writeln!(
                    stdout,
                    "  {} ({} v{})",
                    offer.rule, offer.contract_record, offer.contract_record_version
                );
            }

            ExitCode::Ok
        }
        GateOutcome::Contradictory(error) =>
        {
            let _ = writeln!(stderr, "this gate's rule registry is self-contradictory: {error:?}");

            ExitCode::Contradictory
        }
    };
}

/// Renders what [`super::run::Run_Gate`] answered for `run`.
///
/// Matches on [`GateRunResult::check_outcome`] directly, the same shape
/// `check::report::Render` already uses, rather than switching on `disposition`: the exit
/// code for every non-`Judged` variant is a property of *why* nothing was judged, which
/// only `check_outcome` carries.
pub(super) fn Render_Run(result: &GateRunResult, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    return match &result.check_outcome
    {
        CheckOutcome::Unreadable => Render_Check_Unreadable(&result.root, stderr),
        CheckOutcome::Contradictory(error) => Render_Run_Contradictory(error, stderr),
        CheckOutcome::NoSource => Render_Run_No_Source(&result.root, stderr),
        CheckOutcome::NoFacts { files } => Render_Run_No_Facts(&result.root, *files, stderr),
        CheckOutcome::Judged { findings, .. } => Report_Judged(findings, result, stdout),
    };
}

/// The check layer beneath this gate run has its own composition contradictory.
fn Render_Run_Contradictory(error: &RegistryError, stderr: &mut impl Write) -> ExitCode
{
    let _ = writeln!(
        stderr,
        "the check layer beneath this gate run has its own composition \
         contradictory, so no fact it produced would have been offered by anybody: \
         {error}"
    );

    return ExitCode::Contradictory;
}

/// The walk found no source under `root`, so `run` judged nothing.
fn Render_Run_No_Source(root: &Path, stderr: &mut impl Write) -> ExitCode
{
    let _ = writeln!(
        stderr,
        "no Rust source found under `{}`, so nothing was judged.\n\
         A clean result here would mean only that the walk found nothing.",
        root.display()
    );

    return ExitCode::Vacuous;
}

/// Source was found under `root` but no syntax fact was materialized for any of it.
fn Render_Run_No_Facts(root: &Path, files: usize, stderr: &mut impl Write) -> ExitCode
{
    let _ = writeln!(
        stderr,
        "{files} file(s) were read under `{}` and no syntax fact was materialized \
         for any of them, so no mirror claim could be resolved.\n\
         A clean result here would mean only that the analysis never ran.",
        root.display()
    );

    return ExitCode::Vacuous;
}

/// Renders a judged run's findings and reduces its disposition to an [`ExitCode`] -- the
/// one arm of [`Render_Run`] that does real work, the same way `check::report::Render`
/// delegates its own `Judged` arm to a dedicated function rather than folding it into the
/// outer match.
fn Report_Judged(findings: &[Finding], result: &GateRunResult, stdout: &mut impl Write) -> ExitCode
{
    let _ = writeln!(stdout, "run: {}", result.run);

    for finding in findings
    {
        let _ = writeln!(stdout, "{}", finding.Describe());
    }

    let _ = writeln!(
        stdout,
        "\n{} finding(s), {} of which can fail a build, {} calibrated, {} suppressed, {} baselined",
        findings.len(),
        result.findings.blocking_findings.len(),
        result.findings.calibrated_findings.len(),
        result.findings.suppressed_findings.len(),
        result.findings.baselined_findings.len()
    );

    return Exit_Code_For(result.disposition);
}

/// Renders what [`nomos_gate_orchestration::Compare_Gate_Runs`] answered for `compare`.
///
/// # Why a difference is never a verdict
///
/// `compare` is `0` whenever both sides were judged, however much moved between them. A
/// finding added between two trees is a fact about the difference; whether that fact should
/// fail a build is a policy question, and the policy that would answer it — which additions
/// are tolerable, against which baseline — is `GateCommand`'s own
/// `suppressions`/`baseline`/`adoption`, which no flag authors yet. Reading `Violations`
/// onto an addition here would be inventing that policy at the exit code, where nobody
/// declared it.
///
/// A side that was never judged is different in kind, and keeps the code `run` already
/// gives that reason: there is no difference to report, rather than one that happens to be
/// empty.
pub(super) fn Render_Compare(
    baseline: &GateRunResult,
    candidate: &GateRunResult,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> ExitCode
{
    if let Some(code) = Unjudged(baseline, "--root", stderr)
    {
        return code;
    }
    if let Some(code) = Unjudged(candidate, "--against", stderr)
    {
        return code;
    }

    let compared = nomos_gate_orchestration::Compare_Gate_Runs(baseline, candidate);

    let _ = writeln!(
        stdout,
        "baseline: {} ({})\ncandidate: {} ({})",
        compared.baseline,
        baseline.root.display(),
        compared.candidate,
        candidate.root.display()
    );

    for finding in &compared.added
    {
        let _ = writeln!(stdout, "+ {}", finding.Describe());
    }
    for finding in &compared.removed
    {
        let _ = writeln!(stdout, "- {}", finding.Describe());
    }
    for change in &compared.changed
    {
        let _ = writeln!(
            stdout,
            "~ {} {}: {:?} -> {:?}",
            change.rule, change.subject_name, change.before, change.after
        );
    }

    let _ = writeln!(
        stdout,
        "\n{} added, {} removed, {} changed disposition",
        compared.added.len(),
        compared.removed.len(),
        compared.changed.len()
    );

    return ExitCode::Ok;
}

/// The exit code for a side of a `compare` that never reached a judgment, and the reason
/// why, named by the flag that selected it.
///
/// `None` when the side was judged and there is nothing to report about it. Reuses
/// [`Render_Run`]'s own refusals rather than restating them, so `compare` cannot drift into
/// describing an unreadable tree differently from `run`.
fn Unjudged(result: &GateRunResult, flag: &str, stderr: &mut impl Write) -> Option<ExitCode>
{
    if matches!(result.check_outcome, CheckOutcome::Judged { .. })
    {
        return None;
    }

    let _ = writeln!(stderr, "{flag} was not judged, so there is no difference to report:");

    return Some(match &result.check_outcome
    {
        CheckOutcome::Unreadable => Render_Check_Unreadable(&result.root, stderr),
        CheckOutcome::Contradictory(error) => Render_Run_Contradictory(error, stderr),
        CheckOutcome::NoSource => Render_Run_No_Source(&result.root, stderr),
        CheckOutcome::NoFacts { files } => Render_Run_No_Facts(&result.root, *files, stderr),
        // Guarded by the matches! above, which returns before reaching here.
        CheckOutcome::Judged { .. } => ExitCode::Ok,
    });
}

/// Reduces a real run's disposition to the [`ExitCode`] it reports.
fn Exit_Code_For(disposition: GateRunOutcome) -> ExitCode
{
    return match disposition
    {
        GateRunOutcome::Failed => ExitCode::Violations,
        GateRunOutcome::Passed => ExitCode::Ok,
        // Run_Gate only produces Indeterminate for a CheckOutcome that never reached Judged,
        // and this function is only ever called from the Judged arm of Render_Run, so that
        // disposition cannot arrive here.
        GateRunOutcome::Indeterminate => unreachable!(
            "nomos_gate_orchestration::Disposition never returns Indeterminate; \
             Run_Gate only assigns it for a CheckOutcome that never reached Judged, \
             and this arm is Judged's own"
        ),
    };
}

/// Renders what [`nomos_gate_orchestration::Explain_Gate`] answered for `explain`.
///
/// The same `check_outcome`-first match [`Render_Run`] uses, for the same reason: the exit
/// code for every non-`Judged` variant is a property of *why* nothing was judged, not of
/// the query.
pub(super) fn Render_Explain(result: &GateExplainResult, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    return match &result.check_outcome
    {
        CheckOutcome::Unreadable => Render_Check_Unreadable(&result.root, stderr),
        CheckOutcome::Contradictory(error) => Render_Explain_Contradictory(error, stderr),
        CheckOutcome::NoSource => Render_Explain_No_Source(&result.root, stderr),
        CheckOutcome::NoFacts { files } => Render_Explain_No_Facts(&result.root, *files, stderr),
        CheckOutcome::Judged { .. } => Report_Explanation(&result.explanation, stdout),
    };
}

/// The check layer beneath this gate explain has its own composition contradictory.
fn Render_Explain_Contradictory(error: &RegistryError, stderr: &mut impl Write) -> ExitCode
{
    let _ = writeln!(
        stderr,
        "the check layer beneath this gate explain has its own composition \
         contradictory, so no fact it produced would have been offered by anybody: \
         {error}"
    );

    return ExitCode::Contradictory;
}

/// The walk found no source under `root`, so `explain`'s query cannot be answered.
fn Render_Explain_No_Source(root: &Path, stderr: &mut impl Write) -> ExitCode
{
    let _ = writeln!(
        stderr,
        "no Rust source found under `{}`, so nothing was judged and the query \
         cannot be answered.",
        root.display()
    );

    return ExitCode::Vacuous;
}

/// Source was found under `root` but no syntax fact was materialized for any of it, so
/// `explain`'s query cannot be answered.
fn Render_Explain_No_Facts(root: &Path, files: usize, stderr: &mut impl Write) -> ExitCode
{
    let _ = writeln!(
        stderr,
        "{files} file(s) were read under `{}` and no syntax fact was materialized \
         for any of them, so the query cannot be answered.",
        root.display()
    );

    return ExitCode::Vacuous;
}

/// Renders `explain`'s answer and reduces it to an [`ExitCode`] -- `Violations` when the
/// named finding would block a real run, `Ok` otherwise (not found, or found but not
/// blocking), the same "the exit code mirrors what `run` would decide for this one
/// finding" reasoning `nomos_gate_orchestration::explain`'s own doc gives.
fn Report_Explanation(explanation: &Explanation, stdout: &mut impl Write) -> ExitCode
{
    return match explanation
    {
        Explanation::NotFound =>
        {
            let _ = writeln!(stdout, "not found");

            ExitCode::Ok
        }
        Explanation::Found { finding, would_block, calibrated_by, suppressed_by, baselined_by, contract } =>
        {
            let tolerance = Toleration {
                calibrated_by: calibrated_by.as_ref(),
                suppressed_by: suppressed_by.as_ref(),
                baselined_by: baselined_by.as_ref(),
            };
            let found = FoundExplanation { finding, would_block: *would_block, contract: contract.as_ref() };

            Report_Found(found, tolerance, stdout)
        }
    };
}

/// The calibration, suppression or baseline note [`Report_Found`] renders alongside a found
/// explanation's block status -- never more than one at once, since `Explain_Gate` checks
/// them in that order and stops at the first match, but grouped as a triple rather than
/// three parameters: what a found explanation was tolerated by is one fact, not three.
#[derive(Clone, Copy)]
#[allow(clippy::struct_field_names)] // each field answers "tolerated by ___"; the shared
                                      // suffix is the point, not an accident to rename away
struct Toleration<'a>
{
    calibrated_by: Option<&'a RuleCalibration>,
    suppressed_by: Option<&'a Suppression>,
    baselined_by: Option<&'a BaselineDebt>,
}

/// A found explanation's finding, block status and contract -- grouped separately from
/// [`Toleration`] because these three come directly off [`Explanation::Found`], while
/// `Toleration` is the one of at most three ways that finding was tolerated.
#[derive(Clone, Copy)]
struct FoundExplanation<'a>
{
    finding: &'a Finding,
    would_block: bool,
    contract: Option<&'a (String, u32)>,
}

/// Renders one found explanation's finding, block status, and calibration, suppression or
/// baseline note (if any applies), and reduces it to the [`ExitCode`] a real run would
/// decide for this one finding.
fn Report_Found(found: FoundExplanation<'_>, tolerance: Toleration<'_>, stdout: &mut impl Write) -> ExitCode
{
    let _ = writeln!(stdout, "{}", found.finding.Describe());
    let _ = writeln!(stdout, "would block: {}", found.would_block);
    if let Some((record, version)) = found.contract
    {
        let _ = writeln!(stdout, "contract: {record} v{version}");
    }
    if let Some(calibration) = tolerance.calibrated_by
    {
        let _ = writeln!(stdout, "calibrated by: {}", calibration.rationale);
    }
    if let Some(suppression) = tolerance.suppressed_by
    {
        let _ = writeln!(
            stdout,
            "suppressed by: {:?} — {} (owner: {})",
            suppression.disposition, suppression.rationale, suppression.owner
        );
    }
    if let Some(debt) = tolerance.baselined_by
    {
        let _ = writeln!(stdout, "baselined by: {}", debt.rationale);
    }

    return if found.would_block { ExitCode::Violations } else { ExitCode::Ok };
}

/// The root does not exist, is not a directory, or its walk could not be ingested -- the
/// same message whether the caller was `run` or `explain`, since neither verb's own
/// question ever got asked.
fn Render_Check_Unreadable(root: &Path, stderr: &mut impl Write) -> ExitCode
{
    let _ = writeln!(
        stderr,
        "cannot judge `{}`: not a directory, or its walk could not be ingested as a \
         workspace state",
        root.display()
    );

    return ExitCode::Contradictory;
}

#[cfg(test)]
mod tests
{
    //! [`super::Render_Plan`], [`super::Render_Run`] and [`super::Render_Explain`], exercised.
    //!
    //! Split from `report.rs` itself once that file passed the ~500-line review trigger --
    //! `report.rs` is the rendering logic, this is its own coverage, the same split this
    //! workspace already keeps between `spec.rs` and `spec/tests.rs`.

    use super::*;
    use nomos_check_orchestration::{Claim, Examined};
    use nomos_contracts::{Applicability, Digest128, EvidenceClass, GateCategory, RuleId, SubjectId};
    use nomos_gate_orchestration::{Fresh_Run_Id, GateFindings};
    use nomos_platform::Timestamp;
    use std::path::PathBuf;

    /// A check outcome that never reached `Judged` must render as `Vacuous`, the same claim
    /// `Render_Check_Unreadable`'s siblings already make for `run`'s own non-judged arms --
    /// `Render_Run` picks the same arm for `explain`'s `NoSource`.
    #[test]
    fn Test_Render_Run_Should_Report_Vacuous_When_The_Check_Outcome_Never_Reached_Judged()
    {
        let result = GateRunResult {
            run: Fresh_Run_Id(Timestamp::From_Unix_Seconds(0)),
            root: PathBuf::from("does/not/matter"),
            check_outcome: CheckOutcome::NoSource,
            findings: Empty_Findings(),
            disposition: GateRunOutcome::Indeterminate,
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = Render_Run(&result, &mut stdout, &mut stderr);

        let rendered_stderr = String::from_utf8_lossy(&stderr).into_owned();
        assert_eq!(code, ExitCode::Vacuous, "{rendered_stderr}");
        assert!(rendered_stderr.contains("nothing was judged"), "{rendered_stderr}");
        assert!(String::from_utf8_lossy(&stdout).is_empty());
    }

    /// A judged run with nothing blocking reports `Ok` and names its own `RunId` -- the
    /// "real work" arm `Report_Judged` does, end to end at this function's own boundary.
    #[test]
    fn Test_Render_Run_Should_Report_The_RunId_And_Ok_When_Nothing_Blocks()
    {
        let run = Fresh_Run_Id(Timestamp::From_Unix_Seconds(0));
        let result = GateRunResult {
            run,
            root: PathBuf::from("."),
            check_outcome: CheckOutcome::Judged {
                findings: Vec::new(),
                examined: Examined { files: 1, facts: 1 },
                claim: Claim::Complete,
            },
            findings: Empty_Findings(),
            disposition: GateRunOutcome::Passed,
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = Render_Run(&result, &mut stdout, &mut stderr);

        let rendered = String::from_utf8_lossy(&stdout).into_owned();
        assert_eq!(code, ExitCode::Ok, "{rendered}");
        assert!(rendered.contains(&format!("run: {run}")), "{rendered}");
        assert!(rendered.contains("0 finding(s), 0 of which can fail a build"), "{rendered}");
        assert!(String::from_utf8_lossy(&stderr).is_empty());
    }

    /// A judged run with one blocking finding reports `Violations` and names the finding --
    /// `Exit_Code_For`'s `Failed` arm, only reachable through `Report_Judged`.
    #[test]
    fn Test_Render_Run_Should_Report_Violations_When_A_Finding_Blocks()
    {
        let finding = Example_Finding(GateCategory::Blocking);
        let result = GateRunResult {
            run: Fresh_Run_Id(Timestamp::From_Unix_Seconds(0)),
            root: PathBuf::from("."),
            check_outcome: CheckOutcome::Judged {
                findings: vec![finding.clone()],
                examined: Examined { files: 1, facts: 1 },
                claim: Claim::Complete,
            },
            findings: GateFindings { blocking_findings: vec![finding.clone()], ..Empty_Findings() },
            disposition: GateRunOutcome::Failed,
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = Render_Run(&result, &mut stdout, &mut stderr);

        let rendered = String::from_utf8_lossy(&stdout).into_owned();
        assert_eq!(code, ExitCode::Violations, "{rendered}");
        assert!(rendered.contains(&finding.Describe()), "{rendered}");
        assert!(rendered.contains("1 finding(s), 1 of which can fail a build"), "{rendered}");
    }

    /// `explain` shares `run`'s own non-judged rendering, so a check outcome that never
    /// reached `Judged` must report `Vacuous` here too, regardless of what `explanation`
    /// carries.
    #[test]
    fn Test_Render_Explain_Should_Report_Vacuous_When_The_Check_Outcome_Never_Reached_Judged()
    {
        let result = GateExplainResult {
            root: PathBuf::from("does/not/matter"),
            check_outcome: CheckOutcome::NoSource,
            explanation: Explanation::NotFound,
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = Render_Explain(&result, &mut stdout, &mut stderr);

        let rendered_stderr = String::from_utf8_lossy(&stderr).into_owned();
        assert_eq!(code, ExitCode::Vacuous, "{rendered_stderr}");
        assert!(rendered_stderr.contains("the query cannot be answered"), "{rendered_stderr}");
    }

    /// A judged tree in which no finding names the query's location answers `not found` and
    /// exits clean.
    #[test]
    fn Test_Render_Explain_Should_Report_Not_Found_When_Judged_And_No_Finding_Matches()
    {
        let result = GateExplainResult {
            root: PathBuf::from("."),
            check_outcome: CheckOutcome::Judged {
                findings: Vec::new(),
                examined: Examined { files: 1, facts: 1 },
                claim: Claim::Complete,
            },
            explanation: Explanation::NotFound,
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = Render_Explain(&result, &mut stdout, &mut stderr);

        let rendered = String::from_utf8_lossy(&stdout).into_owned();
        assert_eq!(code, ExitCode::Ok, "{rendered}");
        assert!(rendered.contains("not found"), "{rendered}");
        assert!(String::from_utf8_lossy(&stderr).is_empty());
    }

    /// A found finding that would block a real run reports `Violations` and names both the
    /// finding and its block status -- `Report_Found`'s own "real work" arm.
    #[test]
    fn Test_Render_Explain_Should_Report_Would_Block_For_A_Found_Blocking_Finding()
    {
        let finding = Example_Finding(GateCategory::Blocking);
        let result = GateExplainResult {
            root: PathBuf::from("."),
            check_outcome: CheckOutcome::Judged {
                findings: vec![finding.clone()],
                examined: Examined { files: 1, facts: 1 },
                claim: Claim::Complete,
            },
            explanation: Explanation::Found {
                finding: Box::new(finding.clone()),
                would_block: true,
                calibrated_by: None,
                suppressed_by: None,
                baselined_by: None,
                contract: None,
            },
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = Render_Explain(&result, &mut stdout, &mut stderr);

        let rendered = String::from_utf8_lossy(&stdout).into_owned();
        assert_eq!(code, ExitCode::Violations, "{rendered}");
        assert!(rendered.contains("would block: true"), "{rendered}");
        assert!(rendered.contains(&finding.Describe()), "{rendered}");
    }

    /// A real, composed registry -- `nomos_gate_orchestration::Run` never touches a
    /// filesystem or a subprocess for `plan`, so this drives `Render_Plan` against a
    /// genuine `GateOutcome` rather than a hand-built one.
    #[test]
    fn Test_Render_Plan_Should_Report_Ok_And_List_Every_Registered_Rule()
    {
        let outcome = nomos_gate_orchestration::Run(&nomos_gate_orchestration::GateCommand::default());
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = Render_Plan(&outcome, &mut stdout, &mut stderr);

        let rendered = String::from_utf8_lossy(&stdout).into_owned();
        assert_eq!(code, ExitCode::Ok, "{rendered}{}", String::from_utf8_lossy(&stderr));
        assert!(rendered.starts_with("rules: "), "{rendered}");
    }

    /// One real finding, distinguishable from another only by the gate category a caller
    /// passes in -- everything else about it is incidental to what these tests check.
    fn Example_Finding(gate: GateCategory) -> Finding
    {
        return Finding {
            rule: RuleId::New("unread-reaches-finding"),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([9; Digest128::BYTE_LENGTH])),
            subject_name: "Example::Subject".to_owned(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate,
            summary: "reaches an unread item".to_owned(),
            locations: vec!["a.rs".to_owned()],
        };
    }

    /// No finding blocked, calibrated, suppressed or baselined -- the starting point every
    /// test above that does not care about one of these buckets builds on.
    fn Empty_Findings() -> GateFindings
    {
        return GateFindings {
            blocking_findings: Vec::new(),
            calibrated_findings: Vec::new(),
            suppressed_findings: Vec::new(),
            baselined_findings: Vec::new(),
        };
    }
}
