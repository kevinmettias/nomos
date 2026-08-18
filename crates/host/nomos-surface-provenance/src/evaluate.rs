//! Running the three queries and turning their output into `OD-STORE-002`'s join.
//!
//! Everything here is pure once a command's stdout is in hand, which is what makes it
//! unit-testable against a scripted launcher rather than against this repository's own
//! git history — the history a CI checkout is not guaranteed to have in full (see
//! `main.rs`).

use crate::git;
use nomos_platform::{Command, ExitOutcome, ProcessLauncher};
use std::path::Path;

/// One commit that touched a crate's surface snapshot, for a reader to judge — not for
/// this tool to judge on their behalf.
pub(crate) struct CommitRef
{
    pub(crate) hash: String,
    pub(crate) subject: String,
}

/// What the join found for one crate.
pub(crate) struct CrateFinding
{
    pub(crate) krate: String,
    pub(crate) surface_changed: bool,
    pub(crate) surface_commits: Vec<CommitRef>,
    pub(crate) records_touched: bool,
}

impl CrateFinding
{
    /// `OD-STORE-002`'s Worked Case, read as a boolean: the surface moved and no
    /// `docs/records/` commit landed in the same range.
    ///
    /// Reported, never asserted — this crate exits `0` whether or not any
    /// [`CrateFinding`] answers `true`. See `main.rs`.
    pub(crate) fn Is_A_Finding(&self) -> bool
    {
        return self.surface_changed && !self.records_touched;
    }
}

/// Whether any commit in `since..until` touched `docs/records/`.
///
/// One call for the whole run: the range does not vary per crate, and `docs/records/`
/// is not scoped to any one of them.
///
/// # Errors
///
/// Returns a message when `git` could not be run, or ran and refused (a bad revision,
/// most commonly).
pub(crate) fn Records_Touched(
    launcher: &impl ProcessLauncher,
    root: &Path,
    since: &str,
    until: &str,
) -> Result<bool, String>
{
    let command = git::Records_Touched_In_Range(root, since, until);
    let stdout = Ran(launcher, &command)?;

    return Ok(Nonempty(&stdout));
}

/// The finding for one crate, given whether `docs/records/` was already answered for
/// this range.
///
/// # Errors
///
/// Returns a message when either `git` call could not be run or refused.
pub(crate) fn Finding_For(
    launcher: &impl ProcessLauncher,
    root: &Path,
    since: &str,
    until: &str,
    krate: &str,
    records_touched: bool,
) -> Result<CrateFinding, String>
{
    let path = crate::discovery::Snapshot_Path(krate);

    let diff = Ran(launcher, &git::Endpoint_Diff(root, since, until, &path))?;
    let surface_changed = Nonempty(&diff);

    let surface_commits = if surface_changed
    {
        let history = Ran(launcher, &git::Path_History(root, since, until, &path))?;
        Parse_Commits(&history)
    }
    else
    {
        Vec::new()
    };

    return Ok(CrateFinding {
        krate: krate.to_owned(),
        surface_changed,
        surface_commits,
        records_touched,
    });
}

/// Runs `command` and returns its stdout, or a message describing why no answer came
/// back.
///
/// A non-zero exit and a timeout are both "no answer" here — the same distinction
/// `nomos_platform::ExitOutcome` draws generally: this report can state a finding only
/// from a query that actually completed and said yes or no.
fn Ran(launcher: &impl ProcessLauncher, command: &Command) -> Result<String, String>
{
    let output = launcher.Run(command)?;

    return match output.outcome
    {
        ExitOutcome::Exited { code: 0 } => Ok(output.stdout),
        ExitOutcome::Exited { code } => Err(format!(
            "`{}` exited {code}: {}",
            command.argv.join(" "),
            output.stderr.trim()
        )),
        other => Err(format!("`{}` produced no verdict: {other:?}", command.argv.join(" "))),
    };
}

/// Whether `text` holds anything but whitespace.
fn Nonempty(text: &str) -> bool
{
    return !text.trim().is_empty();
}

/// Parses `git log --format=%H\t%s` output into commit references, skipping any line
/// that does not carry both halves — defensive rather than load-bearing, since a real
/// `git` never emits a partial line from this format string.
fn Parse_Commits(text: &str) -> Vec<CommitRef>
{
    let mut commits = Vec::new();
    for line in text.lines()
    {
        if let Some((hash, subject)) = line.split_once('\t')
        {
            commits.push(CommitRef {
                hash: hash.to_owned(),
                subject: subject.to_owned(),
            });
        }
    }

    return commits;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::fake_launcher::Scripted;

    #[test]
    fn Test_A_Blank_Diff_Means_No_Finding()
    {
        let launcher = Scripted::New()
            .Answer("diff", 0, "", "")
            .Answer("log a..b --format=%H --", 0, "", "");
        let touched = Records_Touched(&launcher, Path::new("/repo"), "a", "b").expect("must run");
        let finding = Finding_For(&launcher, Path::new("/repo"), "a", "b", "nomos-model", touched)
            .expect("must run");

        assert!(!finding.surface_changed);
        assert!(!finding.Is_A_Finding());
    }

    #[test]
    fn Test_A_Changed_Surface_With_No_Records_Commit_Is_A_Finding()
    {
        let launcher = Scripted::New()
            .Answer("diff", 0, "tests/contract/surface/nomos-model.txt\n", "")
            .Answer("log a..b --format=%H --", 0, "", "")
            .Answer("log a..b --format=%H\t%s --", 0, "deadbeef\treblessed\n", "");
        let touched = Records_Touched(&launcher, Path::new("/repo"), "a", "b").expect("must run");
        let finding = Finding_For(&launcher, Path::new("/repo"), "a", "b", "nomos-model", touched)
            .expect("must run");

        assert!(finding.surface_changed);
        assert!(!finding.records_touched);
        assert!(finding.Is_A_Finding());
        assert_eq!(finding.surface_commits.len(), 1);
        assert_eq!(finding.surface_commits.first().map(|commit| commit.hash.as_str()), Some("deadbeef"));
    }

    #[test]
    fn Test_A_Changed_Surface_With_A_Records_Commit_Is_Not_A_Finding()
    {
        let launcher = Scripted::New()
            .Answer("diff", 0, "tests/contract/surface/nomos-model.txt\n", "")
            .Answer("log a..b --format=%H --", 0, "cafef00d\n", "")
            .Answer("log a..b --format=%H\t%s --", 0, "deadbeef\treal change\n", "");
        let touched = Records_Touched(&launcher, Path::new("/repo"), "a", "b").expect("must run");
        let finding = Finding_For(&launcher, Path::new("/repo"), "a", "b", "nomos-model", touched)
            .expect("must run");

        assert!(finding.surface_changed);
        assert!(finding.records_touched);
        assert!(!finding.Is_A_Finding());
    }

    #[test]
    fn Test_A_Bad_Revision_Is_An_Error_Not_A_Finding()
    {
        let launcher = Scripted::New().Answer("diff", 128, "", "fatal: bad revision 'nonsense'");

        let result = Finding_For(&launcher, Path::new("/repo"), "nonsense", "b", "nomos-model", false);

        assert!(result.is_err());
    }
}
