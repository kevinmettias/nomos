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
    since: git::Since<'_>,
    until: git::Until<'_>,
) -> Result<bool, String>
{
    let command = git::Records_Touched_In_Range(root, since, until);
    let stdout = Ran_Command(launcher, &command)?;

    return Ok(Has_Nonblank_Content(&stdout));
}

/// The commit range and repository root every per-crate query needs.
///
/// Grouped because [`Finding_For`] otherwise carries three coordinates that never vary
/// independently of one another within one run — they come from the same `--since`,
/// `--until` and `--root` this whole invocation was given.
pub(crate) struct CommitRange<'a>
{
    pub(crate) root: &'a Path,
    pub(crate) since: &'a str,
    pub(crate) until: &'a str,
}

/// A [`CommitRange`] together with whether `docs/records/` was already touched in that
/// same range -- answered once per run, not once per crate, and carried alongside the
/// range because [`Finding_For`] needs both to decide anything about one crate.
pub(crate) struct Query<'a>
{
    pub(crate) range: CommitRange<'a>,
    pub(crate) records_touched: bool,
}

/// The finding for one crate, given `query`'s range and its already-answered
/// `records_touched`.
///
/// # Errors
///
/// Returns a message when either `git` call could not be run or refused.
pub(crate) fn Finding_For(launcher: &impl ProcessLauncher, query: &Query<'_>, krate: &str) -> Result<CrateFinding, String>
{
    let path = crate::discovery::Snapshot_Path(krate);
    let range = &query.range;

    let diff_command = git::Endpoint_Diff(range.root, git::Since(range.since), git::Until(range.until), &path);
    let diff = Ran_Command(launcher, &diff_command)?;
    let surface_changed = Has_Nonblank_Content(&diff);

    let surface_commits = Surface_Commits(launcher, range, &path, surface_changed.into())?;

    return Ok(CrateFinding {
        krate: krate.to_owned(),
        surface_changed,
        surface_commits,
        records_touched: query.records_touched,
    });
}

/// Whether the surface snapshot actually differs between the two endpoints -- the
/// answer [`Finding_For`] already computed before asking [`Surface_Commits`] whether to
/// also list the commits that produced it. A named two-state type rather than a bare
/// `bool` parameter, so a call site reads as a decision rather than an unlabeled flag.
enum SurfaceChanged
{
    Yes,
    No,
}

impl From<bool> for SurfaceChanged
{
    fn from(value: bool) -> Self
    {
        return if value { Self::Yes } else { Self::No };
    }
}

/// Every commit touching `path` in `range`, or none when the surface never changed --
/// skipping the second `git log` call entirely rather than running it just to discard the
/// answer.
fn Surface_Commits(
    launcher: &impl ProcessLauncher,
    range: &CommitRange<'_>,
    path: &str,
    surface_changed: SurfaceChanged,
) -> Result<Vec<CommitRef>, String>
{
    if matches!(surface_changed, SurfaceChanged::No)
    {
        return Ok(Vec::new());
    }

    let history_command = git::Path_History(range.root, git::Since(range.since), git::Until(range.until), path);
    let history = Ran_Command(launcher, &history_command)?;

    return Ok(Parse_Commits(&history));
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

/// Runs `command` and returns its stdout, or a message describing why no answer came
/// back.
///
/// A non-zero exit and a timeout are both "no answer" here — the same distinction
/// `nomos_platform::ExitOutcome` draws generally: this report can state a finding only
/// from a query that actually completed and said yes or no.
fn Ran_Command(launcher: &impl ProcessLauncher, command: &Command) -> Result<String, String>
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
fn Has_Nonblank_Content(text: &str) -> bool
{
    return !text.trim().is_empty();
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::fake_launcher::{Scripted, Stderr, Stdout};

    #[test]
    fn Test_A_Blank_Diff_Means_No_Finding()
    {
        let launcher = Scripted::New()
            .Answer("diff", 0, Stdout(""), Stderr(""))
            .Answer("log a..b --format=%H --", 0, Stdout(""), Stderr(""));
        let finding = Joined_Finding(&launcher, Repository_Range(), "nomos-model");

        assert!(!finding.surface_changed);
        assert!(!finding.Is_A_Finding());
    }

    #[test]
    fn Test_A_Changed_Surface_With_No_Records_Commit_Is_A_Finding()
    {
        let launcher = Changed_Surface_Launcher("", "deadbeef\treblessed\n");
        let finding = Joined_Finding(&launcher, Repository_Range(), "nomos-model");

        assert!(finding.surface_changed);
        assert!(!finding.records_touched);
        assert!(finding.Is_A_Finding());
        assert_eq!(finding.surface_commits.len(), 1);
        assert_eq!(finding.surface_commits.first().map(|commit| commit.hash.as_str()), Some("deadbeef"));
    }

    #[test]
    fn Test_Records_Touched_Should_Suppress_A_Finding_Despite_A_Changed_Surface()
    {
        let launcher = Changed_Surface_Launcher("cafef00d\n", "deadbeef\treal change\n");
        let finding = Joined_Finding(&launcher, Repository_Range(), "nomos-model");

        assert!(finding.surface_changed);
        assert!(finding.records_touched);
        assert!(!finding.Is_A_Finding());
    }

    #[test]
    fn Test_Finding_For_Should_Fail_On_A_Bad_Revision_Rather_Than_Report_A_Finding()
    {
        let launcher = Scripted::New().Answer("diff", 128, Stdout(""), Stderr("fatal: bad revision 'nonsense'"));
        let query = Query {
            range: CommitRange { root: Path::new("/repo"), since: "nonsense", until: "b" },
            records_touched: false,
        };

        let result = Finding_For(&launcher, &query, "nomos-model");
        let error = Refusal_Of(result);

        // Names the failure rather than just its presence: a malformed input, an
        // out-of-range index, or a typo in the fixture would all still be `is_err()`, but
        // only `git diff` itself refusing a revision produces `git`'s own "exited 128"
        // and "bad revision" text.
        assert!(error.contains("exited 128"), "{error}");
        assert!(error.contains("bad revision"), "{error}");
    }

    /// The error `result` carries, or a panic naming why nothing else is reachable here: the
    /// launcher above is scripted to answer `git diff` with exit 128 and a bad-revision
    /// message, so reaching `Ok` would mean that propagation regressed rather than a condition
    /// this test should assert around.
    fn Refusal_Of(result: Result<CrateFinding, String>) -> String
    {
        return match result
        {
            Err(error) => error,
            Ok(_) => panic!("a bad revision must fail the diff query, not report a finding"),
        };
    }

    /// Runs the whole join a real invocation performs -- `Records_Touched` then
    /// `Finding_For`, both expected to succeed. Every test above wants exactly this
    /// sequence and differs only in what `launcher` answers, so they share it rather than
    /// each repeating the two-call join.
    fn Joined_Finding(launcher: &Scripted, range: CommitRange<'_>, krate: &str) -> CrateFinding
    {
        let records_touched =
            Records_Touched(launcher, range.root, git::Since(range.since), git::Until(range.until)).expect("must run");
        let query = Query { range, records_touched };

        return Finding_For(launcher, &query, krate).expect("must run");
    }

    /// The range and root every test in this module scripts a launcher against -- the git
    /// coordinates themselves are never the fact under test, only what `launcher` answers
    /// for them.
    fn Repository_Range() -> CommitRange<'static>
    {
        return CommitRange { root: Path::new("/repo"), since: "a", until: "b" };
    }

    /// A launcher scripted for `nomos-model`'s surface file changing -- the "diff touched
    /// the surface" premise both `Test_A_Changed_Surface_With_*` tests share, differing only
    /// in what commits the two `log` answers report: `records_log` for whether a records
    /// commit touched the range, `surface_log` for the surface commit itself.
    fn Changed_Surface_Launcher(records_log: &str, surface_log: &str) -> Scripted
    {
        return Scripted::New()
            .Answer("diff", 0, Stdout("tests/contract/surface/nomos-model.txt\n"), Stderr(""))
            .Answer("log a..b --format=%H --", 0, Stdout(records_log), Stderr(""))
            .Answer("log a..b --format=%H\t%s --", 0, Stdout(surface_log), Stderr(""));
    }
}
