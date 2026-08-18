//! A launcher that answers by matching a substring of the argv it was asked to run,
//! never by running anything.
//!
//! `nomos-ledger`'s own test apparatus (`gate_covers_finish/launcher.rs`) answers by
//! shape — "is this the lint call or the predicate" — because it only ever has two
//! calls to tell apart. This report makes three calls per crate checked, so matching
//! answers a caller writes down are told apart by a fragment of the command line
//! instead, and a run against no scripted answer is a bug in the test rather than a
//! silent stub.

use nomos_platform::{Command, ExitOutcome, ProcessLauncher, ProcessOutput};

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
    pub(crate) fn Answer(mut self, matching: &str, code: i32, stdout: &str, stderr: &str) -> Self
    {
        self.answers.push(ScriptedAnswer {
            matching: matching.to_owned(),
            code,
            stdout: stdout.to_owned(),
            stderr: stderr.to_owned(),
        });

        return self;
    }
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
