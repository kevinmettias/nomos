//! Declared phases, thresholds and approvals -- `WF-001`'s "required phases ... thresholds
//! ... approvals ... and blocking behavior" clauses, the three concerns every increment
//! before this one explicitly left unbuilt. `crate::RuleCalibration`'s own doc names them by
//! name as absent from its shape; this module is where they land instead.

use nomos_contracts::{Finding, RuleId};

/// One ordered stage of a real `nomos gate run`, judging only the findings whose rule this
/// phase names.
///
/// Matched by `rules` alone, the same rule-only addressing [`crate::RuleCalibration`] uses
/// and for the same reason: a phase groups whole rules a team has organized into a stage,
/// not individual findings. A rule named by no phase is outside every phase's own scope, and
/// [`Phased_Disposition`] never lets that silently stop counting toward the run's own
/// disposition -- see that function's own doc.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GatePhase
{
    /// This phase's own name -- what a [`PhaseApproval`] names to address it, and what a
    /// caller reads back off [`PhaseOutcome::name`] to tell phases apart. Not a `RuleId`: a
    /// phase is a team's own grouping, not a rule this workspace registers.
    pub name: String,
    /// Which rules' findings this phase judges.
    pub rules: Vec<RuleId>,
    /// How many blocking findings this phase tolerates before it fails.
    pub threshold: PhaseThreshold,
}

/// A phase's own numeric threshold -- `WF-001`'s "thresholds" clause, narrower than a whole
/// run's: a phase judges only the blocking findings its own [`GatePhase::rules`] admits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhaseThreshold
{
    /// Any blocking finding fails this phase -- [`crate::Disposition_Of_Findings`]'s own
    /// rule, narrowed to one phase's own findings rather than a whole run's.
    AnyBlockingFinding,
    /// This phase tolerates up to `max` blocking findings before it fails; `max` itself is
    /// still tolerated.
    MaxBlockingFindings
    {
        max: usize,
    },
}

impl PhaseThreshold
{
    /// Whether `count` blocking findings exceed this threshold.
    #[must_use]
    const fn Exceeded_By(&self, count: usize) -> bool
    {
        return match *self
        {
            Self::AnyBlockingFinding => count > 0,
            Self::MaxBlockingFindings { max } => count > max,
        };
    }
}

/// One approval of a phase that would otherwise fail -- `WF-001`'s "approvals" clause.
///
/// Matched by `phase` name alone, the same coarse, whole-phase addressing
/// [`crate::RuleCalibration`] uses for a whole rule: an approval is a team's own record that
/// a human or agent accepted this phase's findings despite its threshold, not a per-finding
/// override the way [`crate::Suppression`] and [`crate::BaselineDebt`] both are.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhaseApproval
{
    /// The [`GatePhase::name`] this approval covers.
    pub phase: String,
    /// Why this phase was approved despite exceeding its threshold -- required, the same
    /// "never a silent override" discipline [`crate::Suppression::rationale`] and
    /// [`crate::BaselineDebt::rationale`] both already hold.
    pub rationale: String,
}

/// One phase's own outcome, after [`Evaluated_Phases`] judged it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhaseOutcome
{
    /// The [`GatePhase::name`] this outcome answers for.
    pub name: String,
    /// Whether this phase was judged at all -- `false` for every phase after the first one
    /// that failed unapproved. This crate already computed every finding in one pass
    /// (`crate::gate_environment::Judged_Sources`), so nothing here re-runs a rule; what
    /// changes is whether a phase's own verdict is judged and reported at all, or left
    /// unjudged because an earlier phase in the declared order already stopped the run.
    pub ran: bool,
    /// This phase's own blocking findings -- carried even when [`Self::approved`] is `true`,
    /// the same "never silently drop a finding" discipline `crate::GateFindings`'s own doc
    /// already states for calibration, suppression and baseline.
    pub blocking_findings: Vec<Finding>,
    /// Whether a [`PhaseApproval`] matching this phase's name let it pass despite exceeding
    /// its threshold. `false` when nothing needed approving.
    pub approved: bool,
    /// This phase's own verdict.
    pub disposition: PhaseDisposition,
}

/// One phase's own verdict, apart from the run's overall [`crate::GateRunOutcome`] -- a third
/// state, [`Self::Skipped`], that `GateRunOutcome` deliberately does not carry, because a run
/// either judged something or could not judge anything at all, while a phase can additionally
/// simply never have been reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhaseDisposition
{
    /// This phase's blocking findings did not exceed its threshold, or did and a
    /// [`PhaseApproval`] covered them.
    Passed,
    /// This phase's blocking findings exceeded its threshold and nothing approved them.
    Failed,
    /// An earlier phase already failed unapproved, so this phase was never judged.
    Skipped,
}

/// Judges `phases` in declared order over `findings`, stopping at the first phase that fails
/// unapproved -- `WF-001`'s "required phases ... and blocking behavior" read together: a
/// later phase does not run once an earlier one blocks.
///
/// `findings` is every blocking finding a run already reduced down to
/// (`crate::gate_environment::Reduced_Findings`'s own `blocking_findings`, after calibration,
/// suppression and baseline have already decided what still counts), not narrowed to one
/// phase's rules beforehand -- each [`GatePhase`] does its own narrowing by
/// [`GatePhase::rules`], the same "a phase is a partition of the selected rule set"
/// `OD-GATE-023` measured `P40-GATE-PHASES-APPROVALS-2`'s own condition against.
#[must_use]
pub fn Evaluated_Phases(phases: &[GatePhase], findings: &[Finding], approvals: &[PhaseApproval]) -> Vec<PhaseOutcome>
{
    let mut outcomes = Vec::with_capacity(phases.len());
    let mut blocked = false;

    for phase in phases
    {
        if blocked
        {
            outcomes.push(Skipped(phase));
            continue;
        }

        let outcome = Judged_Phase(phase, findings, approvals);
        blocked = matches!(outcome.disposition, PhaseDisposition::Failed);
        outcomes.push(outcome);
    }

    return outcomes;
}

/// One phase's own outcome, judged in isolation -- [`Evaluated_Phases`]'s own per-phase step,
/// split out so that function reads as the sequencing decision alone.
fn Judged_Phase(phase: &GatePhase, findings: &[Finding], approvals: &[PhaseApproval]) -> PhaseOutcome
{
    let blocking_findings: Vec<Finding> = findings.iter().filter(|finding| return phase.rules.contains(&finding.rule)).cloned().collect();
    let exceeded = phase.threshold.Exceeded_By(blocking_findings.len());
    let approved = exceeded && approvals.iter().any(|approval| return approval.phase == phase.name);

    return PhaseOutcome {
        name: phase.name.clone(),
        ran: true,
        blocking_findings,
        approved,
        disposition: if exceeded && !approved { PhaseDisposition::Failed } else { PhaseDisposition::Passed },
    };
}

/// `phase`'s own outcome when an earlier phase already failed unapproved -- never judged, so
/// carries no findings and needs no approval.
fn Skipped(phase: &GatePhase) -> PhaseOutcome
{
    return PhaseOutcome { name: phase.name.clone(), ran: false, blocking_findings: Vec::new(), approved: false, disposition: PhaseDisposition::Skipped };
}

/// `disposition`, downgraded from [`crate::GateRunOutcome::Failed`] to
/// [`crate::GateRunOutcome::Passed`] when `phases` is declared, every one of `blocking`'s
/// findings is named by some phase's own [`GatePhase::rules`], and no phase in `outcomes`
/// failed -- `WF-001`'s "approvals" clause read as what it changes: a run that would
/// otherwise fail can still pass when a team's own staged, approved policy accounts for
/// every blocking finding.
///
/// Never touches a `disposition` that is not already [`crate::GateRunOutcome::Failed`], and
/// never touches it when `phases` is empty -- "no phase policy declared" is the state every
/// existing caller and CI's own `gate run --root .` are already in, so this function is the
/// identity for both. Leaves `disposition` at `Failed` when any blocking finding belongs to
/// no declared phase at all: a phase policy only ever adds a way to still pass, never a way
/// for a finding outside its own scope to silently stop blocking.
#[must_use]
pub fn Phased_Disposition(disposition: crate::GateRunOutcome, phases: &[GatePhase], outcomes: &[PhaseOutcome], blocking: &[Finding]) -> crate::GateRunOutcome
{
    if phases.is_empty() || !matches!(disposition, crate::GateRunOutcome::Failed)
    {
        return disposition;
    }

    let every_finding_phased = blocking.iter().all(|finding| return phases.iter().any(|phase| return phase.rules.contains(&finding.rule)));
    let no_phase_failed = outcomes.iter().all(|outcome| return !matches!(outcome.disposition, PhaseDisposition::Failed));

    return if every_finding_phased && no_phase_failed { crate::GateRunOutcome::Passed } else { disposition };
}

#[cfg(test)]
mod tests
{
    use super::{Evaluated_Phases, GatePhase, PhaseApproval, PhaseDisposition, PhaseThreshold, Phased_Disposition};
    use crate::GateRunOutcome;
    use nomos_contracts::{Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

    fn Finding_For(rule: &str, subject_seed: u8) -> Finding
    {
        return Finding {
            rule: RuleId::New(rule),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([subject_seed; Digest128::BYTE_LENGTH])),
            subject_name: "Example".to_owned(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: "example".to_owned(),
            locations: vec!["a.rs".to_owned()],
        };
    }

    fn Phase(name: &str, rule: &str, threshold: PhaseThreshold) -> GatePhase
    {
        return GatePhase { name: name.to_owned(), rules: vec![RuleId::New(rule)], threshold };
    }

    #[test]
    fn Test_A_Phase_With_No_Blocking_Findings_Should_Pass()
    {
        let phases = vec![Phase("first", "naming-convention", PhaseThreshold::AnyBlockingFinding)];

        let outcomes = Evaluated_Phases(&phases, &[], &[]);

        let [first] = outcomes.as_slice()
        else
        {
            panic!("expected exactly one outcome, got {outcomes:?}");
        };
        assert!(first.ran);
        assert!(matches!(first.disposition, PhaseDisposition::Passed));
    }

    #[test]
    fn Test_A_Later_Phase_Should_Not_Run_When_An_Earlier_One_Fails()
    {
        let phases = vec![
            Phase("first", "naming-convention", PhaseThreshold::AnyBlockingFinding),
            Phase("second", "dependency-direction", PhaseThreshold::AnyBlockingFinding),
        ];
        let findings = vec![Finding_For("naming-convention", 1)];

        let outcomes = Evaluated_Phases(&phases, &findings, &[]);

        let [first, second] = outcomes.as_slice()
        else
        {
            panic!("expected exactly two outcomes, got {outcomes:?}");
        };
        assert!(matches!(first.disposition, PhaseDisposition::Failed));
        assert!(!second.ran, "the second phase must not run once the first failed");
        assert!(matches!(second.disposition, PhaseDisposition::Skipped));
    }

    #[test]
    fn Test_A_Max_Threshold_Should_Tolerate_Findings_Up_To_And_Including_Its_Own_Max()
    {
        let phase = Phase("first", "naming-convention", PhaseThreshold::MaxBlockingFindings { max: 2 });
        let two_findings = vec![Finding_For("naming-convention", 1), Finding_For("naming-convention", 2)];

        let outcomes = Evaluated_Phases(&[phase.clone()], &two_findings, &[]);
        let [tolerated] = outcomes.as_slice()
        else
        {
            panic!("expected exactly one outcome, got {outcomes:?}");
        };
        assert!(matches!(tolerated.disposition, PhaseDisposition::Passed), "two findings must not exceed a max of two");

        let three_findings = vec![Finding_For("naming-convention", 1), Finding_For("naming-convention", 2), Finding_For("naming-convention", 3)];
        let outcomes = Evaluated_Phases(&[phase], &three_findings, &[]);
        let [exceeded] = outcomes.as_slice()
        else
        {
            panic!("expected exactly one outcome, got {outcomes:?}");
        };
        assert!(matches!(exceeded.disposition, PhaseDisposition::Failed), "three findings must exceed a max of two");
    }

    #[test]
    fn Test_An_Approval_Should_Pass_A_Phase_That_Would_Otherwise_Fail()
    {
        let phases = vec![Phase("first", "naming-convention", PhaseThreshold::AnyBlockingFinding)];
        let findings = vec![Finding_For("naming-convention", 1)];
        let approvals = vec![PhaseApproval { phase: "first".to_owned(), rationale: "reviewed and accepted".to_owned() }];

        let outcomes = Evaluated_Phases(&phases, &findings, &approvals);

        let [outcome] = outcomes.as_slice()
        else
        {
            panic!("expected exactly one outcome, got {outcomes:?}");
        };
        assert!(outcome.approved);
        assert!(matches!(outcome.disposition, PhaseDisposition::Passed));
        assert_eq!(outcome.blocking_findings.len(), 1, "an approved phase still carries its own findings rather than hiding them");
    }

    #[test]
    fn Test_An_Approval_Naming_A_Different_Phase_Should_Not_Match()
    {
        let phases = vec![Phase("first", "naming-convention", PhaseThreshold::AnyBlockingFinding)];
        let findings = vec![Finding_For("naming-convention", 1)];
        let approvals = vec![PhaseApproval { phase: "second".to_owned(), rationale: "wrong phase".to_owned() }];

        let outcomes = Evaluated_Phases(&phases, &findings, &approvals);

        let [outcome] = outcomes.as_slice()
        else
        {
            panic!("expected exactly one outcome, got {outcomes:?}");
        };
        assert!(!outcome.approved);
        assert!(matches!(outcome.disposition, PhaseDisposition::Failed));
    }

    #[test]
    fn Test_Phased_Disposition_Should_Leave_A_Passed_Run_Untouched()
    {
        let phases = vec![Phase("first", "naming-convention", PhaseThreshold::AnyBlockingFinding)];

        let disposition = Phased_Disposition(GateRunOutcome::Passed, &phases, &[], &[]);

        assert!(matches!(disposition, GateRunOutcome::Passed));
    }

    #[test]
    fn Test_Phased_Disposition_Should_Leave_A_Failed_Run_Untouched_When_No_Phases_Are_Declared()
    {
        let findings = vec![Finding_For("naming-convention", 1)];

        let disposition = Phased_Disposition(GateRunOutcome::Failed, &[], &[], &findings);

        assert!(matches!(disposition, GateRunOutcome::Failed));
    }

    #[test]
    fn Test_Phased_Disposition_Should_Pass_A_Failed_Run_When_Every_Finding_Is_Approved()
    {
        let phases = vec![Phase("first", "naming-convention", PhaseThreshold::AnyBlockingFinding)];
        let findings = vec![Finding_For("naming-convention", 1)];
        let approvals = vec![PhaseApproval { phase: "first".to_owned(), rationale: "reviewed".to_owned() }];
        let outcomes = Evaluated_Phases(&phases, &findings, &approvals);

        let disposition = Phased_Disposition(GateRunOutcome::Failed, &phases, &outcomes, &findings);

        assert!(matches!(disposition, GateRunOutcome::Passed));
    }

    #[test]
    fn Test_Phased_Disposition_Should_Stay_Failed_When_A_Blocking_Finding_Belongs_To_No_Phase()
    {
        let phases = vec![Phase("first", "naming-convention", PhaseThreshold::AnyBlockingFinding)];
        let findings = vec![Finding_For("dependency-direction", 1)];
        let outcomes = Evaluated_Phases(&phases, &findings, &[]);

        let disposition = Phased_Disposition(GateRunOutcome::Failed, &phases, &outcomes, &findings);

        assert!(matches!(disposition, GateRunOutcome::Failed), "a finding no declared phase covers must not silently stop blocking");
    }
}
