use super::*;
use crate::test_support::{Plan_Rewriting, Plan_Rewriting_While_Reading, Rewrite, Unresolved_Plan};
use nomos_contracts::{Applicability, GateCategory, RuleId, SubjectId};
use nomos_platform_std::StdFileSystem;
use nomos_workspace::BuildVariant;

const FIRST: usize = 0;
const SECOND: usize = 1;
const THIRD: usize = 2;
const FOURTH: usize = 3;
const ONE_WAVE: usize = 1;
const TWO_WAVES: usize = 2;
const THREE_WAVES: usize = 3;
const THREE_ROUNDS: usize = 3;

/// What [`Reading_Fix`] leaves in the file it rewrites.
const CORRECTED: &str = "corrected";

/// A file declaring a mirror for a check that does not exist -- the exact shape
/// `phantom_mirror`'s own recognizer strikes, and the one line it strikes.
const PHANTOM_FIXTURE: &str = concat!(
    "/// A list of things this crate owns.\n",
    "/// Mirrored by `Test_Nonexistent_Check_That_Does_Not_Exist`.\n",
    "pub const THINGS: &[&str] = &[\"a\"];\n",
);

/// Two lines of one file ending in a space or a tab.
const TRAILING_WHITESPACE_FIXTURE: &str = "pub fn Something() -> u32 \n{\n    return 1; \t\n}\n";

/// [`TRAILING_WHITESPACE_FIXTURE`] with both flagged lines stripped.
const TRAILING_WHITESPACE_CORRECTED: &str = "pub fn Something() -> u32\n{\n    return 1;\n}\n";

/// A fresh temporary directory nothing else in this module writes into.
fn Root(name: &str) -> std::path::PathBuf
{
    let root = std::env::temp_dir().join(name);
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the temporary root is creatable");

    return root;
}

fn Test_Variant() -> BuildVariant
{
    return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
}

fn Scheduling_Over(root: &std::path::Path, commit: bool) -> WaveScheduling<'_, StdFileSystem>
{
    return WaveScheduling {
        root,
        variant: Test_Variant(),
        filesystem: &StdFileSystem,
        commit,
    };
}

/// A rerun a dry run never calls, and a partition refusal never reaches.
fn Never_Rerun(_scope: &[String]) -> Vec<Finding>
{
    return Vec::new();
}

/// A rerun that reports a state it has never reported before, every time it is asked.
///
/// For the cases whose claim is not about convergence. A rerun that kept reporting the
/// same state would be a stall, and the run would stop at the first committed wave for a
/// reason the case is not about -- which is itself the point of
/// [`Test_A_Wave_That_Changes_Nothing_Observable_Should_Stop_The_Run`].
fn Always_A_New_State() -> impl FnMut(&[String]) -> Vec<Finding>
{
    let mut round: usize = 0;

    return move |_scope: &[String]| {
        round = round.saturating_add(1);
        let location = format!("a.rs:{round}");

        return Observed(&[location.as_str()]);
    };
}

/// A rerun that reports each scripted state in turn, and nothing once they run out.
fn Scripted(rounds: Vec<Vec<Finding>>) -> impl FnMut(&[String]) -> Vec<Finding>
{
    let mut scripted = rounds.into_iter();

    return move |_scope: &[String]| {
        return scripted.next().unwrap_or_default();
    };
}

/// One blocking `no-trailing-whitespace` finding naming `location`.
fn Whitespace_Finding(location: &str) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(nomos_rules::NO_TRAILING_WHITESPACE),
        subject: SubjectId::From_Digest(Content_Digest(location.as_bytes())),
        subject_name: location.to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("{location} ends with trailing whitespace"),
        locations: vec![location.to_owned()],
    };
}

/// One blocking phantom finding for `claimed`, declared in `path` -- the one summary
/// shape `phantom_mirror::Phantom_Claim` ever parses.
fn Phantom_Finding(path: &str, claimed: &str) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(nomos_rules::COMPLETENESS_MIRROR),
        subject: SubjectId::From_Digest(Content_Digest(path.as_bytes())),
        subject_name: claimed.to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("`{claimed}` resolves to no check, so this rule is declared enforced and never runs"),
        locations: vec![path.to_owned()],
    };
}

/// The findings a scripted rerun reports in one round.
fn Observed(locations: &[&str]) -> Vec<Finding>
{
    return locations.iter().map(|location| return Whitespace_Finding(location)).collect();
}

/// A fix the wave runner can be handed directly, without a family having claimed it.
fn Fix(path: &str, before: &str, after: &str) -> ScheduledFix
{
    return Fix_With(Plan_Rewriting(path, before, after), path, before, after);
}

/// What a wave-2 plan looks like without two plans writing one path: it writes its own
/// file and declares it reads one another plan writes, which is a read-write conflict.
fn Reading_Fix(path: &str, before: &str, reads: &str) -> ScheduledFix
{
    let rewrite = Rewrite {
        path,
        from: before,
        to: CORRECTED,
    };

    return Fix_With(Plan_Rewriting_While_Reading(&rewrite, reads), path, before, CORRECTED);
}

fn Fix_With(plan: CorrectionPlan, path: &str, before: &str, after: &str) -> ScheduledFix
{
    return ScheduledFix {
        plan,
        path: path.to_owned(),
        summary: format!("rewrite {path}"),
        before: before.to_owned(),
        after: after.to_owned(),
        evidence_reference: Whitespace_Reference(path),
    };
}

fn Built_From(fixes: Vec<ScheduledFix>) -> ScheduledFixes
{
    return ScheduledFixes {
        fixes,
        unbuilt: Vec::new(),
    };
}

/// The paths every outcome in `report` is about, in the order they were attempted.
fn Attempted(report: &WaveReport) -> Vec<&str>
{
    return report.Outcomes().iter().map(PlanOutcome::Path).collect();
}

/// The report for the wave at `index`, which the case asking for it ran.
fn Wave_At(schedule: &CorrectionSchedule, index: usize) -> &WaveReport
{
    return schedule.Waves().get(index).expect("the case under test runs this wave");
}

/// The one observation `findings` reduces to, for a case that scripted exactly one.
fn One_Observation(findings: &[Finding]) -> String
{
    return Observations_Of(findings).first().cloned().expect("the caller scripted one finding");
}

/// The item's minimum interesting case: three plans, exactly one conflicting pair, and
/// the substrate's partition -- not this scheduler -- decides that the two compatible
/// ones share the first wave while the conflicting successor waits.
#[test]
fn Test_Three_Plans_With_One_Conflict_Should_Run_In_Two_Waves()
{
    let root = Root("nomos-correction-orchestration-waves-one-conflict");
    let fixes = vec![Fix("a.rs", "a-old", "a-new"), Fix("b.rs", "b-old", "b-new"), Reading_Fix("c.rs", "c-old", "a.rs")];

    let schedule = Scheduled_Run(Built_From(fixes), Scheduling_Over(&root, false), Never_Rerun);

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(schedule.Scheduled(), TWO_WAVES);
    assert_eq!(Wave_At(&schedule, FIRST).Positions(), [FIRST, SECOND]);
    assert_eq!(Wave_At(&schedule, SECOND).Positions(), [THIRD]);
    assert_eq!(Attempted(Wave_At(&schedule, FIRST)), ["a.rs", "b.rs"]);
    assert_eq!(Attempted(Wave_At(&schedule, SECOND)), ["c.rs"]);
    assert!(schedule.Halt().is_none());
}

/// The same three plans, committed: both waves run to the end, the wave-2 plan stages
/// against the workspace the wave-1 commits left, and all three files are written.
#[test]
fn Test_Two_Waves_Should_Both_Commit_When_Neither_Refuses()
{
    let root = Root("nomos-correction-orchestration-waves-both-commit");
    let fixes = vec![Fix("a.rs", "a-old", "a-new"), Fix("b.rs", "b-old", "b-new"), Reading_Fix("c.rs", "c-old", "a.rs")];

    let schedule = Scheduled_Run(Built_From(fixes), Scheduling_Over(&root, true), Always_A_New_State());

    let corrected = std::fs::read_to_string(root.join("c.rs")).expect("the second wave committed and wrote this file");
    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(schedule.Waves().len(), TWO_WAVES);
    assert_eq!(schedule.Committed(), ["a.rs", "b.rs", "c.rs"]);
    assert_eq!(corrected, CORRECTED);
    assert!(schedule.Halt().is_none(), "{:?}", schedule.Halt());
}

/// A dry run stages and validates every wave and commits nothing, so it never reruns
/// anything either -- an observation of a scope nothing wrote to could only ever repeat.
#[test]
fn Test_A_Dry_Run_Should_Stage_Every_Wave_And_Observe_Nothing()
{
    let root = Root("nomos-correction-orchestration-waves-dry-run");
    let fixes = vec![Fix("a.rs", "a-old", "a-new"), Fix("b.rs", "b-old", "b-new")];

    let schedule = Scheduled_Run(Built_From(fixes), Scheduling_Over(&root, false), Never_Rerun);

    let staged = Wave_At(&schedule, FIRST).Outcomes().iter().all(|outcome| return matches!(outcome, PlanOutcome::Staged { .. }));
    let _ignored = std::fs::remove_dir_all(&root);
    assert!(staged, "a dry run stages and validates without committing");
    assert_eq!(schedule.Rounds(), 0);
    assert!(schedule.Committed().is_empty());
}

/// The guard this scheduler owns: a wave whose plan refuses ends the run, and the waves
/// the partition computed after it are never staged. `Scheduled` and `Waves` say so
/// directly -- three waves computed, two attempted.
#[test]
fn Test_A_Refusing_Wave_Should_Not_Stage_The_Waves_After_It()
{
    let root = Root("nomos-correction-orchestration-waves-refusal");
    let fixes = vec![
        Fix("a.rs", "a-old", "a-new"),
        Fix("b.rs", "b-old", "b-new"),
        Fix("a.rs", "a-old", "a-other"),
        Fix("a.rs", "a-old", "a-third"),
    ];

    let schedule = Scheduled_Run(Built_From(fixes), Scheduling_Over(&root, true), Always_A_New_State());

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(schedule.Scheduled(), THREE_WAVES);
    assert_eq!(schedule.Waves().len(), TWO_WAVES, "the wave after the refusing one must never be staged");
    assert!(matches!(schedule.Halt(), Some(ScheduleHalt::Refused { wave: SECOND, position: THIRD, .. })), "{:?}", schedule.Halt());
}

/// What was already committed is kept and named, not silently kept and not silently
/// rolled back. `ScheduleHalt`'s own doc carries the argument for keeping it.
#[test]
fn Test_A_Refusing_Wave_Should_Report_What_Was_Already_Committed()
{
    let root = Root("nomos-correction-orchestration-waves-committed-then-refused");
    let fixes = vec![Fix("a.rs", "a-old", "a-new"), Fix("b.rs", "b-old", "b-new"), Fix("a.rs", "a-old", "a-other")];

    let schedule = Scheduled_Run(Built_From(fixes), Scheduling_Over(&root, true), Always_A_New_State());

    let on_disk = std::fs::read_to_string(root.join("a.rs")).expect("the first wave committed and wrote this file");
    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(schedule.Committed(), ["a.rs", "b.rs"]);
    assert_eq!(on_disk, "a-new", "committed work stays committed when a later wave refuses");
}

/// A committed plan carries the snapshots a deliberate rollback would need, which is what
/// makes keeping it a choice the caller can still undo rather than a fait accompli.
#[test]
fn Test_A_Committed_Plan_Should_Carry_The_Snapshots_A_Rollback_Would_Need()
{
    let root = Root("nomos-correction-orchestration-waves-snapshots");

    let schedule = Scheduled_Run(Built_From(vec![Fix("a.rs", "a-old", "a-new")]), Scheduling_Over(&root, true), Always_A_New_State());

    let outcome = Wave_At(&schedule, FIRST).Outcomes().first().cloned().expect("the one plan ran");
    let _ignored = std::fs::remove_dir_all(&root);
    match outcome
    {
        PlanOutcome::Committed { base, after_snapshot, .. } => assert_ne!(base, after_snapshot, "a commit moves the workspace"),
        other => panic!("expected Committed, got {other:?}"),
    }
}

/// Two corrections that undo each other put the affected scope back in a state the run
/// has already been in. The trail names the round that already held it, the run stops,
/// and the wave that reintroduced the finding names it -- by name, not as a count.
#[test]
fn Test_An_Oscillating_Pair_Should_Stop_At_The_Repeated_State()
{
    let root = Root("nomos-correction-orchestration-waves-oscillation");
    let fixes = vec![Fix("a.rs", "v0", "v1"), Fix("a.rs", "v1", "v0"), Fix("a.rs", "v0", "v2")];
    let script = vec![Observed(&["a.rs:1"]), Observed(&["a.rs:2"]), Observed(&["a.rs:1"])];

    let schedule = Scheduled_Run(Built_From(fixes), Scheduling_Over(&root, true), Scripted(script));

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(schedule.Scheduled(), THREE_WAVES);
    assert_eq!(schedule.Waves().len(), TWO_WAVES, "the wave after the repeated state must never be staged");
    assert_eq!(schedule.Rounds(), THREE_ROUNDS);
    assert!(matches!(schedule.Halt(), Some(ScheduleHalt::Repeated { wave: SECOND, round: FIRST })), "{:?}", schedule.Halt());
}

/// The compare half of the rerun: a finding the second wave brought back is named in that
/// wave's own `Introduced`, and the one it cleared in `Cleared`.
#[test]
fn Test_A_Rerun_Should_Name_What_A_Wave_Cleared_And_What_It_Introduced()
{
    let root = Root("nomos-correction-orchestration-waves-compare");
    let fixes = vec![Fix("a.rs", "v0", "v1"), Fix("a.rs", "v1", "v0"), Fix("a.rs", "v0", "v2")];
    let script = vec![Observed(&["a.rs:1"]), Observed(&["a.rs:2"]), Observed(&["a.rs:1"])];

    let schedule = Scheduled_Run(Built_From(fixes), Scheduling_Over(&root, true), Scripted(script));

    let second = Wave_At(&schedule, SECOND).clone();
    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(second.Introduced(), [One_Observation(&Observed(&["a.rs:1"]))]);
    assert_eq!(second.Cleared(), [One_Observation(&Observed(&["a.rs:2"]))]);
}

/// A run that keeps reaching new states is never stopped, however many waves it takes --
/// the property that separates an observed repeat from an attempt budget.
#[test]
fn Test_A_Run_That_Keeps_Reaching_New_States_Should_Not_Be_Stopped()
{
    let root = Root("nomos-correction-orchestration-waves-divergence");
    let fixes = vec![Fix("a.rs", "v0", "v1"), Fix("a.rs", "v1", "v2"), Fix("a.rs", "v2", "v3")];
    let script = vec![Observed(&["a:1"]), Observed(&["a:2"]), Observed(&["a:3"]), Observed(&["a:4"])];

    let schedule = Scheduled_Run(Built_From(fixes), Scheduling_Over(&root, true), Scripted(script));

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(schedule.Scheduled(), THREE_WAVES);
    assert_eq!(schedule.Waves().len(), THREE_WAVES);
    assert!(schedule.Halt().is_none(), "{:?}", schedule.Halt());
}

/// A plan the partition will not place is reported with the substrate's own refusal, and
/// no wave runs at all -- unknown independence is not safe parallelism, so there is no
/// partial schedule to fall back on.
#[test]
fn Test_A_Plan_The_Partition_Refuses_Should_Be_Reported_And_Never_Dropped()
{
    let root = Root("nomos-correction-orchestration-waves-unresolved");
    let unresolved = ScheduledFix {
        plan: Unresolved_Plan(),
        path: "unresolved.rs".to_owned(),
        summary: "unresolved".to_owned(),
        before: "before".to_owned(),
        after: "after".to_owned(),
        evidence_reference: Whitespace_Reference("unresolved.rs"),
    };
    let fixes = vec![Fix("a.rs", "a-old", "a-new"), unresolved];

    let schedule = Scheduled_Run(Built_From(fixes), Scheduling_Over(&root, true), Never_Rerun);

    let _ignored = std::fs::remove_dir_all(&root);
    let refusal = schedule.Unschedulable().expect("an incomplete derived set is unresolved");
    assert_eq!(refusal.Plan(), SECOND);
    assert_eq!(schedule.Scheduled(), 0);
    assert!(schedule.Waves().is_empty());
}

/// The whole point of the item, through the public entry: two findings over two files,
/// two plans, one wave, and the run learns they were independent rather than fixing one
/// family at a time.
#[test]
fn Test_Two_Independent_Findings_Should_Be_Scheduled_Into_One_Wave()
{
    let root = Root("nomos-correction-orchestration-waves-entry");
    std::fs::write(root.join("a.rs"), PHANTOM_FIXTURE).expect("the temporary root holds this file");
    std::fs::write(root.join("b.rs"), TRAILING_WHITESPACE_FIXTURE).expect("the temporary root holds this file");
    let findings = vec![Phantom_Finding("a.rs", "Test_Nonexistent_Check_That_Does_Not_Exist"), Whitespace_Finding("b.rs:1")];

    let schedule = Run_Correction_Waves(&findings, Scheduling_Over(&root, true), Always_A_New_State());

    let corrected = std::fs::read_to_string(root.join("b.rs")).expect("the one wave committed and wrote this file");
    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(schedule.Scheduled(), 1, "two independent corrections share one wave");
    assert_eq!(schedule.Committed(), ["a.rs", "b.rs"]);
    assert_eq!(corrected, TRAILING_WHITESPACE_CORRECTED);
}

/// A claim a family recognized but could not build a candidate for is reported rather
/// than dropped: a claim that vanished would look exactly like a tree that never had it.
#[test]
fn Test_A_Claim_That_Cannot_Be_Built_Should_Be_Reported()
{
    let root = Root("nomos-correction-orchestration-waves-unbuilt");
    let findings = vec![Phantom_Finding("missing.rs", "Test_Some_Check")];

    let schedule = Run_Correction_Waves(&findings, Scheduling_Over(&root, false), Never_Rerun);

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(schedule.Unbuilt().len(), 1, "{:?}", schedule.Unbuilt());
    let unbuilt = schedule.Unbuilt().first().cloned().expect("the claim that could not be built is reported");
    assert!(unbuilt.contains("missing.rs"), "{unbuilt}");
    assert_eq!(schedule.Scheduled(), 0);
}

/// Two claims on one path become one plan, the first family's. This module's own doc says
/// why: the second candidate's declared prior content is stale the instant the first
/// commits, so scheduling it would manufacture a refusal rather than discover one.
#[test]
fn Test_Two_Claims_On_One_Path_Should_Become_One_Plan()
{
    let root = Root("nomos-correction-orchestration-waves-one-plan-per-path");
    std::fs::write(root.join("a.rs"), PHANTOM_FIXTURE).expect("the temporary root holds this file");
    let findings = vec![Phantom_Finding("a.rs", "Test_Nonexistent_Check_That_Does_Not_Exist"), Whitespace_Finding("a.rs:1")];

    let schedule = Run_Correction_Waves(&findings, Scheduling_Over(&root, false), Never_Rerun);

    let attempted = Attempted(Wave_At(&schedule, FIRST));
    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(schedule.Scheduled(), 1);
    assert_eq!(attempted, ["a.rs"]);
}

#[test]
fn Test_Observations_Of_Should_Name_The_Rule_The_Subject_And_The_Locations()
{
    let observations = Observations_Of(&[Whitespace_Finding("a.rs:1")]);

    assert_eq!(observations, [format!("{}|a.rs:1|a.rs:1", nomos_rules::NO_TRAILING_WHITESPACE)]);
}

#[test]
fn Test_Absent_From_Should_Report_Only_What_The_Other_Side_Does_Not_Hold()
{
    let items = vec!["kept".to_owned(), "gone".to_owned()];
    let other = vec!["kept".to_owned()];

    assert_eq!(Absent_From(&items, &other), ["gone".to_owned()]);
}

#[test]
fn Test_Affected_Scope_Should_Name_Every_Written_Path_Once_In_Order()
{
    let fixes = vec![Fix("z.rs", "z", "Z"), Fix("a.rs", "a", "A"), Fix("z.rs", "z", "ZZ")];

    assert_eq!(Affected_Scope(&fixes), ["a.rs".to_owned(), "z.rs".to_owned()]);
}

#[test]
fn Test_A_Wave_Report_Should_Name_Every_Position_The_Partition_Placed_In_It()
{
    let root = Root("nomos-correction-orchestration-waves-positions");
    let fixes = vec![Fix("a.rs", "a", "A"), Fix("b.rs", "b", "B"), Fix("c.rs", "c", "C"), Fix("a.rs", "a", "AA")];

    let schedule = Scheduled_Run(Built_From(fixes), Scheduling_Over(&root, false), Never_Rerun);

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(Wave_At(&schedule, FIRST).Positions(), [FIRST, SECOND, THIRD]);
    assert_eq!(Wave_At(&schedule, SECOND).Positions(), [FOURTH]);
}

/// A stall is a repeated state with a nearer round. A wave that commits and changes
/// nothing the rerun can see leaves the scope where it was, and the run stops rather
/// than staging the next wave on the strength of a correction that did nothing
/// observable.
#[test]
fn Test_A_Wave_That_Changes_Nothing_Observable_Should_Stop_The_Run()
{
    let root = Root("nomos-correction-orchestration-waves-stall");
    let fixes = vec![Fix("a.rs", "v0", "v1"), Fix("a.rs", "v1", "v2"), Fix("a.rs", "v2", "v3")];
    let script = vec![Observed(&["a.rs:1"]), Observed(&["a.rs:1"])];

    let schedule = Scheduled_Run(Built_From(fixes), Scheduling_Over(&root, true), Scripted(script));

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(schedule.Scheduled(), THREE_WAVES);
    assert_eq!(schedule.Waves().len(), ONE_WAVE, "a stalled run must not stage the waves after it");
    assert!(matches!(schedule.Halt(), Some(ScheduleHalt::Repeated { wave: FIRST, round: FIRST })), "{:?}", schedule.Halt());
}
