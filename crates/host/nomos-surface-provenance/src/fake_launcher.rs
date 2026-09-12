//! A launcher that answers by matching a substring of the argv it was asked to run,
//! never by running anything.
//!
//! `nomos-ledger`'s own test apparatus (`gate_covers_finish/launcher.rs`) answers by
//! shape — "is this the lint call or the predicate" — because it only ever has two
//! calls to tell apart. This report makes three calls per crate checked, so matching
//! answers a caller writes down are told apart by a fragment of the command line
//! instead, and a run against no scripted answer is a bug in the test rather than a
//! silent stub.

use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_platform::{Command, ExitOutcome, ProcessLauncher, ProcessOutput};

/// The stdout a scripted answer hands back. A distinct type from [`Stderr`] only so the
/// two adjacent `&str` positions in [`Scripted::Answer`] cannot be swapped without the
/// compiler noticing -- this is test-support fixture code, not part of the crate's
/// production surface, so a lightweight local pair is enough.
pub(crate) struct Stdout<'a>(pub(crate) &'a str);

/// The stderr a scripted answer hands back. See [`Stdout`].
pub(crate) struct Stderr<'a>(pub(crate) &'a str);

/// One scripted answer: a substring of the joined argv to match, and what to hand
/// back when it does.
struct ScriptedAnswer
{
    matching: String,
    code: i32,
    stdout: String,
    stderr: String,
}

/// A launcher whose every answer was written down by the test that built it.
pub(crate) struct Scripted
{
    answers: Vec<ScriptedAnswer>,
}

impl Scripted
{
    pub(crate) fn New() -> Self
    {
        return Self { answers: Vec::new() };
    }

    /// Adds an answer for the first command whose joined argv contains `matching`.
    #[must_use]
    pub(crate) fn Answer(mut self, matching: &str, code: i32, stdout: Stdout<'_>, stderr: Stderr<'_>) -> Self
    {
        self.answers.push(ScriptedAnswer {
            matching: matching.to_owned(),
            code,
            stdout: stdout.0.to_owned(),
            stderr: stderr.0.to_owned(),
        });

        return self;
    }
}

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for Scripted
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl ProcessLauncher for Scripted
{
    fn Run(&self, command: &Command) -> Result<ProcessOutput, String>
    {
        let joined = command.argv.join(" ");
        let answer = self
            .answers
            .iter()
            .find(|answer| joined.contains(answer.matching.as_str()))
            .ok_or_else(|| format!("no scripted answer matches: {joined}"))?;

        return Ok(ProcessOutput {
            outcome: ExitOutcome::Exited { code: answer.code },
            stdout: answer.stdout.clone(),
            stderr: answer.stderr.clone(),
        });
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::time::Duration;

    /// Every other test in this crate builds a [`Scripted`] only to give it answers
    /// immediately, so nothing else exercises the empty state on its own: a launcher
    /// nobody scripted anything for must refuse rather than invent an answer.
    #[test]
    fn Test_New_Should_Start_With_No_Scripted_Answers()
    {
        let launcher = Scripted::New();
        let command = Command::New(vec!["git".to_owned(), "diff".to_owned()], Duration::from_secs(1));

        let result = launcher.Run(&command);

        assert!(
            result.is_err(),
            "a launcher scripted with nothing must refuse rather than invent an answer"
        );
    }

    /// The dispatch every other test in this crate relies on without ever proving it
    /// directly: an answer is chosen by whether its `matching` fragment is a substring of
    /// the joined argv, not by the order `Answer` was called in.
    #[test]
    fn Test_Answer_Should_Match_By_Argv_Substring_Not_By_The_Order_Answers_Were_Added()
    {
        let launcher = Scripted::New()
            .Answer("log", 1, Stdout(""), Stderr("boom"))
            .Answer("diff", 0, Stdout("clean\n"), Stderr(""));
        let command = Command::New(
            vec!["git".to_owned(), "diff".to_owned(), "a".to_owned(), "b".to_owned()],
            Duration::from_secs(1),
        );

        let output = launcher.Run(&command).expect("the second scripted answer's substring matches this argv");

        assert_eq!(output.outcome, ExitOutcome::Exited { code: 0 });
        assert_eq!(output.stdout, "clean\n");
        assert_eq!(output.stderr, "");
    }
}
