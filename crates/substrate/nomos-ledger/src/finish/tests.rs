//! What this module promises, exercised.

use super::*;
use nomos_platform::ProcessOutput;

#[test]
fn Test_Short_Output_Should_Be_Kept_Whole()
{
    assert_eq!(Tail_Of("all good", 100), "all good");
}

#[test]
fn Test_Long_Output_Should_Keep_The_End()
{
    let long = "a".repeat(50) + "the failure";

    let tail = Tail_Of(&long, 11);

    assert_eq!(tail, "the failure");
}

/// Slicing a multi-byte string at an arbitrary byte offset panics. A predicate that
/// prints a non-ASCII character must not be able to crash the ledger.
#[test]
fn Test_Truncation_Should_Survive_Multi_Byte_Characters()
{
    let text = "é".repeat(100);

    let tail = Tail_Of(&text, 51);

    assert!(tail.len() <= 51);
    assert!(tail.chars().all(|character| character == 'é'));
}

/// The distinction the whole enum exists for.
#[test]
fn Test_Only_A_Failing_Predicate_Should_Judge_The_Work()
{
    let item = ItemId::New("T-1");

    for judging in Refusals_That_Judged_The_Work(&item)
    {
        assert!(judging.Judged_The_Work(), "{} is a statement about the work", judging.Describe());
    }
    for prevented in Refusals_That_Judged_Nothing(&item)
    {
        assert!(
            !prevented.Judged_The_Work(),
            "{} is not a statement about the work",
            prevented.Describe()
        );
    }
}

/// Somebody found out and the answer was no.
///
/// A red gate is an answer about the work too. The author's next move differs from a failing
/// test, which is why it is a separate variant, but "nobody found out" it is not.
fn Refusals_That_Judged_The_Work(item: &ItemId) -> Vec<FinishRefusal>
{
    return vec![
        FinishRefusal::PredicateFailed {
            item: item.clone(),
            exit_code: 1,
            output_tail: String::new(),
        },
        FinishRefusal::GateFailed {
            item: item.clone(),
            argv: vec!["cargo".to_owned(), "clippy".to_owned()],
            exit_code: 101,
            output_tail: String::new(),
        },
    ];
}

/// Every refusal that means nobody found out, as opposed to somebody finding out and the
/// answer being no.
fn Refusals_That_Judged_Nothing(item: &ItemId) -> Vec<FinishRefusal>
{
    return vec![
        FinishRefusal::GateUndetermined {
            item: item.clone(),
            cause: GateUnknown::NoSuchStep {
                step: LINT_STEP.to_owned(),
            },
        },
        FinishRefusal::NoPredicate {
            item: item.clone(),
            done_when: "when it works".to_owned(),
        },
        FinishRefusal::CouldNotRun {
            item: item.clone(),
            cause: "no such program".to_owned(),
        },
        FinishRefusal::NoVerdict {
            item: item.clone(),
            outcome: ExitOutcome::TimedOut,
        },
        FinishRefusal::NotRecorded {
            cause: "locked".to_owned(),
        },
    ];
}

#[test]
fn Test_Every_Refusal_Should_Describe_Itself_Usefully()
{
    let item = ItemId::New("T-1");

    for refusal in Every_Refusal(item)
    {
        assert!(
            refusal.Describe().len() > 15,
            "{} is too terse to act on",
            refusal.Describe()
        );
    }
}

/// One of each, so that a variant added without a sentence of its own fails here.
fn Every_Refusal(item: ItemId) -> Vec<FinishRefusal>
{
    return vec![
        FinishRefusal::NotHeld {
            refusal: ClaimRefusal::NoSuchItem { item: item.clone() },
        },
        FinishRefusal::NoPredicate {
            item: item.clone(),
            done_when: "when it works".to_owned(),
        },
        FinishRefusal::CouldNotRun {
            item: item.clone(),
            cause: "no such program".to_owned(),
        },
        FinishRefusal::NoVerdict {
            item: item.clone(),
            outcome: ExitOutcome::TimedOut,
        },
        FinishRefusal::PredicateFailed {
            item: item.clone(),
            exit_code: 101,
            output_tail: "assertion failed".to_owned(),
        },
        FinishRefusal::GateUndetermined {
            item: item.clone(),
            cause: GateUnknown::NotASingleCommand {
                step: LINT_STEP.to_owned(),
                run: "a && b".to_owned(),
            },
        },
        FinishRefusal::GateFailed {
            item,
            argv: vec!["cargo".to_owned(), "clippy".to_owned()],
            exit_code: 101,
            output_tail: "indexing may panic".to_owned(),
        },
        FinishRefusal::NotRecorded {
            cause: "locked".to_owned(),
        },
    ];
}

/// [`Commanded`] must give every predicate it builds an idle bound of its own, or a real
/// `nomos-ledger` stall is indistinguishable from honest work until the wall bound -- the
/// gap `OD-PLATFORM-001` left open for whichever item wired a default in.
#[test]
fn Test_Commanded_Should_Give_The_Predicate_An_Idle_Bound_Shorter_Than_The_Wall_Bound()
{
    let runner = Runner {
        working_directory: None,
        timeout: std::time::Duration::from_secs(600),
    };

    let command = Commanded(vec!["a-predicate".to_owned()], runner);

    assert!(command.idle_timeout < command.timeout, "an idle bound equal to the wall bound can never fire first");
    assert_eq!(command.idle_timeout, std::time::Duration::from_secs(300));
}

/// A launcher standing in for the real one: it never sees a live process, and instead
/// answers as the real launcher would once its own idle-vs-wall arithmetic has run its
/// course. What is under test here is `running.rs`'s wiring -- that it hands the launcher
/// an idle bound shorter than the wall bound -- not the polling loop that turns silence
/// into `Stalled`, which is `nomos-platform-std`'s own territory and already proven there.
struct Simulated
{
    /// Whether the predicate this stands in for ever produces output again after it
    /// starts.
    keeps_producing: bool,
}

impl ProcessLauncher for &Simulated
{
    fn Run(&self, command: &Command) -> Result<ProcessOutput, String>
    {
        let outcome = if self.keeps_producing
        {
            // Progress keeps resetting the idle clock, so only the wall bound can ever
            // catch this one -- exactly today's behaviour, unaffected by the idle bound
            // `Commanded` now sets.
            ExitOutcome::TimedOut
        }
        else if command.idle_timeout < command.timeout
        {
            // Silent from the start, so the idle bound -- shorter than the wall bound --
            // is what actually catches it.
            ExitOutcome::Stalled {
                idle_elapsed: command.idle_timeout,
            }
        }
        else
        {
            // No idle bound of its own: the pre-`OD-PLATFORM-001` behaviour, where a
            // silent predicate can only ever be judged at the wall bound.
            ExitOutcome::TimedOut
        };

        return Ok(ProcessOutput {
            outcome,
            stdout: String::new(),
            stderr: String::new(),
        });
    }
}

/// The property this item exists for: a predicate that goes silent well before its wall
/// timeout is reported as `Stalled`, not eventually `TimedOut`, once it runs through
/// `Commanded`'s wiring.
#[test]
fn Test_A_Silent_Predicate_Should_Surface_As_Stalled_Not_Timed_Out()
{
    let item = ItemId::New("T-1");
    let runner = Runner {
        working_directory: None,
        timeout: std::time::Duration::from_secs(600),
    };
    let command = Commanded(vec!["a-predicate".to_owned()], runner);
    let launcher = Simulated { keeps_producing: false };

    let result = Ran_To_Completion(&&launcher, &command, &item);

    assert!(
        matches!(
            result,
            Err(FinishRefusal::NoVerdict {
                outcome: ExitOutcome::Stalled { .. },
                ..
            })
        ),
        "a predicate silent from the start should be caught by the idle bound, not the wall bound"
    );
}

/// The control for the test above: a predicate that keeps producing output right up to
/// its wall bound is genuinely slow, not stalled, and must still read as `TimedOut`. The
/// idle bound only ever catches silence -- it must never turn ongoing work into a false
/// stall.
#[test]
fn Test_A_Predicate_That_Keeps_Producing_Should_Still_Report_Timed_Out()
{
    let item = ItemId::New("T-1");
    let runner = Runner {
        working_directory: None,
        timeout: std::time::Duration::from_secs(600),
    };
    let command = Commanded(vec!["a-predicate".to_owned()], runner);
    let launcher = Simulated { keeps_producing: true };

    let result = Ran_To_Completion(&&launcher, &command, &item);

    assert!(
        matches!(
            result,
            Err(FinishRefusal::NoVerdict {
                outcome: ExitOutcome::TimedOut,
                ..
            })
        ),
        "ongoing output must never be reported as a stall"
    );
}

/// A launcher for a predicate that exits immediately, standing in for the ordinary case
/// this item must leave undisturbed.
struct ExitsPromptly;

impl ProcessLauncher for &ExitsPromptly
{
    fn Run(&self, _command: &Command) -> Result<ProcessOutput, String>
    {
        return Ok(ProcessOutput {
            outcome: ExitOutcome::Exited { code: 0 },
            stdout: "all good".to_owned(),
            stderr: String::new(),
        });
    }
}

/// A predicate that exits promptly reaches a verdict exactly as it did before this item:
/// the idle bound this module now sets must not disturb the case that was never in
/// question.
#[test]
fn Test_A_Predicate_That_Exits_Promptly_Should_Still_Reach_A_Verdict()
{
    let item = ItemId::New("T-1");
    let runner = Runner {
        working_directory: None,
        timeout: std::time::Duration::from_secs(600),
    };
    let command = Commanded(vec!["a-predicate".to_owned()], runner);

    let ran = Ran_To_Completion(&&ExitsPromptly, &command, &item).expect("a zero exit is a verdict");

    assert_eq!(ran.code, 0);
    assert_eq!(ran.tail, "all good");
}
