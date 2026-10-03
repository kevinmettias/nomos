//! The `Rules` step. `OD-GATE-036` decided that a finish runs it after `Lint` and before the
//! predicate, because two landings passed their own predicate and left a Blocking finding at
//! `HEAD` for hours, and one of them sat in a crate nothing depended on.
//!
//! Every case here is against a fixture workflow and a scripted launcher. What the step's
//! command is in this repository's own workflow is `repository_gate.rs`'s question, and that
//! the derivation follows the workflow is `derivation.rs`'s.

use crate::launcher::{
    Bench, Bench_At, Finish_In, Is_Rules_Step, RULES_OUTPUT, RULES_RUN, Scripted, Standing, State_Of, WORKFLOW,
    WORKFLOW_WITHOUT_RULES,
};
use nomos_ledger::{FinishRefusal, GateOutcome, GateUnknown, ItemId, ItemState, OptionalStepOutcome};
use std::time::Duration;

/// The exit code `gate run` gives a run with a Blocking finding.
const BLOCKING_FINDING_EXIT: i32 = 1;

/// Every nonzero exit `gate run` documents: a Blocking finding, a run that could not be
/// assembled, and a run that judged nothing. `OD-GATE-036` names all three as refusals, and a
/// guard that refused only the first would let a run that looked at nothing finish an item.
const NONZERO_RULES_EXITS: [i32; 3] = [1, 5, 6];

/// A timeout no default and no other fixture carries, so a step bounded by anything but the
/// item's own timeout cannot match it by coincidence.
const ITEM_TIMEOUT_SECONDS: u64 = 4_321;

/// The argv a bare `run:` line splits into.
fn Words(line: &str) -> Vec<String>
{
    return line.split_whitespace().map(str::to_owned).collect();
}

/// The instance `OD-GATE-036` measured: the predicate passes, the lint step passes, and the
/// rule layer reports a Blocking finding. The finish is refused as a failed gate step that
/// carries the step's command and what it printed, and the predicate never runs.
#[test]
fn Test_A_Blocking_Rules_Finding_Should_Refuse_The_Finish_Before_The_Predicate_Runs()
{
    let scripted = Scripted::New(0, 0).With_Rules_Exit(BLOCKING_FINDING_EXIT);
    let Bench { directory, mut ledger, launcher } = Bench_At("rules-blocking", Some(WORKFLOW), scripted);

    let Err(refusal) = Finish_In(&mut ledger, &directory, &launcher, "T-1")
    else
    {
        panic!("a Blocking rules finding must refuse the finish");
    };

    let FinishRefusal::GateFailed { argv, exit_code, output_tail, .. } = &refusal
    else
    {
        panic!("expected GateFailed, got {}", refusal.Describe());
    };
    assert_eq!(argv, &Words(RULES_RUN), "the refusal must name the Rules step's own command");
    assert_eq!(*exit_code, BLOCKING_FINDING_EXIT);
    assert!(output_tail.contains(RULES_OUTPUT), "the refusal must carry what the step printed: {output_tail}");
    assert!(refusal.Has_Judged_The_Work(), "a Blocking finding is an answer about the work");
    Assert_Ran_Lint_Then_Rules_Only(&launcher.Calls());
    Assert_Not_Finished(&directory);
}

/// Zero is the only success. A run that could not be assembled and a run that judged nothing
/// refuse exactly as a Blocking finding does.
#[test]
fn Test_Every_Nonzero_Exit_Of_The_Rules_Step_Should_Refuse_The_Finish()
{
    for code in NONZERO_RULES_EXITS
    {
        let scripted = Scripted::New(0, 0).With_Rules_Exit(code);
        let Bench { directory, mut ledger, launcher } =
            Bench_At(&format!("rules-exit-{code}"), Some(WORKFLOW), scripted);

        let refusal = Finish_In(&mut ledger, &directory, &launcher, "T-1");

        assert!(
            matches!(refusal, Err(FinishRefusal::GateFailed { exit_code, .. }) if exit_code == code),
            "a Rules step exiting {code} must refuse as a failed gate step, got {refusal:?}"
        );
        Assert_Not_Finished(&directory);
    }
}

/// The control for the two above, and the ordering: when every step is green, lint runs
/// first, the rule layer second, the predicate last, and the record carries what the rule
/// layer ran.
#[test]
fn Test_A_Green_Rules_Step_Should_Run_Between_Lint_And_The_Predicate_And_Be_Recorded()
{
    let scripted = Scripted::New(0, 0);
    let Bench { directory, mut ledger, launcher } = Bench_At("rules-green", Some(WORKFLOW), scripted);

    let record = Finish_In(&mut ledger, &directory, &launcher, "T-1")
        .unwrap_or_else(|refusal| panic!("every step passed, so the item finishes: {}", refusal.Describe()));

    let calls = launcher.Calls();
    let order: Vec<&str> = calls.iter().map(|argv| return Step_Called(argv)).collect();
    assert_eq!(order, ["Lint", "Rules", "predicate"], "{calls:?}");

    let ran = Some(OptionalStepOutcome::Ran(GateOutcome { argv: Words(RULES_RUN), exit_code: 0 }));
    assert_eq!(record.rules, ran, "the record must say the Rules step ran and what it ran");
    let Standing { state, verified } = State_Of(&directory);
    assert_eq!(state, ItemState::Done);
    assert_eq!(verified.and_then(|written| return written.rules), ran, "and so must the board");
}

/// KWB's gate declares `Lint`, `Test` and `Contract`. A repository that never declared a
/// `Rules` step made no claim for a finish to honour, so the finish goes ahead -- and the
/// record says the step was not declared, never that it passed.
#[test]
fn Test_A_Workflow_That_Declares_No_Rules_Step_Should_Finish_And_Record_It_As_Not_Declared()
{
    let scripted = Scripted::New(0, 0).With_Rules_Exit(BLOCKING_FINDING_EXIT);
    let Bench { directory, mut ledger, launcher } =
        Bench_At("rules-absent", Some(WORKFLOW_WITHOUT_RULES), scripted);

    let record = Finish_In(&mut ledger, &directory, &launcher, "T-1")
        .unwrap_or_else(|refusal| panic!("a workflow with no Rules step must still finish: {}", refusal.Describe()));

    assert_eq!(record.rules, Some(OptionalStepOutcome::NotDeclared));
    let calls = launcher.Calls();
    assert!(!calls.iter().any(|argv| return Is_Rules_Step(argv)), "nothing may run in a missing step's place: {calls:?}");
    let Standing { state, verified } = State_Of(&directory);
    assert_eq!(state, ItemState::Done);
    assert_eq!(
        verified.and_then(|written| return written.rules),
        Some(OptionalStepOutcome::NotDeclared),
        "the board must say the step was not declared, which is not the same as passed"
    );
}

/// A `Rules` step that is there and cannot be derived is not an absent one. A script and an
/// action both refuse as an undetermined gate, and nothing runs -- not even the lint step,
/// whose minutes would be spent on a finish that was always going to refuse.
#[test]
fn Test_A_Rules_Step_Nobody_Can_Derive_Should_Refuse_As_Undetermined_And_Run_Nothing()
{
    let scripted_body = WORKFLOW.replace(RULES_RUN, &format!("{RULES_RUN} || true"));
    let an_action = WORKFLOW.replace(&format!("run: {RULES_RUN}"), "uses: some/rules-action@v1");

    for (name, workflow) in [("rules-scripted", scripted_body), ("rules-action", an_action)]
    {
        let Bench { directory, mut ledger, launcher } = Bench_At(name, Some(&workflow), Scripted::New(0, 0));

        let refusal = Finish_In(&mut ledger, &directory, &launcher, "T-1");

        assert!(
            matches!(
                &refusal,
                Err(FinishRefusal::GateUndetermined { cause: GateUnknown::NotASingleCommand { step, .. }, .. })
                    if step == nomos_ledger::RULES_STEP
            ),
            "{name}: an underivable Rules step must refuse as undetermined, got {refusal:?}"
        );
        assert!(launcher.Calls().is_empty(), "{name}: nothing may run once the gate is undetermined");
        Assert_Not_Finished(&directory);
    }
}

/// The `Rules` step runs under the item's own bound, as the lint step does: the same wall
/// bound, the same idle bound derived from it, and the same tree.
#[test]
fn Test_The_Rules_Step_Should_Run_Under_The_Items_Own_Bound()
{
    let Bench { directory, mut ledger, launcher } = Bench_At("rules-bound", Some(WORKFLOW), Scripted::New(0, 0));
    With_Item_Timeout(&ledger, ITEM_TIMEOUT_SECONDS);

    Finish_In(&mut ledger, &directory, &launcher, "T-1")
        .unwrap_or_else(|refusal| panic!("every step passed, so the item finishes: {}", refusal.Describe()));

    let commands = launcher.Commands();
    let lint = commands.first().expect("the lint step ran first");
    let rules = commands.iter().find(|command| return Is_Rules_Step(&command.argv)).expect("the Rules step ran");
    assert_eq!(rules.timeout, Duration::from_secs(ITEM_TIMEOUT_SECONDS), "the item's own wall bound");
    assert_eq!(rules.idle_timeout, lint.idle_timeout, "the idle bound the lint step gets");
    assert!(rules.idle_timeout < rules.timeout, "an idle bound of its own, shorter than the wall bound");
    assert_eq!(rules.working_directory.as_deref(), Some(directory.as_path()), "the tree being finished");
}

/// A record written before the step was part of finishing stays exactly as it was: absent,
/// which is neither "ran" nor "not declared". Finishing another item rewrites the board and
/// must not fill it in.
#[test]
fn Test_A_Record_Written_Before_The_Rules_Step_Should_Not_Be_Backfilled()
{
    let Bench { directory, mut ledger, launcher } = Bench_At("rules-no-backfill", Some(WORKFLOW), Scripted::New(0, 0));
    With_A_Record_From_Before_The_Rules_Step(&ledger, &directory);

    Finish_In(&mut ledger, &directory, &launcher, "T-1")
        .unwrap_or_else(|refusal| panic!("every step passed, so the item finishes: {}", refusal.Describe()));

    let document = ledger.Load().expect("the ledger is readable");
    let earlier = document.items.iter().find(|item| return item.id == ItemId::New("T-0")).expect("the earlier item survives");
    let record = earlier.verified.as_ref().expect("the earlier item keeps its evidence");
    assert_eq!(record.rules, None, "a finish from before the Rules step must not acquire one");
}

/// Which step an argv is, by the same markers [`Scripted`] answers by.
fn Step_Called(argv: &[String]) -> &'static str
{
    if argv.iter().any(|argument| return argument == "clippy")
    {
        return "Lint";
    }
    if Is_Rules_Step(argv)
    {
        return "Rules";
    }

    return "predicate";
}

/// The two gate steps ran, in order, and the predicate did not.
fn Assert_Ran_Lint_Then_Rules_Only(calls: &[Vec<String>])
{
    let order: Vec<&str> = calls.iter().map(|argv| return Step_Called(argv)).collect();

    assert_eq!(order, ["Lint", "Rules"], "the predicate must not run behind a red Rules step: {calls:?}");
}

/// The item is still held and carries no record, which is what a refused finish leaves.
fn Assert_Not_Finished(directory: &std::path::Path)
{
    let Standing { state, verified } = State_Of(directory);

    assert_eq!(state, ItemState::Claimed, "the item must not have finished");
    assert!(verified.is_none(), "a refused finish leaves no verification record");
}

/// Gives the bench's one item a predicate bounded by `seconds` rather than the default.
fn With_Item_Timeout(ledger: &crate::launcher::BenchLedger, seconds: u64)
{
    let mut document = ledger.Load().expect("the ledger is readable");
    let item = document.items.first_mut().expect("the bench has its item");
    let predicate = item.verification.as_mut().expect("the bench's item carries a predicate");
    predicate.timeout_seconds = seconds;

    ledger.Save(&document).expect("test needs to rewrite its ledger");
}

/// Adds a `Done` item whose record was written before the `Rules` step existed: the key is
/// absent from the file, which is how every such record on the real board reads.
fn With_A_Record_From_Before_The_Rules_Step(ledger: &crate::launcher::BenchLedger, directory: &std::path::Path)
{
    let mut document = ledger.Load().expect("the ledger is readable");
    let mut earlier = document.items.first().cloned().expect("the bench has its item");
    earlier.id = ItemId::New("T-0");
    earlier.state = ItemState::Done;
    earlier.claim = None;
    earlier.verified = Some(nomos_ledger::VerificationRecord {
        argv: Words("cargo test"),
        exit_code: 0,
        output_tail: String::new(),
        verified_at: nomos_platform::Timestamp::From_Unix_Seconds(0),
        gate: None,
        rules: None,
        revision: None,
    });
    document.items.push(earlier);
    ledger.Save(&document).expect("test needs to rewrite its ledger");

    crate::launcher::Strip_Key_From_Ledger(directory, "rules");
}
