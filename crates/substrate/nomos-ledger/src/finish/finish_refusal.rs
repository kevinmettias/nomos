//! Every way finishing is refused, in the words the holder gets.

use super::{ClaimRefusal, ItemId, ExitOutcome, GateUnknown};

/// How much of the predicate's output to keep on the item.
///
/// Enough to see a test failure, not so much that one verbose run makes the committed
/// ledger unreadable.
pub(super) const OUTPUT_TAIL_LIMIT: usize = 2_000;

/// Why an item could not be finished.
///
/// The split that matters runs between [`FinishRefusal::PredicateFailed`] and everything
/// above it. A non-zero exit is an answer: the work is not done. A missing program, a
/// timeout or a killed process is the *absence* of an answer, and recording either of
/// those as "the check failed" would tell an author their work is wrong when the truth
/// is that nobody found out. That is the same conflation this system exists to prevent
/// one level up, so it is not permitted here either.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FinishRefusal
{
    /// The item is not this holder's to finish, or does not exist.
    NotHeld
    {
        /// What stopped it.
        refusal: ClaimRefusal,
    },
    /// The item carries no verification predicate.
    ///
    /// Refused rather than waved through. An item with nothing to run cannot be shown
    /// to be finished, and "there was nothing to check" must not read the same as
    /// "everything checked out".
    NoPredicate
    {
        /// The item.
        item: ItemId,
        /// The prose that is not a substitute for one.
        done_when: String,
    },
    /// The predicate could not be started at all.
    CouldNotRun
    {
        /// The item.
        item: ItemId,
        /// Why not.
        cause: String,
    },
    /// The predicate ran but never produced a verdict.
    NoVerdict
    {
        /// The item.
        item: ItemId,
        /// How it ended instead.
        outcome: ExitOutcome,
    },
    /// The predicate ran, and the answer was no.
    PredicateFailed
    {
        /// The item.
        item: ItemId,
        /// What it exited with.
        exit_code: i32,
        /// The tail of what it printed.
        output_tail: String,
    },
    /// What the gate checks could not be established.
    ///
    /// Not a licence to run the item's predicate alone. An item finished without knowing
    /// what the gate checks is the defect this arm exists to prevent, and the rule is the
    /// one [`ClaimRefusal::UnknownIndependence`] already applies one level up: an
    /// unanswered question refuses rather than grants.
    GateUndetermined
    {
        /// The item.
        item: ItemId,
        /// Why the gate could not be derived.
        cause: GateUnknown,
    },
    /// The gate's own step ran, and the answer was no.
    ///
    /// Separate from [`Self::PredicateFailed`] because the remedy differs. A
    /// failing predicate says the work does not do what the item asked. A failing gate
    /// step says the work may be exactly right and still cannot land, and telling an
    /// author the first when the truth is the second sends them to rewrite working code.
    GateFailed
    {
        /// The item.
        item: ItemId,
        /// What ran, as derived from the workflow.
        argv: Vec<String>,
        /// What it exited with.
        exit_code: i32,
        /// The tail of what it printed.
        output_tail: String,
    },
    /// The result could not be written to the ledger.
    NotRecorded
    {
        /// Why not.
        cause: String,
    },
}

impl FinishRefusal
{
    /// A one-line explanation a person or an agent can act on.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::NotHeld { refusal } => refusal.Describe(),
            Self::NoPredicate { item, done_when } => No_Predicate(item, done_when),
            Self::CouldNotRun { item, cause } => Could_Not_Run(item, cause),
            Self::NoVerdict { item, outcome } => No_Verdict(item, *outcome),
            Self::PredicateFailed {
                item,
                exit_code,
                output_tail,
            } => Predicate_Failed(item, *exit_code, output_tail),
            Self::GateUndetermined { item, cause } => Gate_Undetermined(item, cause),
            Self::GateFailed {
                item,
                argv,
                exit_code,
                output_tail,
            } => Gate_Failed_From_Argv(item, argv, *exit_code, output_tail),
            Self::NotRecorded { cause } =>
            {
                format!("the predicate passed but the result could not be recorded: {cause}")
            }
        };
    }

    /// Whether the work itself was judged, as opposed to the check having been prevented.
    ///
    /// [`Self::PredicateFailed`] and [`Self::GateFailed`] say something
    /// about the work; they differ in what the author does next, not in whether an answer
    /// was reached. Everything else says something about the tooling, and a report that
    /// does not separate the two sends an author to fix the wrong thing.
    ///
    /// [`Self::GateUndetermined`] is deliberately on the tooling side. Nobody
    /// found out whether the work passes the gate, and reporting that as failing work
    /// would be the same conflation one arm further down.
    #[must_use]
    pub const fn Has_Judged_The_Work(&self) -> bool
    {
        return matches!(self, Self::PredicateFailed { .. } | Self::GateFailed { .. });
    }
}

/// An item whose `done_when` is prose and nothing else.
pub(super) fn No_Predicate(item: &ItemId, done_when: &str) -> String
{
    return format!(
        "{item} has no verification predicate, so it cannot be finished. done_when says \
         \"{done_when}\", and prose is not a predicate"
    );
}

/// A predicate the launcher would not start.
pub(super) fn Could_Not_Run(item: &ItemId, cause: &str) -> String
{
    return format!(
        "{item}'s predicate could not be started: {cause}. This is a broken predicate, not \
         failing work"
    );
}

/// A predicate that ended without an exit code — killed, timed out, or lost.
pub(super) fn No_Verdict(item: &ItemId, outcome: ExitOutcome) -> String
{
    return format!(
        "{item}'s predicate produced no verdict ({outcome:?}), so whether the work is \
         finished is still unknown"
    );
}

/// A predicate that ran and said no.
pub(super) fn Predicate_Failed(item: &ItemId, exit_code: i32, output_tail: &str) -> String
{
    return format!("{item}'s predicate exited {exit_code}, so it is not finished:\n{output_tail}");
}

/// A gate nobody could ask.
pub(super) fn Gate_Undetermined(item: &ItemId, cause: &GateUnknown) -> String
{
    return format!(
        "{item} cannot be finished because {}. An item finished against a check weaker than \
         the gate is the defect this refusal exists to prevent",
        cause.Describe()
    );
}

/// A gate that ran and said no, before the item's own predicate was asked.
pub(super) fn Gate_Failed_From_Argv(item: &ItemId, argv: &[String], exit_code: i32, output_tail: &str) -> String
{
    return format!(
        "{item}'s work may be right and still cannot land: the gate's own step `{}` exited \
         {exit_code}. The item's predicate was not run.\n{output_tail}",
        argv.join(" ")
    );
}

/// The last `limit` bytes of `text`, on a character boundary.
///
/// Truncated from the front, because the end of a failing test run is where the failure
/// is and the beginning is where the compiler warnings are.
pub(super) fn Tail_Of(text: &str, limit: usize) -> String
{
    if text.len() <= limit
    {
        return text.to_owned();
    }

    let mut start = text.len().saturating_sub(limit);
    while start < text.len() && !text.is_char_boundary(start)
    {
        start = start.saturating_add(1);
    }

    return text.get(start..).unwrap_or_default().to_owned();
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The exit code a gate step reported when it failed. A real `cargo clippy` run's code,
    /// so the sentence under test is the one an author would actually read.
    const A_GATE_EXIT_CODE: i32 = 101;

    /// The exit code a predicate reported when it failed. Any nonzero would do; the case
    /// reads the number back out of the sentence.
    const A_PREDICATE_EXIT_CODE: i32 = 7;

    /// How long a tail the case asks [`Tail_Of`] to keep.
    const TAIL_LIMIT_CHARS: usize = 3;

    /// A report comfortably longer than [`TAIL_LIMIT_CHARS`], so the truncation the case is
    /// about actually happens rather than the whole text being handed straight back.
    const REPORT_TEXT_CHARS: usize = 10;

    #[test]
    fn Test_Describe_Should_Word_The_Not_Recorded_Case_Inline()
    {
        let refusal = FinishRefusal::NotRecorded { cause: "disk full".to_owned() };

        assert_eq!(
            refusal.Describe(),
            "the predicate passed but the result could not be recorded: disk full"
        );
    }

    #[test]
    fn Test_Has_Judged_The_Work_Should_Be_True_Only_For_The_Two_Verdict_Bearing_Variants()
    {
        let item = ItemId::New("T-1");

        assert!(
            FinishRefusal::PredicateFailed { item: item.clone(), exit_code: 1, output_tail: String::new() }
                .Has_Judged_The_Work()
        );
        assert!(
            FinishRefusal::GateFailed {
                item: item.clone(),
                argv: vec!["cargo".to_owned()],
                exit_code: A_GATE_EXIT_CODE,
                output_tail: String::new(),
            }
            .Has_Judged_The_Work()
        );
        assert!(!FinishRefusal::NotRecorded { cause: "locked".to_owned() }.Has_Judged_The_Work());
        assert!(
            !FinishRefusal::NoPredicate { item, done_when: "when it works".to_owned() }.Has_Judged_The_Work()
        );
    }

    #[test]
    fn Test_No_Predicate_Should_Quote_The_Items_Own_Done_When_Prose()
    {
        let item = ItemId::New("T-9");

        let sentence = No_Predicate(&item, "when the tests pass");

        assert!(sentence.contains("T-9"));
        assert!(sentence.contains("when the tests pass"));
        assert!(sentence.contains("prose is not a predicate"));
    }

    #[test]
    fn Test_Could_Not_Run_Should_Distinguish_A_Broken_Predicate_From_Failing_Work()
    {
        let item = ItemId::New("T-2");

        let sentence = Could_Not_Run(&item, "no such program");

        assert!(sentence.contains("no such program"));
        assert!(sentence.contains("not failing work"));
    }

    #[test]
    fn Test_No_Verdict_Should_Say_The_Question_Is_Still_Unknown()
    {
        let item = ItemId::New("T-3");

        let sentence = No_Verdict(&item, ExitOutcome::TimedOut);

        assert!(sentence.contains("TimedOut"));
        assert!(sentence.contains("still unknown"));
    }

    #[test]
    fn Test_Predicate_Failed_Should_Report_The_Exit_Code_And_Its_Output()
    {
        let item = ItemId::New("T-4");

        let sentence = Predicate_Failed(&item, A_PREDICATE_EXIT_CODE, "assertion failed");

        assert!(sentence.contains("exited 7"));
        assert!(sentence.contains("assertion failed"));
    }

    #[test]
    fn Test_Gate_Undetermined_Should_Explain_Why_The_Gate_Could_Not_Be_Asked()
    {
        let item = ItemId::New("T-5");
        let cause = GateUnknown::NoSuchStep { step: "Lint".to_owned() };

        let sentence = Gate_Undetermined(&item, &cause);

        assert!(sentence.contains("no step named"));
    }

    #[test]
    fn Test_Gate_Failed_From_Argv_Should_Name_The_Steps_Own_Command()
    {
        let item = ItemId::New("T-6");

        let sentence = Gate_Failed_From_Argv(&item, &["cargo".to_owned(), "clippy".to_owned()], A_GATE_EXIT_CODE, "warnings found");

        assert!(sentence.contains("cargo clippy"));
        assert!(sentence.contains("exited 101"));
        assert!(sentence.contains("warnings found"));
    }

    #[test]
    fn Test_Tail_Of_Should_Keep_The_Last_Bytes_On_A_Character_Boundary()
    {
        let text = "a".repeat(REPORT_TEXT_CHARS) + "END";

        let tail = Tail_Of(&text, TAIL_LIMIT_CHARS);

        assert_eq!(tail, "END");
    }
}
