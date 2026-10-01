//! The four moments `OD-POLICY-002` decision 3 judges.
//!
//! Each stands aside first if the repository is one of the party's own, then judges only what
//! the moment adds: the commit being made, its message, or the commits a push would give a
//! remote that the remote does not already have. [`Scan`] is the audit of what a repository
//! already holds, and the one place a whole file is read.

use std::path::Path;

use nomos_platform::{FileSystem, ProgramLauncher};

use crate::git_reader::GitReader;
use crate::guard_error::GuardError;
use crate::judge::Judge;
use crate::refusal::{Place, Refusal};
use crate::unified_diff::Judge_Diff;

/// What judging a transition concluded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict
{
    /// Nothing the policy refuses.
    Clean,
    /// The repository is one of the party's own, so the guard stood aside.
    OwnRepository,
    /// What was refused, after exceptions.
    Refused(Vec<Refusal>),
}

/// What auditing a repository found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanOutcome
{
    /// How many files' contents were read and judged. Zero is not a clean audit.
    pub judged: usize,
    /// What was refused, after exceptions.
    pub refusals: Vec<Refusal>,
}

/// The line git writes above a verbose commit's diff; nothing below it is the message.
const SCISSORS: &str = "# ------------------------ >8";

/// A hash of all zeros: a push that deletes a ref, or a ref the remote does not have yet.
const ABSENT_COMMIT: &str = "0000000000000000000000000000000000000000";

/// The commit being made: who it will be recorded as, and every path and line it adds.
///
/// # Errors
///
/// [`GuardError::Git`] when git cannot answer.
pub fn Judge_Commit_Being_Made<Launcher: ProgramLauncher>(judge: &Judge<'_>, git: &GitReader<'_, Launcher>) -> Result<Verdict, GuardError>
{
    if Stands_Aside(judge, git)?
    {
        return Ok(Verdict::OwnRepository);
    }
    let mut refusals = Vec::new();
    for (variable, role) in [("GIT_AUTHOR_IDENT", "author"), ("GIT_COMMITTER_IDENT", "committer")]
    {
        let email = git.Email_Of(variable)?;
        refusals.extend(judge.Identity(&Place::Identity { commit: None, role }, &email));
    }
    refusals.extend(Judge_Diff(judge, None, &git.Staged_Diff()?));

    return Ok(Verdict_Of(judge, refusals));
}

/// A commit message, as git hands it to `commit-msg`: its comment lines and everything below a
/// verbose commit's scissors line are git's, not the author's, and are not judged.
///
/// # Errors
///
/// [`GuardError::Git`] when git cannot answer.
pub fn Judge_Message<Launcher: ProgramLauncher>(judge: &Judge<'_>, git: &GitReader<'_, Launcher>, message: &str) -> Result<Verdict, GuardError>
{
    if Stands_Aside(judge, git)?
    {
        return Ok(Verdict::OwnRepository);
    }
    let place = Place::Message { commit: None };
    let refusals = message
        .lines()
        .take_while(|line| return !line.starts_with(SCISSORS))
        .filter(|line| return !line.starts_with('#'))
        .flat_map(|line| return judge.Text(&place, line))
        .collect();

    return Ok(Verdict_Of(judge, refusals));
}

/// A push: `pushed` is what git writes on `pre-push`'s standard input, one
/// `<local ref> <local id> <remote ref> <remote id>` line per ref. Every commit the push would
/// give `remote` that it does not already have is judged whole: identities, message, and every
/// path and line it adds. That is the net under `git commit --no-verify`.
///
/// # Errors
///
/// [`GuardError::Git`] when git cannot answer.
pub fn Judge_Push<Launcher: ProgramLauncher>(judge: &Judge<'_>, git: &GitReader<'_, Launcher>, remote: &str, pushed: &str) -> Result<Verdict, GuardError>
{
    if Stands_Aside(judge, git)?
    {
        return Ok(Verdict::OwnRepository);
    }
    let mut tips = Vec::new();
    let mut excluded = Vec::new();
    for line in pushed.lines()
    {
        let fields: Vec<&str> = line.split_whitespace().collect();
        let (Some(local), Some(remote_id)) = (fields.get(1), fields.get(3)) else { continue };
        if *local == ABSENT_COMMIT
        {
            continue;
        }
        tips.push((*local).to_owned());
        if *remote_id != ABSENT_COMMIT && git.Has_Commit(remote_id)
        {
            excluded.push(format!("^{remote_id}"));
        }
    }
    if tips.is_empty()
    {
        return Ok(Verdict::Clean);
    }
    let log = git.Log_Of(&tips, &excluded, remote)?;

    return Ok(Verdict_Of(judge, Judge_Log(judge, &log)));
}

/// Every refusal in `git log -p` output written in [`crate::git_reader::LOG_FORMAT`].
fn Judge_Log(judge: &Judge<'_>, log: &str) -> Vec<Refusal>
{
    let mut refusals = Vec::new();
    for record in log.split('\u{1}')
    {
        let fields: Vec<&str> = record.splitn(5, '\0').collect();
        let [id, author, committer, message, diff] = fields.as_slice() else { continue };
        let commit = Some((*id).to_owned());
        refusals.extend(judge.Identity(&Place::Identity { commit: commit.clone(), role: "author" }, author));
        refusals.extend(judge.Identity(&Place::Identity { commit: commit.clone(), role: "committer" }, committer));
        let place = Place::Message { commit: commit.clone() };
        refusals.extend(message.lines().flat_map(|line| return judge.Text(&place, line)));
        refusals.extend(Judge_Diff(judge, Some(id), diff));
    }
    return refusals;
}

/// Every tracked file's path and text in the working tree. A file that is not text is skipped
/// and not counted as judged.
///
/// # Errors
///
/// [`GuardError::Git`] when git cannot answer.
pub fn Scan<Launcher: ProgramLauncher, Files: FileSystem>(
    judge: &Judge<'_>,
    git: &GitReader<'_, Launcher>,
    files: &Files,
    root: &Path,
) -> Result<ScanOutcome, GuardError>
{
    if Stands_Aside(judge, git)?
    {
        return Ok(ScanOutcome { judged: 0, refusals: Vec::new() });
    }
    let mut judged: usize = 0;
    let mut refusals = Vec::new();
    for path in git.Tracked_Files()?
    {
        refusals.extend(judge.Text(&Place::Path { commit: None, path: path.clone() }, &path));
        let Ok(text) = files.Read_To_String(&root.join(&path)) else { continue };
        if text.contains('\0')
        {
            continue;
        }
        judged = judged.saturating_add(1);
        for (index, line) in text.lines().enumerate()
        {
            let place = Place::Line { commit: None, path: path.clone(), line: index.saturating_add(1) };
            refusals.extend(judge.Text(&place, line));
        }
    }

    return Ok(ScanOutcome { judged, refusals: judge.Unexcepted(refusals) });
}

fn Stands_Aside<Launcher: ProgramLauncher>(judge: &Judge<'_>, git: &GitReader<'_, Launcher>) -> Result<bool, GuardError>
{
    return Ok(judge.Is_Own_Repository(&git.Remote_Urls()?, &git.Top_Level()?));
}

fn Verdict_Of(judge: &Judge<'_>, refusals: Vec<Refusal>) -> Verdict
{
    let refusals = judge.Unexcepted(refusals);
    return if refusals.is_empty() { Verdict::Clean } else { Verdict::Refused(refusals) };
}
