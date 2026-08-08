//! Finishing an item, which means running something rather than saying something.
//!
//! `done_when` is prose. A person reads it, agrees with it, and moves on. That is how
//! the prototype's ledger accumulated items marked complete whose work had not been
//! done: nothing stood between the claim of completion and the record of it.
//!
//! [`Finish`] is what stands there. It runs the item's [`VerificationPredicate`] and
//! writes the result into the item, so `Done` is a state the ledger arrives at by
//! observation.

use crate::exclusion::{ClaimRefusal, ExclusionLedger, ReleaseOutcome};
use crate::item::{ItemId, VerificationRecord};
use crate::store::FileLedger;
use nomos_platform::{
    Clock, Command, CrossProcessLock, ExitOutcome, FileSystem, ProcessLauncher,
};
use std::path::Path;

/// How much of the predicate's output to keep on the item.
///
/// Enough to see a test failure, not so much that one verbose run makes the committed
/// ledger unreadable.
const OUTPUT_TAIL_LIMIT: usize = 2_000;

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
            Self::NoPredicate { item, done_when } => format!(
                "{item} has no verification predicate, so it cannot be finished. \
                 done_when says \"{done_when}\", and prose is not a predicate"
            ),
            Self::CouldNotRun { item, cause } => format!(
                "{item}'s predicate could not be started: {cause}. This is a broken \
                 predicate, not failing work"
            ),
            Self::NoVerdict { item, outcome } => format!(
                "{item}'s predicate produced no verdict ({outcome:?}), so whether the work \
                 is finished is still unknown"
            ),
            Self::PredicateFailed {
                item,
                exit_code,
                output_tail,
            } => format!("{item}'s predicate exited {exit_code}, so it is not finished:\n{output_tail}"),
            Self::NotRecorded { cause } => {
                format!("the predicate passed but the result could not be recorded: {cause}")
            }
        };
    }

    /// Whether the work itself was judged, as opposed to the check having been prevented.
    ///
    /// Only [`FinishRefusal::PredicateFailed`] says anything about the work. Everything
    /// else says something about the tooling, and a report that does not separate the
    /// two sends an author to fix the wrong thing.
    #[must_use]
    pub const fn Judged_The_Work(&self) -> bool
    {
        return matches!(self, Self::PredicateFailed { .. });
    }
}

/// Runs an item's verification predicate and, if it passes, records the item as done.
///
/// `working_directory` of `None` runs the predicate wherever the caller already is,
/// which for a repository tool invoked from a repository is the right answer. A test, or
/// a caller that is not where it wants the predicate to run, names the directory.
///
/// # Errors
///
/// Returns a [`FinishRefusal`] naming what stopped it, and in particular distinguishing
/// a failing predicate from one that could not be asked.
pub fn Finish<F: FileSystem, C: Clock, L: CrossProcessLock>(
    ledger: &mut FileLedger<F, C, L>,
    launcher: &impl ProcessLauncher,
    item: &ItemId,
    holder: &str,
    working_directory: Option<&Path>,
) -> Result<VerificationRecord, FinishRefusal>
{
    let document = ledger
        .Load()
        .map_err(|error| FinishRefusal::NotRecorded {
            cause: error.to_string(),
        })?;

    let target = document
        .items
        .iter()
        .find(|candidate| &candidate.id == item)
        .ok_or_else(|| FinishRefusal::NotHeld {
            refusal: ClaimRefusal::NoSuchItem { item: item.clone() },
        })?;

    let Some(predicate) = &target.verification
    else
    {
        return Err(FinishRefusal::NoPredicate {
            item: item.clone(),
            done_when: target.done_when.clone(),
        });
    };

    if !predicate.Is_Runnable()
    {
        return Err(FinishRefusal::CouldNotRun {
            item: item.clone(),
            cause: "the predicate has no program to run".to_owned(),
        });
    }

    let mut command = Command::New(
        predicate.argv.clone(),
        std::time::Duration::from_secs(predicate.timeout_seconds),
    );
    command.working_directory = working_directory.map(std::path::Path::to_path_buf);

    let output = launcher
        .Run(&command)
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

    if code != 0
    {
        return Err(FinishRefusal::PredicateFailed {
            item: item.clone(),
            exit_code: code,
            output_tail: tail,
        });
    }

    let record = VerificationRecord {
        argv: predicate.argv.clone(),
        exit_code: code,
        output_tail: tail,
        verified_at: ledger.Now(),
    };

    ledger
        .Release(item, holder, ReleaseOutcome::Finished(record.clone()))
        .map_err(|refusal| FinishRefusal::NotHeld { refusal })?;

    return Ok(record);
}

/// The last `limit` bytes of `text`, on a character boundary.
///
/// Truncated from the front, because the end of a failing test run is where the failure
/// is and the beginning is where the compiler warnings are.
fn Tail_Of(text: &str, limit: usize) -> String
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

        assert!(
            FinishRefusal::PredicateFailed {
                item: item.clone(),
                exit_code: 1,
                output_tail: String::new(),
            }
            .Judged_The_Work()
        );

        for prevented in [
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
        ]
        {
            assert!(
                !prevented.Judged_The_Work(),
                "{} is not a statement about the work",
                prevented.Describe()
            );
        }
    }

    #[test]
    fn Test_Every_Refusal_Should_Describe_Itself_Usefully()
    {
        let item = ItemId::New("T-1");
        let refusals = [
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
                item,
                exit_code: 101,
                output_tail: "assertion failed".to_owned(),
            },
            FinishRefusal::NotRecorded {
                cause: "locked".to_owned(),
            },
        ];

        for refusal in &refusals
        {
            assert!(
                refusal.Describe().len() > 15,
                "{} is too terse to act on",
                refusal.Describe()
            );
        }
    }
}
