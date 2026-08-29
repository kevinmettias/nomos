//! The three git invocations the report needs, and nothing about running them.
//!
//! Every `Command` here is built and handed to a [`nomos_platform::ProcessLauncher`] by
//! the caller. Kept as plain construction rather than a method on the launcher so the
//! argv itself — the thing a reviewer actually wants to check — is visible without
//! stepping into a mock.

use nomos_platform::Command;
use std::path::Path;
use std::time::Duration;

/// Generous on purpose: `git log`/`git diff` over this repository's own history is fast,
/// and a report a human runs on demand can afford to wait rather than misreport a slow
/// disk as "unchanged".
const TIMEOUT: Duration = Duration::from_secs(60);

/// One endpoint of the commit range every query in this crate takes -- the revision
/// named after `--since`.
///
/// A distinct type from [`Until`] only so the two positions in a call such as
/// `Endpoint_Diff(root, since, until, path)` cannot be swapped without the compiler
/// noticing: `since` and `until` are both `&str` and name opposite ends of the same
/// range, which is exactly the pair a transposition would not announce itself.
#[derive(Clone, Copy)]
pub(crate) struct Since<'a>(pub(crate) &'a str);

/// The other endpoint of the range. See [`Since`].
#[derive(Clone, Copy)]
pub(crate) struct Until<'a>(pub(crate) &'a str);

/// Whether `path`'s blob differs between the two endpoints, read literally as
/// `OD-STORE-002`'s Worked Case states it — the two trees compared directly, not a
/// three-dot merge-base range.
///
/// `--name-only` against a single path answers exactly one question: is `path` among the
/// names `git diff` prints. Nonempty output means yes.
pub(crate) fn Endpoint_Diff(root: &Path, since: Since<'_>, until: Until<'_>, path: &str) -> Command
{
    return Command_From_Arguments_In(
        root,
        vec![
            "git".to_owned(),
            "diff".to_owned(),
            since.0.to_owned(),
            until.0.to_owned(),
            "--name-only".to_owned(),
            "--".to_owned(),
            path.to_owned(),
        ],
    );
}

/// Every commit in the range `since..until` that touched `path`, one hash and subject
/// per line, tab-separated — the context a finding is rendered with, not the boolean
/// itself.
pub(crate) fn Path_History(root: &Path, since: Since<'_>, until: Until<'_>, path: &str) -> Command
{
    return Command_From_Arguments_In(
        root,
        vec![
            "git".to_owned(),
            "log".to_owned(),
            format!("{}..{}", since.0, until.0),
            "--format=%H\t%s".to_owned(),
            "--".to_owned(),
            path.to_owned(),
        ],
    );
}

/// Whether any commit in `since..until` touched `docs/records/`.
///
/// One call per report run, not per crate: the range is the same for every crate this
/// run checks, and `docs/records/` is not scoped to any one of them.
pub(crate) fn Records_Touched_In_Range(root: &Path, since: Since<'_>, until: Until<'_>) -> Command
{
    return Command_From_Arguments_In(
        root,
        vec![
            "git".to_owned(),
            "log".to_owned(),
            format!("{}..{}", since.0, until.0),
            "--format=%H".to_owned(),
            "--".to_owned(),
            "docs/records/".to_owned(),
        ],
    );
}

fn Command_From_Arguments_In(root: &Path, argv: Vec<String>) -> Command
{
    let mut command = Command::New(argv, TIMEOUT);
    command.working_directory = Some(root.to_path_buf());

    return command;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Endpoint_Diff_Compares_The_Two_Trees_Directly()
    {
        let command = Endpoint_Diff(Path::new("/repo"), Since("a"), Until("b"), "tests/contract/surface/x.txt");

        assert_eq!(
            command.argv,
            vec!["git", "diff", "a", "b", "--name-only", "--", "tests/contract/surface/x.txt"]
        );
        assert_eq!(command.working_directory.as_deref(), Some(Path::new("/repo")));
    }

    #[test]
    fn Test_Records_Touched_Scopes_To_Documentation_Records_Only()
    {
        let command = Records_Touched_In_Range(Path::new("/repo"), Since("a"), Until("b"));

        assert_eq!(
            command.argv,
            vec!["git", "log", "a..b", "--format=%H", "--", "docs/records/"]
        );
    }

    #[test]
    fn Test_Path_History_Carries_Hash_And_Subject()
    {
        let command = Path_History(Path::new("/repo"), Since("a"), Until("b"), "tests/contract/surface/x.txt");

        assert_eq!(
            command.argv,
            vec!["git", "log", "a..b", "--format=%H\t%s", "--", "tests/contract/surface/x.txt"]
        );
    }
}
