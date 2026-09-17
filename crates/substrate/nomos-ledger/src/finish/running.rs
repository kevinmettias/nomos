//! Running a command to a verdict, and refusing one that did not reach one.

use super::{ItemId, FinishRefusal, Path, LedgerDocument, VerificationPredicate, ClaimRefusal, ProgramLauncher, Command, Tail_Of, ExitOutcome};

/// A predicate that ran and said no.
///
/// Its tail is carried into the refusal rather than only its code, because "not finished"
/// is not actionable and "not finished, here is what the run printed" is.
pub(super) fn Refuse_Nonzero(item: &ItemId, code: i32, tail: &str) -> Result<(), FinishRefusal>
{
    if code == 0
    {
        return Ok(());
    }

    return Err(FinishRefusal::PredicateFailed {
        item: item.clone(),
        exit_code: code,
        output_tail: tail.to_owned(),
    });
}

/// How a command is to be run: from where, and for how long at most.
///
/// The timeout is the item's own, so the gate step is bounded by the same patience the
/// predicate is. A gate that hung forever would refuse the item for a reason nobody could
/// distinguish from work that never finished.
#[derive(Clone, Copy)]
pub(super) struct Runner<'a>
{
    pub(super) working_directory: Option<&'a Path>,
    pub(super) timeout: std::time::Duration,
}

/// The item's predicate, if it has one that can actually be run.
///
/// Prose in `done_when` is not a predicate and a predicate with no program is not one
/// either. Both would let an item be finished on a check nobody performed.
pub(super) fn Runnable_Predicate<'a>(
    document: &'a LedgerDocument,
    item: &ItemId,
) -> Result<&'a VerificationPredicate, FinishRefusal>
{
    let predicate = Predicate_Of(document, item)?;
    if !predicate.Is_Runnable()
    {
        return Err(FinishRefusal::CouldNotRun {
            item: item.clone(),
            cause: "the predicate has no program to run".to_owned(),
        });
    }

    return Ok(predicate);
}

/// The predicate an item declares, if the item exists and declares one.
pub(super) fn Predicate_Of<'a>(
    document: &'a LedgerDocument,
    item: &ItemId,
) -> Result<&'a VerificationPredicate, FinishRefusal>
{
    let found = document.items.iter().find(|candidate| return &candidate.id == item);
    let Some(target) = found
    else
    {
        return Err(FinishRefusal::NotHeld {
            refusal: ClaimRefusal::NoSuchItem { item: item.clone() },
        });
    };

    let Some(predicate) = &target.verification
    else
    {
        return Err(FinishRefusal::NoPredicate {
            item: item.clone(),
            done_when: target.done_when.clone(),
        });
    };

    return Ok(predicate);
}

/// One command that reached a verdict, and the tail of what it said on the way.
pub(super) struct Ran
{
    pub(super) code: i32,
    pub(super) tail: String,
}

/// Runs a command to a verdict.
///
/// Both callers need the same three answers — the launcher would not start it, it ended
/// without a code, it ended with one — and answering them once is what keeps the gate step
/// and the predicate from drifting apart in how they treat a process nobody could ask.
pub(super) fn Ran_To_Completion(
    launcher: &impl ProgramLauncher,
    command: &Command,
    item: &ItemId,
) -> Result<Ran, FinishRefusal>
{
    use super::refusal::OUTPUT_TAIL_LIMIT;

    let output = launcher
        .Run(command)
        .map_err(|cause| FinishRefusal::CouldNotRun {
            item: item.clone(),
            cause,
        })?;
    let tail = Tail_Of(&format!("{}{}", output.stdout, output.stderr), OUTPUT_TAIL_LIMIT);

    let ExitOutcome::Exited { code } = output.outcome
    else
    {
        return Err(FinishRefusal::NoVerdict {
            item: item.clone(),
            outcome: output.outcome,
        });
    };

    return Ok(Ran { code, tail });
}

/// A command as this module builds them: what to run, where, and how long to wait.
///
/// Every command this module hands to a launcher gets an idle bound half its wall bound,
/// rather than the two coinciding as [`Command::From_String_Arguments`] alone would leave them. Left alone,
/// `nomos-ledger` was exactly the caller `OD-PLATFORM-001` named as still open: the one
/// place a real predicate runs, asking for no idle bound of its own, so a hung reader or a
/// deadlocked test case was indistinguishable from honest work all the way out to the wall
/// bound — 600 seconds, by [`crate::VerificationPredicate`]'s own default, before a stall
/// was even reported as anything other than a slow `TimedOut`.
///
/// Half was chosen over a smaller fraction because this module cannot tell a healthy
/// predicate's own rhythm from a stalled one before running it: `cargo test --workspace`,
/// the shape of predicate this ledger runs most, can go quiet for a real stretch mid-build
/// on one large crate without anything being wrong. A quarter of the wall bound risks
/// judging that stretch a stall; half leaves as much silence tolerated as the whole
/// pre-`OD-PLATFORM-001` wait allowed for genuinely slow work, while still cutting what a
/// truly hung predicate costs an author in two — 600 seconds of silence is now caught at
/// 300 rather than 600, without the gate step (bounded by the same `runner.timeout`, see
/// [`Runner`]) or the predicate itself ever waiting longer than before for the case that
/// actually finishes.
pub(super) fn Command_From_Argv(argv: Vec<String>, runner: Runner<'_>) -> Command
{
    let mut command = Command::From_String_Arguments(argv, runner.timeout).With_Idle_Timeout(Idle_Timeout(runner.timeout));
    command.working_directory = runner.working_directory.map(std::path::Path::to_path_buf);

    return command;
}

/// The fraction of the wall bound given to the idle bound: one half. See [`Command_From_Argv`] for
/// why half and not some other fraction.
const IDLE_TIMEOUT_DIVISOR: u32 = 2;

/// Half the wall bound, rounded down. See [`Command_From_Argv`] for why half and not some other
/// fraction.
///
/// `checked_div` rather than `/`: dividing by [`IDLE_TIMEOUT_DIVISOR`] cannot itself fail,
/// but this workspace denies raw arithmetic (`arithmetic_side_effects`) uniformly rather
/// than judging each call site's safety by eye, so the fallback -- unreachable, since
/// division by a nonzero constant always succeeds -- is the wall bound itself, never a
/// shorter idle bound silently produced by a wrapped or truncated calculation.
fn Idle_Timeout(timeout: std::time::Duration) -> std::time::Duration
{
    return timeout.checked_div(IDLE_TIMEOUT_DIVISOR).unwrap_or(timeout);
}

#[cfg(test)]
mod tests
{
    use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
    use super::*;
    use crate::{ItemKind, ItemOrigin, ItemState, LedgerItem, Territory};
    use nomos_platform::ProgramOutput;

    /// A nonzero exit code, which is all [`Refuse_Nonzero`] asks about. The case reads the
    /// number back out of the refusal.
    const NONZERO_EXIT_CODE: i32 = 3;

    /// How long the predicate in the tail case is given. Any positive bound does; the
    /// launcher answers immediately rather than running anything.
    const A_PREDICATE_TIMEOUT_SECONDS: u64 = 10;

    /// How long the runner in the working-directory case is given. A real gate's order of
    /// magnitude, so the case exercises the shape [`Command_From_Argv`] is built for.
    const A_RUNNER_TIMEOUT_SECONDS: u64 = 120;

    #[test]
    fn Test_Refuse_Nonzero_Should_Pass_A_Zero_Exit_Through_And_Refuse_Everything_Else()
    {
        let item = ItemId::New("T-1");

        assert!(Refuse_Nonzero(&item, 0, "all good").is_ok());

        let refusal = Refuse_Nonzero(&item, NONZERO_EXIT_CODE, "boom").expect_err("a nonzero exit must refuse");
        assert!(matches!(refusal, FinishRefusal::PredicateFailed { exit_code: NONZERO_EXIT_CODE, .. }), "got {refusal:?}");
    }

    #[test]
    fn Test_Runnable_Predicate_Should_Refuse_An_Argv_With_No_Program()
    {
        let item_with_predicate = Item_With_Predicate(
            "T-2",
            Some(VerificationPredicate::From_String_Arguments(vec![])),
        );
        let document = Board_Of(item_with_predicate);

        let refusal = Runnable_Predicate(&document, &ItemId::New("T-2")).expect_err("an empty argv cannot be run");

        assert!(matches!(refusal, FinishRefusal::CouldNotRun { .. }), "got {refusal:?}");
    }

    #[test]
    fn Test_Predicate_Of_Should_Report_No_Predicate_When_The_Item_Declares_None()
    {
        let item_with_predicate = Item_With_Predicate("T-3", None);
        let document = Board_Of(item_with_predicate);

        let no_predicate =
            Predicate_Of(&document, &ItemId::New("T-3")).expect_err("an item with none declared has none to return");
        assert!(matches!(no_predicate, FinishRefusal::NoPredicate { .. }), "got {no_predicate:?}");

        let missing =
            Predicate_Of(&document, &ItemId::New("GHOST")).expect_err("an unknown identifier matches nothing");
        assert!(
            matches!(missing, FinishRefusal::NotHeld { refusal: ClaimRefusal::NoSuchItem { .. } }),
            "got {missing:?}"
        );
    }

    /// A launcher standing in for a real one: it always ends the same way, with the exit
    /// code and streams this test hands it.
    struct Scripted
    {
        code: i32,
        stdout: String,
        stderr: String,
    }

    /// Answers from fixed data, so its outputs reproduce byte for byte.
    impl Strategy for Scripted
    {
        const STRENGTH: DeterminismStrength = DeterminismStrength::State;
        const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
        const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
    }

    impl ProgramLauncher for &Scripted
    {
        fn Run(&self, _command: &Command) -> Result<ProgramOutput, String>
        {
            return Ok(ProgramOutput {
                outcome: ExitOutcome::Exited { code: self.code },
                stdout: self.stdout.clone(),
                stderr: self.stderr.clone(),
            });
        }
    }

    #[test]
    fn Test_Ran_To_Completion_Should_Combine_Standard_Out_And_Error_Into_One_Tail()
    {
        let item = ItemId::New("T-4");
        let command = Command::From_String_Arguments(vec!["a-predicate".to_owned()], std::time::Duration::from_secs(A_PREDICATE_TIMEOUT_SECONDS));
        let launcher = Scripted { code: 0, stdout: "out-".to_owned(), stderr: "err".to_owned() };

        let ran = Ran_To_Completion(&&launcher, &command, &item).expect("a zero exit is a verdict");

        assert_eq!(ran.code, 0);
        assert_eq!(ran.tail, "out-err");
    }

    #[test]
    fn Test_Command_From_Argv_Should_Carry_The_Runners_Working_Directory_Onto_The_Command()
    {
        let runner = Runner {
            working_directory: Some(Path::new("some/tree")),
            timeout: std::time::Duration::from_secs(A_RUNNER_TIMEOUT_SECONDS),
        };

        let command = Command_From_Argv(vec!["cargo".to_owned(), "test".to_owned()], runner);

        assert_eq!(command.argv, vec!["cargo".to_owned(), "test".to_owned()]);
        assert_eq!(command.working_directory, Some(std::path::PathBuf::from("some/tree")));
    }

    fn Item_With_Predicate(id: &str, verification: Option<VerificationPredicate>) -> LedgerItem
    {
        return LedgerItem {
            id: ItemId::New(id),
            title: "an item".to_owned(),
            why: "because".to_owned(),
            done_when: "when it is done".to_owned(),
            kind: ItemKind::Correction,
            origin: ItemOrigin::Proposed,
            territory: Territory::Of_Files(["src/a.rs"]),
            state: ItemState::Claimed,
            depends_on: Vec::new(),
            blocked: None,
            claim: None,
            verification,
            verified: None,
            abandoned: Vec::new(),
            displaced: Vec::new(),
            widened: Vec::new(),
            declined: None,
        };
    }

    fn Board_Of(item: LedgerItem) -> LedgerDocument
    {
        return LedgerDocument { schema_version: 1, items: vec![item] };
    }
}
