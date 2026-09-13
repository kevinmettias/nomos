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
use std::collections::BTreeMap;

use crate::{GateFindings, GateRunResult};

/// Which of [`GateFindings`]'s own four buckets a finding fell into.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FindingDisposition
{
    Blocking,
    Calibrated,
    Suppressed,
    Baselined,
}

/// One finding whose identity (`rule`, `subject`) is present in both runs [`Compare_Gate_Runs`]
/// compared, but whose bucket differs between them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DispositionChange
{
    pub rule: RuleId,
    pub subject: SubjectId,
    pub subject_name: String,
    pub before: FindingDisposition,
    pub after: FindingDisposition,
}

/// What changed between `baseline` and `candidate`'s own [`GateFindings`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GateCompareResult
{
    /// The run compared against.
    pub baseline: RunId,
    /// The run being compared.
    pub candidate: RunId,
    /// Present in `candidate`, absent from `baseline` -- by (`rule`, `subject`) identity.
    pub added: Vec<Finding>,
    /// Present in `baseline`, absent from `candidate`.
    pub removed: Vec<Finding>,
    /// Present in both, but which bucket it fell into changed.
    pub changed: Vec<DispositionChange>,
}

/// Compares `baseline` and `candidate`'s own reduced findings, identifying one finding
/// with another across the two runs by (`rule`, `subject`) -- the same identity
/// [`crate::SuppressionPolicy`] and [`crate::BaselinePolicy`] already match a finding by,
/// since two runs naming the same subject under the same rule are answering the same
/// question even when the tree, or the policy judging it, moved between them.
#[must_use]
pub fn Compare_Gate_Runs(baseline: &GateRunResult, candidate: &GateRunResult) -> GateCompareResult
{
    let before = Indexed(&baseline.findings);
    let after = Indexed(&candidate.findings);

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
        .filter_map(|(key, (after_disposition, finding))| {
            let (before_disposition, _) = before.get(key)?;
            if before_disposition == after_disposition
            {
                return None;
            }

            return Some(DispositionChange {
                rule: finding.rule.clone(),
                subject: finding.subject,
                subject_name: finding.subject_name.clone(),
                before: *before_disposition,
                after: *after_disposition,
            });
        })
        .collect();

    added.sort_by(|left, right| return (&left.rule, &left.subject_name).cmp(&(&right.rule, &right.subject_name)));
    removed.sort_by(|left, right| return (&left.rule, &left.subject_name).cmp(&(&right.rule, &right.subject_name)));
    changed.sort_by(|left, right| return (&left.rule, &left.subject_name).cmp(&(&right.rule, &right.subject_name)));

    return GateCompareResult { baseline: baseline.run, candidate: candidate.run, added, removed, changed };
}

/// `findings`, keyed by (`rule`, `subject`) identity against which bucket each one fell
/// into -- a finding matched by more than one bucket cannot occur, the same disjointness
/// [`GateFindings`]'s own field docs already state.
fn Indexed(findings: &GateFindings) -> BTreeMap<(RuleId, SubjectId), (FindingDisposition, Finding)>
{
    let mut indexed = BTreeMap::new();

    for (bucket, disposition) in [
        (&findings.blocking_findings, FindingDisposition::Blocking),
        (&findings.calibrated_findings, FindingDisposition::Calibrated),
        (&findings.suppressed_findings, FindingDisposition::Suppressed),
        (&findings.baselined_findings, FindingDisposition::Baselined),
    ]
    {
        for finding in bucket
        {
            indexed.insert((finding.rule.clone(), finding.subject), (disposition, finding.clone()));
        }
    }

    return indexed;
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

    fn Run_Id_Of(fill: u8) -> RunId
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

        return Run_Gate(Some(sources), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &command, run);
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

        let compared = Compare_Gate_Runs(&baseline, &candidate);

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

        let compared = Compare_Gate_Runs(&baseline, &candidate);

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

        let compared = Compare_Gate_Runs(&baseline, &candidate);

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

        let baseline = Result_With(Run_Id_Of(7), GateFindings { blocking_findings: vec![finding.clone()], calibrated_findings: vec![], suppressed_findings: vec![], baselined_findings: vec![] });
        let candidate = Result_With(Run_Id_Of(8), GateFindings { blocking_findings: vec![], calibrated_findings: vec![], suppressed_findings: vec![finding], baselined_findings: vec![] });

        let compared = Compare_Gate_Runs(&baseline, &candidate);

        assert!(compared.added.is_empty(), "{:?}", compared.added);
        assert!(compared.removed.is_empty(), "{:?}", compared.removed);
        assert_eq!(compared.changed.len(), 1, "{:?}", compared.changed);
        let change = compared.changed.first().expect("asserted len 1 above");
        assert_eq!(change.before, FindingDisposition::Blocking);
        assert_eq!(change.after, FindingDisposition::Suppressed);
    }

    fn Result_With(run: RunId, findings: GateFindings) -> GateRunResult
    {
        return GateRunResult {
            run,
            root: std::path::PathBuf::from("."),
            check_outcome: nomos_check_orchestration::CheckOutcome::NoSource,
            findings,
            disposition: crate::GateRunOutcome::Indeterminate,
        };
    }
}
