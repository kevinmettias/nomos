//! `compare`: what changed between two real gate runs.
//!
//! `plan`, `run` and `explain` were all real and CLI-wired before this file existed;
//! `compare` was named in `crate::gate_command`'s own module doc as deliberately absent,
//! the same "no invented shape ahead of a real body" discipline that doc states for every
//! verb this crate has not yet built. It is the verb a gate is actually adopted to
//! answer: not "what does this run say" but "what changed since the run I already
//! trusted" -- a baseline is a number nobody can diff without it, and an adoption
//! decision has no before and after.
//!
//! [`RunId`] already exists and is threaded through [`crate::GateRunResult::run`]
//! (`P13-GATE-REPORT-RUNID`), which is the identity a comparison needs; this file is the
//! missing verb, not a missing identity. [`Compare_Gate_Runs`] takes two already-produced
//! [`crate::GateRunResult`]s directly rather than looking either up by [`RunId`] from a
//! store this crate does not own -- persisting and retrieving a past run by its own
//! identity is a composition-root concern, the same division [`crate::Run_Gate`] itself
//! draws by taking an already-walked tree rather than a root to read.
//!
//! # Why disposition, not a raw finding diff
//!
//! Two runs over an unchanged tree can still disagree about what a finding *means* if the
//! [`crate::AdoptionPolicy`], [`crate::SuppressionPolicy`] or [`crate::BaselinePolicy`]
//! between them changed -- the exact case a real adoption decision needs to see. Diffing
//! [`crate::GateRunResult::findings`]'s own four buckets, rather than
//! [`crate::GateRunResult::check_outcome`]'s raw list, answers "did tightening or loosening
//! a policy change what can fail this build" as directly as `compare` can be asked to.

use nomos_contracts::{Finding, RuleId, RunId, SubjectId};
use nomos_model::{FindingOccurrenceId, Occurrence_Collisions_In, OccurrenceCollision};
use std::collections::BTreeMap;

use crate::{GateFindings, GateRunResult, SuppressionReason};

/// Which of [`GateFindings`]'s own four buckets a finding fell into.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FindingDisposition
{
    Blocking,
    Calibrated,
    Suppressed,
    Baselined,
}

/// A run whose findings do not yield one identity each, so it cannot be compared.
///
/// # Why this refuses instead of coping
///
/// Because every way of coping is a way of losing a finding quietly. Two findings sharing a
/// [`FindingOccurrenceId`] cannot both be a map's value at that key, and the mechanism that
/// would decide between them -- `BTreeMap::insert` returning the displaced value into a `let _`
/// -- is exactly how 103 of this repository's 253 findings were being discarded before this
/// type existed. Not by a decision anybody made: by an ignored return value.
///
/// A collision means the identity material is no longer sufficient for the findings this
/// workspace now produces, which is a defect in the identity rather than a condition a caller
/// can sensibly handle. `FindingOccurrenceId`'s own documentation states the injectivity claim
/// narrowly and names this as the thing that turns a future model change from something
/// somebody has to remember into something that fails. So the refusal is the feature.
///
/// # Why it names the run and both findings
///
/// Naming the run says which side of the comparison to look at -- a caller holds two, and a
/// collision in one says nothing about the other. Naming both findings is the only report that
/// can be acted on: "some identity occurred twice" leaves a reader to re-derive which two from a
/// population of hundreds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollidingOccurrences
{
    /// The run whose findings collided.
    pub run: RunId,
    /// Every colliding pair, each naming both findings that reached one identity.
    pub collisions: Vec<OccurrenceCollision>,
}

/// One finding occurrence present in both runs [`Compare_Gate_Runs`] compared, but whose gate
/// treatment or the reason producing it differs between them.
///
/// Identified across the two runs by [`FindingOccurrenceId`], so two violations one rule makes
/// about one subject are two changes here rather than one. `rule`, `subject` and `subject_name`
/// are carried for a reader; they no longer identify the entry on their own, and `locations` is
/// what tells two entries at one subject apart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DispositionChange
{
    pub rule: RuleId,
    pub subject: SubjectId,
    pub subject_name: String,
    /// Where this occurrence is, carried so two changes at one subject are distinguishable.
    ///
    /// Geometry, not identity: the occurrence identity stays internal to this module, and
    /// `Finding::locations`' own rule -- a location is never what a claim is attributed to --
    /// is unaffected by showing a reader where to look.
    pub locations: Vec<String>,
    pub before: FindingDisposition,
    pub after: FindingDisposition,
    /// Why the finding was in `before`'s bucket, when a disposition named it.
    ///
    /// Carried so a change *within* one bucket is reportable. A `FalsePositiveDisposition`
    /// becoming a `FormalRiskAcceptance` leaves `before` and `after` equal and is an opposite
    /// engineering claim; a lapsed waiver leaves the suppressed bucket entirely, and a reader
    /// needs to see that a tolerance came due rather than that a violation appeared.
    pub before_reason: Option<SuppressionReason>,
    /// Why the finding is in `after`'s bucket, when a disposition names it.
    pub after_reason: Option<SuppressionReason>,
}

/// What changed between `baseline` and `candidate`'s own [`GateFindings`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GateCompareResult
{
    /// The run compared against.
    pub baseline: RunId,
    /// The run being compared.
    pub candidate: RunId,
    /// Present in `candidate`, absent from `baseline` -- by occurrence identity.
    pub added: Vec<Finding>,
    /// Present in `baseline`, absent from `candidate`.
    pub removed: Vec<Finding>,
    /// Present in both, but which bucket it fell into changed.
    pub changed: Vec<DispositionChange>,
}

/// Compares `baseline` and `candidate`'s own reduced findings, identifying one occurrence
/// with another across the two runs by [`FindingOccurrenceId`].
///
/// # Why occurrence identity, and why only here
///
/// This used to key on (`rule`, `subject`), matching what [`crate::SuppressionPolicy`] and
/// [`crate::BaselinePolicy`] address a finding by. That is the right identity for a *tolerance*,
/// which must outlive the revision it was written in, and the wrong one for a *comparison*: a
/// rule emits one finding per occurrence while the subject is the file, so five violations in
/// one file collapsed to one and four were dropped by a map insert. Measured on this
/// repository's own tree before the fix: 253 findings, 150 distinct pairs, 103 discarded, the
/// worst single key holding twenty-four.
///
/// So comparison moved to occurrence scope and **suppression and baseline deliberately did
/// not**. Giving either of them occurrence scope would expire every tolerated entry on the next
/// reformatting commit, which is a policy change and not a consequence of this one;
/// [`Reason_For`] is where that division is enforced, and a test holds it.
///
/// # Errors
///
/// [`CollidingOccurrences`] when either run's findings do not yield one identity each. Checked
/// per run *before* any index exists, so the silent-overwrite mechanism this function was fixed
/// to remove is not reachable on a path that has supposedly been validated.
pub fn Compare_Gate_Runs(
    baseline: &GateRunResult,
    candidate: &GateRunResult,
) -> Result<GateCompareResult, CollidingOccurrences>
{
    let before = Indexed(&baseline.findings, baseline.run)?;
    let after = Indexed(&candidate.findings, candidate.run)?;

    let mut added: Vec<Finding> = after
        .iter()
        .filter(|(key, _)| return !before.contains_key(*key))
        .map(|(_, (_, finding))| return finding.clone())
        .collect();
    let mut removed: Vec<Finding> = before
        .iter()
        .filter(|(key, _)| return !after.contains_key(*key))
        .map(|(_, (_, finding))| return finding.clone())
        .collect();
    let mut changed: Vec<DispositionChange> = after
        .iter()
        .filter_map(|(occurrence, (after_disposition, after_finding))| {
            let (before_disposition, before_finding) = before.get(occurrence)?;
            let before_reason = Reason_For(&baseline.findings, before_finding);
            let after_reason = Reason_For(&candidate.findings, after_finding);

            // Two states are the same state only when the treatment and the reason
            // producing it are both the same. Comparing the bucket alone made a
            // disposition change within `Suppressed` report as nothing at all.
            if before_disposition == after_disposition && before_reason == after_reason
            {
                return None;
            }

            return Some(DispositionChange {
                rule: after_finding.rule.clone(),
                subject: after_finding.subject,
                subject_name: after_finding.subject_name.clone(),
                locations: after_finding.locations.clone(),
                before: *before_disposition,
                after: *after_disposition,
                before_reason,
                after_reason,
            });
        })
        .collect();

    // Sorted by what a reader scans, with `locations` breaking the tie that occurrence scope
    // introduced: (rule, subject_name) alone stopped being unique the moment one subject could
    // contribute more than one entry, and a comparison whose output order is unstable between
    // runs over identical input is a diff nobody can diff.
    added.sort_by(|left, right| return Reading_Order(left).cmp(&Reading_Order(right)));
    removed.sort_by(|left, right| return Reading_Order(left).cmp(&Reading_Order(right)));
    changed.sort_by(|left, right| {
        return (&left.rule, &left.subject_name, &left.locations).cmp(&(&right.rule, &right.subject_name, &right.locations));
    });

    return Ok(GateCompareResult { baseline: baseline.run, candidate: candidate.run, added, removed, changed });
}

/// What a finding sorts by in a rendered comparison.
fn Reading_Order(finding: &Finding) -> (&RuleId, &String, &Vec<String>)
{
    return (&finding.rule, &finding.subject_name, &finding.locations);
}

/// The suppression reason `findings` recorded for `finding`, matched by (`rule`, `subject`).
///
/// **Deliberately not by occurrence.** `GateFindings::suppression_reasons` is written by policy
/// evaluation, and a suppression addresses a rule and a subject so that it survives edits around
/// the finding it tolerates. Looking it up by occurrence identity would silently narrow every
/// existing suppression to one line of one revision -- the policy change this increment does not
/// make and must not make by accident. Written as its own function so the division has a place
/// to be stated and a place for a test to point at.
fn Reason_For(findings: &GateFindings, finding: &Finding) -> Option<SuppressionReason>
{
    return findings.suppression_reasons.get(&(finding.rule.clone(), finding.subject)).copied();
}

/// Every finding in `findings`, paired with the bucket it fell into.
///
/// Separate from [`Indexed`] because the validation below has to see the whole population
/// *before* anything is keyed by identity. A finding matched by more than one bucket cannot
/// occur, the same disjointness [`GateFindings`]'s own field docs already state.
fn Population_Of(findings: &GateFindings) -> Vec<(FindingDisposition, &Finding)>
{
    let mut population = Vec::new();

    for (bucket, disposition) in [
        (&findings.blocking_findings, FindingDisposition::Blocking),
        (&findings.calibrated_findings, FindingDisposition::Calibrated),
        (&findings.suppressed_findings, FindingDisposition::Suppressed),
        (&findings.baselined_findings, FindingDisposition::Baselined),
    ]
    {
        for finding in bucket
        {
            population.push((disposition, finding));
        }
    }

    return population;
}

/// `findings`, keyed by occurrence identity against which bucket each one fell into.
///
/// # Errors
///
/// [`CollidingOccurrences`] when two findings in `run` reach one identity.
///
/// # Why the check is before the map and not inside it
///
/// Because a check inside the loop would be reading `BTreeMap::insert`'s displaced value, and
/// the whole defect this function was rewritten to remove is that nobody reads it. Validating
/// the population first means the map is built only over input already known to be injective,
/// and there is no branch on which a displaced value could be produced and dropped. The order is
/// the guarantee; a comment asking the next reader to preserve it would not be.
fn Indexed(
    findings: &GateFindings,
    run: RunId,
) -> Result<BTreeMap<FindingOccurrenceId, (FindingDisposition, Finding)>, CollidingOccurrences>
{
    let population = Population_Of(findings);
    let subjects: Vec<&Finding> = population.iter().map(|(_, finding)| return *finding).collect();
    let collisions = Occurrence_Collisions_In(&subjects);

    if !collisions.is_empty()
    {
        return Err(CollidingOccurrences { run, collisions });
    }

    let mut indexed = BTreeMap::new();

    for (disposition, finding) in population
    {
        indexed.insert(FindingOccurrenceId::Of(finding), (disposition, finding.clone()));
    }

    return Ok(indexed);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{GateCommand, GateEnvironment, Run_Gate};
    use nomos_contracts::{Digest128, GateCategory};
    use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProcessLauncher};
    use nomos_rules::SourceFile;
    use nomos_workspace::BuildVariant;

    fn Test_Variant() -> BuildVariant
    {
        return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
    }

    pub(super) fn Run_Id_Of(fill: u8) -> RunId
    {
        return RunId::From_Digest(Digest128::From_Bytes([fill; Digest128::BYTE_LENGTH]));
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        return SourceFile::New(path, nomos_model::Subject_Of_Path(path), text.to_owned());
    }

    /// Runs the gate over `sources`, scoped to `Check_Completeness_Mirrors` alone -- the
    /// one rule every fixture below is written against, so no other registered rule's own
    /// opinion of a hand-written fixture snippet can add an unexpected finding neither
    /// test asked about.
    fn Run_Over(sources: Vec<SourceFile>, run: RunId) -> GateRunResult
    {
        use crate::RuleSelector;

        let command = GateCommand {
            root: std::path::PathBuf::from("."),
            rules: RuleSelector { include: vec![RuleId::New(nomos_rules::COMPLETENESS_MIRROR)] },
            ..GateCommand::default()
        };

        return Run_Gate(Some(sources), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, now: nomos_platform::Timestamp::From_Unix_Seconds(0) }, &command, run);
    }

    /// Two runs over a tree that gained one real blocking finding between them: the
    /// second names one real addition and nothing else -- the population
    /// `P40-GATE-COMPARE-VERB`'s own done_when asks for, over two runs that actually
    /// differ rather than two identical ones.
    #[test]
    fn Test_Compare_Gate_Runs_Should_Name_A_Finding_Added_Between_Two_Real_Runs()
    {
        let clean = vec![Source("a.rs", "pub const THINGS: &[&str] = &[\"a\"];\n")];
        let with_violation = vec![Source("a.rs", "/// Mirrored by `Test_Compare_Ghost`.\npub const THINGS: &[&str] = &[\"a\"];\n")];

        let baseline = Run_Over(clean, Run_Id_Of(1));
        let candidate = Run_Over(with_violation, Run_Id_Of(2));

        let compared = Compare_Gate_Runs(&baseline, &candidate)
            .expect("these fixtures hold distinct occurrences; a collision here is the guard firing, not the case under test");

        assert_eq!(compared.baseline, Run_Id_Of(1));
        assert_eq!(compared.candidate, Run_Id_Of(2));
        assert_eq!(compared.added.len(), 1, "{:?}", compared.added);
        assert_eq!(compared.added.first().expect("asserted len 1 above").rule, RuleId::New(nomos_rules::COMPLETENESS_MIRROR));
        assert!(compared.removed.is_empty(), "{:?}", compared.removed);
        assert!(compared.changed.is_empty(), "{:?}", compared.changed);
    }

    /// The inverse direction: a finding present in the baseline and fixed by the
    /// candidate is reported removed, not silently dropped.
    #[test]
    fn Test_Compare_Gate_Runs_Should_Name_A_Finding_Removed_Between_Two_Real_Runs()
    {
        let with_violation = vec![Source("a.rs", "/// Mirrored by `Test_Compare_Ghost_2`.\npub const THINGS: &[&str] = &[\"a\"];\n")];
        let fixed = vec![Source("a.rs", "pub const THINGS: &[&str] = &[\"a\"];\n")];

        let baseline = Run_Over(with_violation, Run_Id_Of(3));
        let candidate = Run_Over(fixed, Run_Id_Of(4));

        let compared = Compare_Gate_Runs(&baseline, &candidate)
            .expect("these fixtures hold distinct occurrences; a collision here is the guard firing, not the case under test");

        assert!(compared.added.is_empty(), "{:?}", compared.added);
        assert_eq!(compared.removed.len(), 1, "{:?}", compared.removed);
        assert!(compared.changed.is_empty(), "{:?}", compared.changed);
    }

    /// Two runs over the identical tree name nothing at all -- the vacuity guard every
    /// diff needs: a comparison that always finds *something* has stopped comparing.
    #[test]
    fn Test_Compare_Gate_Runs_Should_Name_Nothing_Between_Two_Identical_Runs()
    {
        let sources = vec![Source("a.rs", "pub fn Something() -> u32\n{\n    return 1;\n}\n")];

        let baseline = Run_Over(sources.clone(), Run_Id_Of(5));
        let candidate = Run_Over(sources, Run_Id_Of(6));

        let compared = Compare_Gate_Runs(&baseline, &candidate)
            .expect("these fixtures hold distinct occurrences; a collision here is the guard firing, not the case under test");

        assert!(compared.added.is_empty(), "{:?}", compared.added);
        assert!(compared.removed.is_empty(), "{:?}", compared.removed);
        assert!(compared.changed.is_empty(), "{:?}", compared.changed);
    }

    /// The same finding, present in both runs but reported through a different bucket,
    /// is a disposition change -- not an addition and a removal that happen to cancel
    /// out. Built directly against [`GateFindings`] rather than through a real policy
    /// change, since no real caller constructs a non-default `AdoptionPolicy`,
    /// `SuppressionPolicy` or `BaselinePolicy` yet (every one of their own module docs
    /// says so) and this test's job is [`Compare_Gate_Runs`]'s own bucket comparison, not
    /// a second proof that a policy this crate already tests elsewhere matches a finding.
    #[test]
    fn Test_Compare_Gate_Runs_Should_Name_A_Disposition_Change_For_The_Same_Finding_In_A_Different_Bucket()
    {
        let finding = Finding {
            rule: RuleId::New(nomos_rules::COMPLETENESS_MIRROR),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([9; Digest128::BYTE_LENGTH])),
            subject_name: "a.rs".to_owned(),
            applicability: nomos_contracts::Applicability::Supported,
            evidence: nomos_contracts::EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: "a real finding, moved to a different bucket between two runs".to_owned(),
            locations: vec!["a.rs".to_owned()],
        };

        let baseline = Result_With(Run_Id_Of(7), GateFindings { blocking_findings: vec![finding.clone()], calibrated_findings: vec![], suppressed_findings: vec![], baselined_findings: vec![], suppression_reasons: Default::default() });
        let candidate = Result_With(Run_Id_Of(8), GateFindings { blocking_findings: vec![], calibrated_findings: vec![], suppressed_findings: vec![finding], baselined_findings: vec![], suppression_reasons: Default::default() });

        let compared = Compare_Gate_Runs(&baseline, &candidate)
            .expect("these fixtures hold distinct occurrences; a collision here is the guard firing, not the case under test");

        assert!(compared.added.is_empty(), "{:?}", compared.added);
        assert!(compared.removed.is_empty(), "{:?}", compared.removed);
        assert_eq!(compared.changed.len(), 1, "{:?}", compared.changed);
        let change = compared.changed.first().expect("asserted len 1 above");
        assert_eq!(change.before, FindingDisposition::Blocking);
        assert_eq!(change.after, FindingDisposition::Suppressed);
    }

    /// The provenance every [`Result_With`] result carries, so that two of them differ in
    /// their findings and in nothing else.
    fn Identical_Provenance() -> crate::GateRunProvenance
    {
        let digest = Digest128::From_Bytes([0; Digest128::BYTE_LENGTH]);

        return crate::GateRunProvenance {
            source: digest,
            policy: digest,
            selection: digest,
            instrument: digest,
            at: nomos_platform::Timestamp::From_Unix_Seconds(0),
        };
    }

    pub(super) fn Result_With(run: RunId, findings: GateFindings) -> GateRunResult
    {
        return GateRunResult {
            unmatched_policy: Vec::new(),
            run,
            root: std::path::PathBuf::from("."),
            check_outcome: nomos_check_orchestration::CheckOutcome::NoSource,
            findings,
            disposition: crate::GateRunOutcome::Indeterminate,
            no_verdict: None,
            // One fixed provenance for every result this helper builds, so two of them are
            // judged alike by construction and a test about findings stays a test about
            // findings. A test that wants its two sides judged differently says so itself.
            provenance: Some(Identical_Provenance()),
        };
    }
}

#[cfg(test)]
mod reason_tests
{
    use super::tests::{Result_With, Run_Id_Of};
    use crate::{
        Compare_Gate_Runs, FindingDisposition, GateFindings, SuppressionDisposition, SuppressionReason, SuppressionStatus,
    };
    use nomos_contracts::{Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};
    use std::collections::BTreeMap;

    fn Finding_Here() -> Finding
    {
        return Finding {
            rule: RuleId::New("naming-convention"),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([7; Digest128::BYTE_LENGTH])),
            subject_name: "src/lib.rs".to_string(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: "a name".to_string(),
            locations: Vec::new(),
        };
    }

    fn Reason(disposition: SuppressionDisposition, status: SuppressionStatus) -> SuppressionReason
    {
        return SuppressionReason { disposition, status };
    }

    /// `findings` with one finding in `bucket`, and `reason` recorded against it.
    fn Findings_With(bucket: FindingDisposition, reason: Option<SuppressionReason>) -> GateFindings
    {
        let finding = Finding_Here();
        let mut reasons = BTreeMap::new();

        if let Some(reason) = reason
        {
            reasons.insert((finding.rule.clone(), finding.subject), reason);
        }

        let mut findings = GateFindings {
            blocking_findings: Vec::new(),
            calibrated_findings: Vec::new(),
            suppressed_findings: Vec::new(),
            baselined_findings: Vec::new(),
            suppression_reasons: reasons,
        };

        match bucket
        {
            FindingDisposition::Blocking => findings.blocking_findings.push(finding),
            FindingDisposition::Calibrated => findings.calibrated_findings.push(finding),
            FindingDisposition::Suppressed => findings.suppressed_findings.push(finding),
            FindingDisposition::Baselined => findings.baselined_findings.push(finding),
        }

        return findings;
    }

    fn Compared(
        before: (FindingDisposition, Option<SuppressionReason>),
        after: (FindingDisposition, Option<SuppressionReason>),
    ) -> super::GateCompareResult
    {
        return Compare_Gate_Runs(
            &Result_With(Run_Id_Of(1), Findings_With(before.0, before.1)),
            &Result_With(Run_Id_Of(2), Findings_With(after.0, after.1)),
        )
        .expect("these fixtures hold distinct occurrences; a collision here is the guard firing, not the case under test");
    }

    /// A reason changing inside one bucket is a change.
    ///
    /// The transition this whole increment exists for. `FalsePositiveDisposition` says the
    /// rule was wrong here; `FormalRiskAcceptance` says somebody owns the risk. The bucket is
    /// `Suppressed` on both sides and the engineering claim is opposite, and before this the
    /// comparison reported nothing at all.
    #[test]
    fn Test_A_Disposition_Change_Within_Suppressed_Should_Be_Reported()
    {
        let result = Compared(
            (FindingDisposition::Suppressed, Some(Reason(SuppressionDisposition::FalsePositiveDisposition, SuppressionStatus::Active))),
            (FindingDisposition::Suppressed, Some(Reason(SuppressionDisposition::FormalRiskAcceptance, SuppressionStatus::Active))),
        );

        let change = result.changed.first().expect("a reason change is a change");

        assert_eq!(change.before, FindingDisposition::Suppressed, "the bucket did not move and must not appear to");
        assert_eq!(change.after, FindingDisposition::Suppressed);
        assert_eq!(change.before_reason.expect("a reason").disposition, SuppressionDisposition::FalsePositiveDisposition);
        assert_eq!(change.after_reason.expect("a reason").disposition, SuppressionDisposition::FormalRiskAcceptance);
    }

    /// A waiver lapsing is a change, and the reason says so.
    ///
    /// Under `P103` an expired waiver does not suppress, so the finding leaves the bucket.
    /// Without the recorded reason a reader would see only `Suppressed -> Blocking` and could
    /// not tell a tolerance coming due from a suppression being withdrawn by hand.
    #[test]
    fn Test_A_Waiver_Lapsing_Should_Be_Reported_With_Its_Reason()
    {
        let result = Compared(
            (FindingDisposition::Suppressed, Some(Reason(SuppressionDisposition::TemporaryWaiver, SuppressionStatus::Active))),
            (FindingDisposition::Blocking, Some(Reason(SuppressionDisposition::TemporaryWaiver, SuppressionStatus::Expired))),
        );

        let change = result.changed.first().expect("a lapsed waiver is a change");

        assert_eq!(change.before, FindingDisposition::Suppressed);
        assert_eq!(change.after, FindingDisposition::Blocking);
        assert_eq!(change.after_reason.expect("a reason").status, SuppressionStatus::Expired);
    }

    /// A bucket change with no suppression on either side still reports.
    #[test]
    fn Test_Baselined_Becoming_Blocking_Should_Be_Reported()
    {
        let result = Compared((FindingDisposition::Baselined, None), (FindingDisposition::Blocking, None));
        let change = result.changed.first().expect("a bucket change is a change");

        assert_eq!((change.before, change.after), (FindingDisposition::Baselined, FindingDisposition::Blocking));
        assert!(change.before_reason.is_none() && change.after_reason.is_none());
    }

    /// Nothing moving reports nothing.
    ///
    /// The converse control. Without it a comparison that reported every finding every time
    /// would satisfy all three above and tell a reader nothing, which is the failure mode the
    /// added/removed/changed split exists to avoid.
    #[test]
    fn Test_An_Unchanged_Bucket_And_Reason_Should_Report_No_Change()
    {
        let reason = Some(Reason(SuppressionDisposition::InlineSuppression, SuppressionStatus::Active));
        let result = Compared((FindingDisposition::Suppressed, reason), (FindingDisposition::Suppressed, reason));

        assert!(
            result.changed.is_empty(),
            "a finding whose bucket and reason both held still reported as changed: {:?}",
            result.changed
        );
        assert!(result.added.is_empty() && result.removed.is_empty());
    }
}

#[cfg(test)]
mod occurrence_tests
{
    //! The four behaviours occurrence scope changed, the refusal it introduced, and the two
    //! addressings it deliberately left alone.
    //!
    //! Built against [`GateFindings`] directly rather than through a real policy change, the same
    //! reason `reason_tests` states: what is under test is this module's own indexing, not a
    //! second proof that a policy matches a finding.

    use super::tests::{Result_With, Run_Id_Of};
    use crate::{
        Compare_Gate_Runs, FindingDisposition, GateFindings, SuppressionDisposition, SuppressionReason, SuppressionStatus,
    };
    use nomos_contracts::{Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};
    use nomos_model::{FindingOccurrenceId, Occurrence_Collisions_In};
    use std::collections::BTreeMap;

    const RULE: &str = "nesting-depth";

    fn Subject() -> SubjectId
    {
        return SubjectId::From_Digest(Digest128::From_Bytes([5; Digest128::BYTE_LENGTH]));
    }

    /// One occurrence of `RULE` in `src/deep.rs`, at `line`.
    ///
    /// Every field except the location is identical across occurrences, which is exactly the
    /// population that used to collapse: one rule, one subject, several places. `subject_name` is
    /// the file rather than the line on purpose -- it is the coarse spelling most rules use, and
    /// a fixture that varied it would hide the collapse behind a field the identity does not read.
    fn Occurrence_At(line: u32) -> Finding
    {
        return Finding {
            rule: RuleId::New(RULE),
            subject: Subject(),
            subject_name: "src/deep.rs".to_owned(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: "nested too deeply".to_owned(),
            locations: vec![format!("src/deep.rs:{line}")],
        };
    }

    /// One occurrence per entry of `lines`, all in the blocking bucket.
    fn Blocking_At(lines: &[u32]) -> GateFindings
    {
        return GateFindings {
            blocking_findings: lines.iter().map(|line| return Occurrence_At(*line)).collect(),
            calibrated_findings: Vec::new(),
            suppressed_findings: Vec::new(),
            baselined_findings: Vec::new(),
            suppression_reasons: BTreeMap::new(),
        };
    }

    /// Some of a subject's occurrences fixed, the rest still there.
    ///
    /// Before occurrence scope this reported *nothing at all*: one finding before, one finding
    /// after, same key, no difference. Three of five violations disappearing from a file is the
    /// single most ordinary thing a comparison is asked about.
    #[test]
    fn Test_Fixing_Some_Occurrences_In_One_Subject_Should_Report_Removals()
    {
        let baseline = Result_With(Run_Id_Of(1), Blocking_At(&[10, 20, 30, 40, 50]));
        let candidate = Result_With(Run_Id_Of(2), Blocking_At(&[10, 20]));

        let compared = Compare_Gate_Runs(&baseline, &candidate).expect("distinct occurrences");

        assert_eq!(compared.removed.len(), 3, "three of five were fixed: {:?}", compared.removed);
        assert!(compared.added.is_empty(), "{:?}", compared.added);
        assert!(compared.changed.is_empty(), "{:?}", compared.changed);
    }

    /// The same defect spreading through one file.
    ///
    /// The other direction of the same silence, and the dangerous one: a file going from one
    /// violation to five reported no change, so a regression inside an already-failing subject
    /// was invisible.
    #[test]
    fn Test_Occurrences_Multiplying_In_One_Subject_Should_Report_Additions()
    {
        let baseline = Result_With(Run_Id_Of(1), Blocking_At(&[10]));
        let candidate = Result_With(Run_Id_Of(2), Blocking_At(&[10, 20, 30, 40, 50]));

        let compared = Compare_Gate_Runs(&baseline, &candidate).expect("distinct occurrences");

        assert_eq!(compared.added.len(), 4, "four more appeared: {:?}", compared.added);
        assert!(compared.removed.is_empty(), "{:?}", compared.removed);
        assert!(compared.changed.is_empty(), "{:?}", compared.changed);
    }

    /// The vacuity guard, at the size that matters.
    ///
    /// A comparison that finds something whenever a subject holds several occurrences would make
    /// every one of the assertions above pass while being useless.
    #[test]
    fn Test_An_Unchanged_Multi_Occurrence_Subject_Should_Report_No_Change()
    {
        let baseline = Result_With(Run_Id_Of(1), Blocking_At(&[10, 20, 30]));
        let candidate = Result_With(Run_Id_Of(2), Blocking_At(&[10, 20, 30]));

        let compared = Compare_Gate_Runs(&baseline, &candidate).expect("distinct occurrences");

        assert!(compared.added.is_empty(), "{:?}", compared.added);
        assert!(compared.removed.is_empty(), "{:?}", compared.removed);
        assert!(compared.changed.is_empty(), "{:?}", compared.changed);
    }

    /// The population that already worked still works.
    ///
    /// One finding at one subject was never the broken case, and a fix that changed its answer
    /// would have traded one silent wrong report for another.
    #[test]
    fn Test_A_Single_Finding_Subject_Should_Behave_As_It_Did()
    {
        let baseline = Result_With(Run_Id_Of(1), Blocking_At(&[10]));
        let candidate = Result_With(Run_Id_Of(2), Blocking_At(&[]));

        let compared = Compare_Gate_Runs(&baseline, &candidate).expect("distinct occurrences");

        assert_eq!(compared.removed.len(), 1, "{:?}", compared.removed);
        assert!(compared.added.is_empty(), "{:?}", compared.added);
    }

    /// A run holding two findings that cannot be told apart is refused, and both are named.
    ///
    /// The negative control for the whole mechanism. These two differ only in gate category,
    /// which the occurrence identity excludes by construction, so they reach one identity -- the
    /// exact limit `FindingOccurrenceId`'s own documentation states. Without this assertion the
    /// validation below is only known to pass on populations that already satisfy it, which is
    /// indistinguishable from a validation that cannot fail.
    #[test]
    fn Test_A_Run_Whose_Findings_Collide_Should_Refuse_And_Name_Both()
    {
        let blocking = Occurrence_At(10);
        let advisory = Finding { gate: GateCategory::Advisory, ..Occurrence_At(10) };

        let colliding = GateFindings {
            blocking_findings: vec![blocking.clone()],
            calibrated_findings: vec![advisory.clone()],
            suppressed_findings: Vec::new(),
            baselined_findings: Vec::new(),
            suppression_reasons: BTreeMap::new(),
        };

        let baseline = Result_With(Run_Id_Of(1), colliding);
        let candidate = Result_With(Run_Id_Of(2), Blocking_At(&[10]));

        let refused = Compare_Gate_Runs(&baseline, &candidate).expect_err("two findings share one identity");

        assert_eq!(refused.run, Run_Id_Of(1), "the refusal must say which side could not be indexed");
        assert_eq!(refused.collisions.len(), 1, "{:?}", refused.collisions);

        let collision = refused.collisions.first().expect("asserted len 1 above");

        assert_eq!(collision.first, blocking, "the collision names the finding that got there first");
        assert_eq!(collision.second, advisory, "and the one that collided with it");
    }

    /// A clean run is not refused.
    ///
    /// The other direction of the refusal: a validation that rejected everything would satisfy
    /// the test above and destroy the verb.
    #[test]
    fn Test_A_Run_Whose_Findings_Are_Distinct_Should_Not_Be_Refused()
    {
        let baseline = Result_With(Run_Id_Of(1), Blocking_At(&[10, 20, 30]));
        let candidate = Result_With(Run_Id_Of(2), Blocking_At(&[10, 20, 30]));

        assert!(Compare_Gate_Runs(&baseline, &candidate).is_ok());
    }

    /// One suppression entry still covers every occurrence at its subject.
    ///
    /// **The invariant this increment must not break.** Comparison moved to occurrence scope;
    /// suppression addressing did not. `suppression_reasons` is keyed by (`rule`, `subject`), so
    /// a single recorded reason has to reach both occurrences below. If [`Reason_For`] were
    /// looking up by occurrence identity instead, one of these two changes would carry no reason
    /// and a repository's existing suppressions would have silently narrowed to one line of one
    /// revision.
    ///
    /// [`Reason_For`]: super::Reason_For
    #[test]
    fn Test_A_Suppression_Reason_Should_Reach_Every_Occurrence_At_Its_Subject()
    {
        let reason = SuppressionReason {
            disposition: SuppressionDisposition::FormalRiskAcceptance,
            status: SuppressionStatus::Active,
        };

        let mut reasons = BTreeMap::new();
        reasons.insert((RuleId::New(RULE), Subject()), reason);

        let suppressed = GateFindings {
            blocking_findings: Vec::new(),
            calibrated_findings: Vec::new(),
            suppressed_findings: vec![Occurrence_At(10), Occurrence_At(20)],
            baselined_findings: Vec::new(),
            suppression_reasons: reasons,
        };

        let baseline = Result_With(Run_Id_Of(1), Blocking_At(&[10, 20]));
        let candidate = Result_With(Run_Id_Of(2), suppressed);

        let compared = Compare_Gate_Runs(&baseline, &candidate).expect("distinct occurrences");

        assert_eq!(compared.changed.len(), 2, "both occurrences moved bucket: {:?}", compared.changed);

        for change in &compared.changed
        {
            assert_eq!(change.before, FindingDisposition::Blocking);
            assert_eq!(change.after, FindingDisposition::Suppressed);
            assert_eq!(
                change.after_reason,
                Some(reason),
                "one suppression entry addresses a rule and a subject, so it must reach every \
                 occurrence there -- narrowing it to one occurrence is a policy change this \
                 increment does not make"
            );
        }
    }

    /// Two changes at one subject are told apart by their geometry.
    ///
    /// Occurrence scope made (`rule`, `subject_name`) non-unique in `changed`, and the rendered
    /// line is built from those two. Without `locations` a reader would see the same line twice
    /// and have no way to know it described two different places.
    #[test]
    fn Test_Two_Changes_At_One_Subject_Should_Be_Distinguishable()
    {
        let baseline = Result_With(Run_Id_Of(1), Blocking_At(&[10, 20]));
        let candidate = Result_With(
            Run_Id_Of(2),
            GateFindings {
                blocking_findings: Vec::new(),
                calibrated_findings: vec![Occurrence_At(10), Occurrence_At(20)],
                suppressed_findings: Vec::new(),
                baselined_findings: Vec::new(),
                suppression_reasons: BTreeMap::new(),
            },
        );

        let compared = Compare_Gate_Runs(&baseline, &candidate).expect("distinct occurrences");

        let locations: Vec<&Vec<String>> = compared.changed.iter().map(|change| return &change.locations).collect();

        assert_eq!(compared.changed.len(), 2, "{:?}", compared.changed);
        assert_ne!(locations.first(), locations.last(), "two changes at one subject must not render identically");
    }

    /// Every finding a real run of this repository's own rules produces gets its own identity.
    ///
    /// # Why a real population and not a bigger fixture
    ///
    /// Because the property is about the findings this workspace's rules actually emit, and a
    /// fixture proves only that the author of the fixture believed it. The measured shape before
    /// this increment was 253 findings reduced to 150 keys; the claim afterwards is that the
    /// reduction is gone, and only real findings can carry that claim.
    ///
    /// # What is asserted, and what is deliberately not
    ///
    /// The floor is asserted, not the exact count. Peers edit this tree while tests run, so an
    /// exact number would be a flake rather than a stronger claim; a floor plus zero collisions
    /// is the property, and the floor is what stops an empty walk from passing as a clean result
    /// -- the vacuity failure this repository's own records name repeatedly.
    #[test]
    fn Test_Every_Finding_Of_A_Real_Run_Should_Get_Its_Own_Identity()
    {
        let findings = Real_Findings();

        assert!(
            findings.len() >= 50,
            "only {} finding(s) came back from a real run, which is too few to be evidence of \
             anything. A clean result over an empty population is the lie this assertion exists \
             to refuse.",
            findings.len()
        );

        let borrowed: Vec<&Finding> = findings.iter().collect();
        let collisions = Occurrence_Collisions_In(&borrowed);
        let identities: std::collections::BTreeSet<FindingOccurrenceId> =
            findings.iter().map(FindingOccurrenceId::Of).collect();

        assert!(collisions.is_empty(), "{} colliding pair(s): {collisions:?}", collisions.len());
        assert_eq!(
            identities.len(),
            findings.len(),
            "the number of findings entering a comparison must equal the number emitted; \
             {} findings produced {} identities",
            findings.len(),
            identities.len()
        );
    }

    /// Every finding a real run over this repository's own source produces.
    ///
    /// # Why the walk is here rather than passed in
    ///
    /// Because this crate does not walk. [`Run_Gate`] takes an already-walked source set and
    /// reads `None` as *unreadable*, not as *go and look* -- the division that keeps a walk in
    /// the composition root. So a test that wants a real population has to do what `nomos-cli`
    /// and `nomos-api` do, and collect one.
    ///
    /// The whole of `crates/` is collected rather than one crate's worth: a cross-file rule
    /// handed a truncated world answers a different question and labels it the same, which
    /// `OD-GATE-025` records and which this repository has already been bitten by.
    fn Real_Findings() -> Vec<Finding>
    {
        use crate::{GateCommand, GateEnvironment, Run_Gate};
        use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProcessLauncher};
        use nomos_workspace::BuildVariant;

        // The workspace root, not `crates/`: the check beneath a run resolves a workspace from
        // the root it is given, and a directory with no manifest above it comes back
        // `Unreadable` -- which is an empty finding list, which is a vacuous pass.
        // Climbed with `parent` rather than joined with `..`, so the spellings handed to the
        // ingest are the ones a real caller would pass. A root carrying `..` segments reaches
        // the subject derivation as a different spelling of the same file, and this repository
        // has already recorded what a path spelling mismatch does to matching.
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("the manifest directory is three levels below the workspace root")
            .to_path_buf();
        let sources = Rust_Under(&root, &root);

        assert!(!sources.is_empty(), "no source collected under {}", root.display());

        let command = GateCommand { root: root.clone(), ..GateCommand::default() };
        let environment = GateEnvironment {
            variant: BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>()),
            launcher: &StdProcessLauncher,
            filesystem: &StdFileSystem,
            environment: &StdEnvironment,
            now: nomos_platform::Timestamp::From_Unix_Seconds(0),
        };

        let collected = sources.len();
        let result = Run_Gate(Some(sources), environment, &command, Run_Id_Of(9));

        // A run that never reached a judgment would report an empty finding list, and an empty
        // finding list trivially has no colliding identities. That is the vacuous pass this
        // whole test exists to refuse, and the outcome is the only thing that distinguishes it
        // from a real clean result.
        assert!(
            matches!(result.check_outcome, nomos_check_orchestration::CheckOutcome::Judged { .. }),
            "{collected} source(s) collected under {} and the run still did not judge: {:?}",
            root.display(),
            result.check_outcome
        );

        // The judged population, not the four reduced buckets. The gate composes four rules and
        // the check beneath it runs the whole registry, so the buckets hold single digits while
        // the run *emits* hundreds -- and "as many identities as findings emitted" is a claim
        // about what the analysis produced, not about the slice this crate's own policy kept.
        // The 253-to-150 collapse this increment removes was measured over exactly this set.
        let nomos_check_orchestration::CheckOutcome::Judged { findings, .. } = result.check_outcome
        else
        {
            panic!("asserted Judged above");
        };

        return findings;
    }

    /// Every `.rs` file under `directory`, as sources a run can judge, spelled relative to
    /// `root`.
    ///
    /// # Why relative and forward-slashed
    ///
    /// Because that is what a source path is in this system, and neither half is cosmetic. A
    /// `Finding`'s locations are documented repo-relative with forward slashes, and the
    /// workspace ingest validates every path it is handed: an absolute Windows spelling carries
    /// a drive-letter segment the ingest will not name, and the whole change set is refused
    /// rather than half-applied. A test that passed absolute paths got `CheckOutcome::Unreadable`
    /// and an empty finding list -- which is to say, a vacuous pass, if the assertion above had
    /// not been there to catch it.
    fn Rust_Under(directory: &std::path::Path, root: &std::path::Path) -> Vec<nomos_rules::SourceFile>
    {
        let mut sources = Vec::new();
        let mut pending = vec![directory.to_path_buf()];

        while let Some(next) = pending.pop()
        {
            let (directories, files) = Children_Of(&next);
            pending.extend(directories);

            for path in files
            {
                if let Some(source) = Source_Of(&path, root)
                {
                    sources.push(source);
                }
            }
        }

        return sources;
    }

    /// The subdirectories of `directory` worth walking, and the files directly inside it.
    ///
    /// Split from the walk so neither half nests: sorting entries into two kinds and deciding
    /// what each kind is are two jobs, and doing both in one loop is what `nesting-depth`
    /// reported here. An unreadable directory yields nothing rather than failing the walk --
    /// a tree this test cannot fully read still answers the question it asks, and the floor
    /// assertion above is what stops a half-read tree passing as a clean one.
    fn Children_Of(directory: &std::path::Path) -> (Vec<std::path::PathBuf>, Vec<std::path::PathBuf>)
    {
        let mut directories = Vec::new();
        let mut files = Vec::new();

        let Ok(entries) = std::fs::read_dir(directory)
        else
        {
            return (directories, files);
        };

        for entry in entries.flatten()
        {
            let path = entry.path();

            if !path.is_dir()
            {
                files.push(path);
                continue;
            }

            // `target/` holds the compiled world, including every dependency's own source,
            // which is neither this repository's code nor something its rules have any business
            // judging. Walking it would also take minutes.
            let is_build_output = path
                .file_name()
                .is_some_and(|name| return name == "target" || name == ".git" || name == "node_modules");

            if !is_build_output
            {
                directories.push(path);
            }
        }

        return (directories, files);
    }

    /// `path` as a source a run can judge, when it is Rust this walk can read.
    fn Source_Of(path: &std::path::Path, root: &std::path::Path) -> Option<nomos_rules::SourceFile>
    {
        use nomos_rules::SourceFile;

        if !path.extension().is_some_and(|extension| return extension.eq_ignore_ascii_case("rs"))
        {
            return None;
        }

        let text = std::fs::read_to_string(path).ok()?;
        let relative = path.strip_prefix(root).unwrap_or(path);
        let spelling = relative.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");

        return Some(SourceFile::New(&spelling, nomos_model::Subject_Of_Path(&spelling), text));
    }
}
