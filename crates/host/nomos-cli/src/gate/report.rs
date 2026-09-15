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
    Admissibility, BaselineDebt, Explanation, GateExplainResult, GateOutcome, GateRunOutcome, GateRunResult, NoVerdict, RuleCalibration, Suppression,
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
        CheckOutcome::Judged { findings, .. } => Report_Judged(findings, result, stdout, stderr),
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
fn Report_Judged(findings: &[Finding], result: &GateRunResult, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
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

    Report_Unmatched_Policy(result, stdout);

    return Exit_Code_For(result, stderr);
}

/// Names every declared policy entry that matched no finding in this run.
///
/// `OD-GATE-024`: an entry that matches nothing is reported rather than silently ignored,
/// because an author who wrote one cannot otherwise tell a mis-spelling from a finding that
/// has since been fixed. It does not change the exit code — a policy legitimately outlives
/// the finding it was written for, and a repository whose debt was paid must not fail its
/// own gate for having paid it.
///
/// Silent when every entry matched, and when none was declared: a header over an empty list
/// on every clean run is the noise that teaches a reader to skip the line that matters.
fn Report_Unmatched_Policy(result: &GateRunResult, stdout: &mut impl Write)
{
    if result.unmatched_policy.is_empty()
    {
        return;
    }

    let _ = writeln!(stdout, "\ndeclared policy that matched nothing:");
    for entry in &result.unmatched_policy
    {
        let _ = writeln!(stdout, "  {entry}");
    }
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

    let compared = match nomos_gate_orchestration::Compare_Gate_Runs(baseline, candidate)
    {
        Ok(compared) => compared,
        Err(refused) =>
        {
            // Not `Violations`: nothing about the tree was judged badly, and nothing about the
            // difference was judged at all. This is the analysis refusing to answer, which is
            // the distinction `Unjudged` above already draws for a side that could not be read.
            let _ = writeln!(
                stderr,
                "cannot compare: run {} produced {} finding(s) that share an occurrence identity \
                 with another finding in the same run, so indexing them would silently drop one.\n\
                 This is a defect in the identity material rather than something to work around: \
                 two findings agreeing on rule, subject, summary and locations are one occurrence \
                 to every consumer. The colliding findings are:",
                refused.run,
                refused.collisions.len().saturating_mul(2)
            );

            for collision in &refused.collisions
            {
                let _ = writeln!(stderr, "  {}", collision.first.Describe());
                let _ = writeln!(stderr, "  {}", collision.second.Describe());
            }

            return ExitCode::Contradictory;
        }
    };

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
        // `locations` is appended because occurrence scope made the rule-and-subject pair
        // non-unique: one subject can now contribute several changes, and without the geometry
        // they would print as identical lines. Omitted when a finding carries none, rather than
        // printing an empty bracket that says nothing.
        let where_it_is =
            if change.locations.is_empty() { String::new() } else { format!(" [{}]", change.locations.join(", ")) };

        let _ = writeln!(
            stdout,
            "~ {} {}{}: {:?} -> {:?}",
            change.rule, change.subject_name, where_it_is, change.before, change.after
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
///
/// Every arm is reachable from the `Judged` arm this is called under, `Indeterminate`
/// included: `Run_Gate` assigns that disposition *after* a full judgment in two deliberate
/// cases, which [`Render_Run_No_Verdict`] names. This function asserted the opposite and
/// aborted the process on both until the cases were measured.
///
/// Takes the whole result rather than the disposition alone, because the cause and the
/// disposition are one answer: reading `Indeterminate` without the `NoVerdict` beside it is
/// exactly the half-answer this repository had before the result carried one.
fn Exit_Code_For(result: &GateRunResult, stderr: &mut impl Write) -> ExitCode
{
    return match result.disposition
    {
        GateRunOutcome::Failed => ExitCode::Violations,
        GateRunOutcome::Passed => ExitCode::Ok,
        GateRunOutcome::Indeterminate => Render_Run_No_Verdict(result.no_verdict.as_ref(), stderr),
    };
}

/// The tree was judged, the findings reported above are all of them, and no verdict was
/// reached. Says which of the three mechanisms produced that.
///
/// Each wants a different reaction, which is the whole reason `GateRunResult` carries the
/// cause rather than only the disposition. Two are a broken `nomos-gate.json` and send a
/// reader to that file with the reader's own message about it -- for a mis-spelled key,
/// the key. The third is `OD-GATE-016`'s coverage floor doing exactly what the repository
/// asked it to, where there is no fault to find and a reader sent looking for one would
/// waste the trip.
///
/// `None` is not reachable from a real `Run_Gate` today, which fills the cause on every
/// path that produces this disposition after judging. It is still answered rather than
/// asserted away: an unreachable claim about this exact arm is what aborted the process
/// before, and the honest rendering of a missing reason is to say the reason is missing.
fn Render_Run_No_Verdict(cause: Option<&NoVerdict>, stderr: &mut impl Write) -> ExitCode
{
    let _ = match cause
    {
        Some(NoVerdict::UnreadablePolicy(detail)) => writeln!(
            stderr,
            "\nthis run judged the tree and reached no verdict: the `nomos-gate.json` under its \
             root could not be read, so there were no declared rules to reduce the findings \
             above by.\n  {detail}"
        ),
        Some(NoVerdict::MalformedPolicy(detail)) => writeln!(
            stderr,
            "\nthis run judged the tree and reached no verdict: the `nomos-gate.json` under its \
             root is not a policy this reader accepts, so there were no declared rules to \
             reduce the findings above by.\n  {detail}"
        ),
        Some(NoVerdict::IncompleteCoverage) => writeln!(
            stderr,
            "\nthis run judged the tree, found nothing that can fail a build, and is still not \
             a pass: its declared coverage floor is `require-completeness` and some rules \
             could not look.\n\
             Nothing is wrong with the tree or with the policy. A clean result here would \
             mean only that the rules which did run found nothing."
        ),
        None => writeln!(
            stderr,
            "\nthis run judged the tree and reached no verdict, and did not record why."
        ),
    };

    return ExitCode::Contradictory;
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

/// What the architecture says about an edge that does not exist yet.
///
/// `Permitted` and `NotJudged` are both `Ok`; only `Refused` is `Violations`. That pairing is
/// deliberate and it is the one place this verb could mislead: a caller scripting `admits`
/// into a pre-commit hook reads the exit code, and giving `NotJudged` a failing code would
/// stop a build over a crate this workspace has no opinion about, while giving `Refused` a
/// clean one would let the edge through. So the code carries the judgment and the text
/// carries the difference between "yes" and "no answer" — which the line always says, in
/// words, whichever code it exits with.
pub(super) fn Render_Admits(answer: Admissibility, pair: (&str, &str), stdout: &mut impl Write) -> ExitCode
{
    let (depending, depended) = pair;

    return match answer
    {
        Admissibility::Permitted =>
        {
            let _ = writeln!(stdout, "permitted: {depending} may name {depended}.");
            ExitCode::Ok
        }
        Admissibility::Refused =>
        {
            let _ = writeln!(
                stdout,
                "refused: {depending} may not name {depended} under this repository's declared \
                 architecture."
            );
            ExitCode::Violations
        }
        Admissibility::NotJudged =>
        {
            let _ = writeln!(
                stdout,
                "not judged: {depending} or {depended} has no declared zone, so there is nothing \
                 to judge this edge against. This is not permission."
            );
            ExitCode::Ok
        }
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
            no_verdict: None,
            unmatched_policy: Vec::new(),
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

    /// A judged run whose disposition is `Indeterminate` says there is no verdict, says which
    /// mechanism produced that, and exits `Contradictory` -- instead of aborting the process.
    ///
    /// The arm these cover was `unreachable!` until it was measured, on the claim that
    /// `Run_Gate` only assigns `Indeterminate` to a run which never reached `Judged`. It
    /// assigns it to a judged run in three cases, and a real `nomos gate run` aborted with
    /// 101 on every one.
    ///
    /// One test per cause rather than one over all of them, because the whole point of
    /// carrying a cause is that the three read differently to a person: two send a reader to
    /// the policy file, and the third tells them not to go looking for a fault at all.
    #[test]
    fn Test_Render_Run_Should_Name_A_Malformed_Policy_As_The_Reason_There_Is_No_Verdict()
    {
        let finding = Example_Finding(GateCategory::Advisory);
        let result = Judged_Without_A_Verdict(&finding, Some(NoVerdict::MalformedPolicy("unknown field `basline`".to_owned())));
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = Render_Run(&result, &mut stdout, &mut stderr);

        let rendered_stderr = String::from_utf8_lossy(&stderr).into_owned();
        assert_eq!(code, ExitCode::Contradictory, "{rendered_stderr}");
        assert!(rendered_stderr.contains("not a policy this reader accepts"), "{rendered_stderr}");
        assert!(rendered_stderr.contains("basline"), "{rendered_stderr}");
        // The abort came *after* the findings were written, so what it destroyed was the
        // verdict line and the exit code. A fix that reported the state by dropping the
        // report would be the worse answer.
        assert!(String::from_utf8_lossy(&stdout).contains(&finding.Describe()));
    }

    #[test]
    fn Test_Render_Run_Should_Name_An_Unreadable_Policy_Separately_From_A_Malformed_One()
    {
        let finding = Example_Finding(GateCategory::Advisory);
        let result = Judged_Without_A_Verdict(&finding, Some(NoVerdict::UnreadablePolicy("nomos-gate.json: PermissionDenied".to_owned())));
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = Render_Run(&result, &mut stdout, &mut stderr);

        let rendered_stderr = String::from_utf8_lossy(&stderr).into_owned();
        assert_eq!(code, ExitCode::Contradictory, "{rendered_stderr}");
        assert!(rendered_stderr.contains("could not be read"), "{rendered_stderr}");
        assert!(rendered_stderr.contains("PermissionDenied"), "{rendered_stderr}");
    }

    /// The coverage floor is the one cause where nothing is wrong, so its rendering says so
    /// rather than sending a reader to look for a fault that is not there.
    #[test]
    fn Test_Render_Run_Should_Say_Nothing_Is_Wrong_When_The_Coverage_Floor_Withheld_The_Pass()
    {
        let finding = Example_Finding(GateCategory::Advisory);
        let result = Judged_Without_A_Verdict(&finding, Some(NoVerdict::IncompleteCoverage));
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = Render_Run(&result, &mut stdout, &mut stderr);

        let rendered_stderr = String::from_utf8_lossy(&stderr).into_owned();
        assert_eq!(code, ExitCode::Contradictory, "{rendered_stderr}");
        assert!(rendered_stderr.contains("require-completeness"), "{rendered_stderr}");
        assert!(rendered_stderr.contains("Nothing is wrong"), "{rendered_stderr}");
    }

    /// A cause `Run_Gate` does not currently leave unset is still answered rather than
    /// asserted away: an unreachable claim about this exact arm is what aborted the process
    /// before it carried one, and the honest rendering of a missing reason says it is missing.
    #[test]
    fn Test_Render_Run_Should_Say_The_Reason_Is_Missing_Rather_Than_Assume_One()
    {
        let finding = Example_Finding(GateCategory::Advisory);
        let result = Judged_Without_A_Verdict(&finding, None);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = Render_Run(&result, &mut stdout, &mut stderr);

        let rendered_stderr = String::from_utf8_lossy(&stderr).into_owned();
        assert_eq!(code, ExitCode::Contradictory, "{rendered_stderr}");
        assert!(rendered_stderr.contains("did not record why"), "{rendered_stderr}");
    }

    /// A run that judged `finding` and came out with no verdict, for `cause`.
    fn Judged_Without_A_Verdict(finding: &Finding, cause: Option<NoVerdict>) -> GateRunResult
    {
        return GateRunResult {
            no_verdict: cause,
            unmatched_policy: Vec::new(),
            run: Fresh_Run_Id(Timestamp::From_Unix_Seconds(0)),
            root: PathBuf::from("."),
            check_outcome: CheckOutcome::Judged {
                findings: vec![finding.clone()],
                examined: Examined { files: 1, facts: 1 },
                claim: Claim::Incomplete,
            },
            findings: Empty_Findings(),
            disposition: GateRunOutcome::Indeterminate,
        };
    }

    /// A judged run with nothing blocking reports `Ok` and names its own `RunId` -- the
    /// "real work" arm `Report_Judged` does, end to end at this function's own boundary.
    #[test]
    fn Test_Render_Run_Should_Report_The_RunId_And_Ok_When_Nothing_Blocks()
    {
        let run = Fresh_Run_Id(Timestamp::From_Unix_Seconds(0));
        let result = GateRunResult {
            no_verdict: None,
            unmatched_policy: Vec::new(),
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
            no_verdict: None,
            unmatched_policy: Vec::new(),
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
            baselined_findings: Vec::new(), suppression_reasons: Default::default() };
    }
}
