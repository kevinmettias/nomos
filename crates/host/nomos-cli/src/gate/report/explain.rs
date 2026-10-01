//! What the `explain` verb renders: the one finding a query named, and why there is none.

use super::super::ExitCode;
use super::run::Render_Check_Unreadable;
use nomos_capability::RegistryError;
use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::{EvidenceClass, Finding};
use nomos_gate_orchestration::{BaselineDebt, Explanation, GateExplainResult, RuleCalibration, Suppression};
use std::io::Write;
use std::path::Path;

/// Renders what [`nomos_gate_orchestration::Explain_Gate`] answered for `explain`.
///
/// The same `check_outcome`-first match [`super::run::Render_Run`] uses, for the same reason: the exit
/// code for every non-`Judged` variant is a property of *why* nothing was judged, not of
/// the query.
pub(in crate::gate) fn Render_Explain(result: &GateExplainResult, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
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
        Explanation::Found { finding, would_block, floored_by, calibrated_by, suppressed_by, baselined_by, contract } =>
        {
            let tolerance = Toleration {
                floored_by: *floored_by,
                calibrated_by: calibrated_by.as_ref(),
                suppressed_by: suppressed_by.as_ref(),
                baselined_by: baselined_by.as_ref(),
            };
            let found = FoundExplanation { finding, would_block: *would_block, contract: contract.as_ref() };

            Report_Found(found, tolerance, stdout)
        }
    };
}

/// The evidence floor, calibration, suppression or baseline note [`Report_Found`] renders
/// alongside a found explanation's block status -- never more than one at once, since
/// `Explain_Gate` checks them in that order and stops at the first match, but grouped as one
/// value rather than four parameters: what kept a found explanation from blocking is one
/// fact, not four.
#[derive(Clone, Copy)]
#[allow(clippy::struct_field_names)] // each field answers "kept from blocking by ___"; the
                                      // shared suffix is the point, not an accident to rename away
struct Toleration<'a>
{
    /// The declared floor this finding's evidence fell under, when one did.
    ///
    /// `OD-GATE-034`: rendered under its own label, never folded into the three below. A
    /// reader told "calibrated" about a finding whose evidence was simply too weak under this
    /// gate would go looking for a calibration nobody wrote.
    floored_by: Option<EvidenceClass>,
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

/// Renders one found explanation's finding, block status, and evidence-floor, calibration,
/// suppression or baseline note (if any applies), and reduces it to the [`ExitCode`] a real
/// run would decide for this one finding.
fn Report_Found(found: FoundExplanation<'_>, tolerance: Toleration<'_>, stdout: &mut impl Write) -> ExitCode
{
    let _ = writeln!(stdout, "{}", found.finding.Describe());
    let _ = writeln!(stdout, "would block: {}", found.would_block);
    if let Some((record, version)) = found.contract
    {
        let _ = writeln!(stdout, "contract: {record} v{version}");
    }
    if let Some(floor) = tolerance.floored_by
    {
        let _ = writeln!(stdout, "below the evidence floor: this gate requires at least {}", floor.Label());
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
