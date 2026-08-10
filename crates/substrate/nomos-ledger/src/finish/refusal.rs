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
    /// Separate from [`FinishRefusal::PredicateFailed`] because the remedy differs. A
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
            } => Gate_Failed(item, argv, *exit_code, output_tail),
            Self::NotRecorded { cause } =>
            {
                format!("the predicate passed but the result could not be recorded: {cause}")
            }
        };
    }

    /// Whether the work itself was judged, as opposed to the check having been prevented.
    ///
    /// [`FinishRefusal::PredicateFailed`] and [`FinishRefusal::GateFailed`] say something
    /// about the work; they differ in what the author does next, not in whether an answer
    /// was reached. Everything else says something about the tooling, and a report that
    /// does not separate the two sends an author to fix the wrong thing.
    ///
    /// [`FinishRefusal::GateUndetermined`] is deliberately on the tooling side. Nobody
    /// found out whether the work passes the gate, and reporting that as failing work
    /// would be the same conflation one arm further down.
    #[must_use]
    pub const fn Judged_The_Work(&self) -> bool
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
pub(super) fn Gate_Failed(item: &ItemId, argv: &[String], exit_code: i32, output_tail: &str) -> String
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
