//! What the guard asks git, through the [`ProgramLauncher`] port.
//!
//! Every question is a `git` command run without a shell, in the repository's working tree,
//! with paths left unescaped (`core.quotePath=false`) so a refusal names a file as a person
//! would type it.

use std::path::{Path, PathBuf};
use std::time::Duration;

use nomos_platform::{Command, ProgramLauncher};

use crate::guard_error::GuardError;

/// How long one git question may take. A push of a long rewritten history reads every commit's
/// added lines, which is the slowest question asked.
const GIT_TIMEOUT: Duration = Duration::from_secs(1800);

/// The diff shape every judged diff is asked in: added lines only, real paths, no external diff
/// driver, and a rename as a delete plus an add so the new path's lines are all judged.
pub(crate) const DIFF_OPTIONS: [&str; 6] = ["-U0", "--no-color", "--no-ext-diff", "--no-renames", "--src-prefix=a/", "--dst-prefix=b/"];

/// The `git log` format [`crate::transition`] parses: a record per commit, opened by `\x01`, its
/// fields closed by `\0`, with the commit's diff after the last field.
pub(crate) const LOG_FORMAT: &str = "--format=%x01%H%x00%ae%x00%ce%x00%B%x00";

/// Asks git about one repository.
pub struct GitReader<'launcher, Launcher: ProgramLauncher>
{
    launcher: &'launcher Launcher,
    root: PathBuf,
}

impl<'launcher, Launcher: ProgramLauncher> GitReader<'launcher, Launcher>
{
    /// A reader for the repository whose working tree is at or above `root`.
    #[must_use]
    pub fn New(launcher: &'launcher Launcher, root: &Path) -> Self
    {
        return Self { launcher, root: root.to_path_buf() };
    }

    fn Run(&self, arguments: &[&str]) -> Result<String, GuardError>
    {
        let mut argv = vec!["git".to_owned(), "-c".to_owned(), "core.quotePath=false".to_owned()];
        argv.extend(arguments.iter().map(|argument| return (*argument).to_owned()));
        let mut command = Command::From_String_Arguments(argv, GIT_TIMEOUT);
        command.working_directory = Some(self.root.clone());
        let output = self.launcher.Run(&command).map_err(GuardError::Git)?;
        if !output.outcome.Is_Successful()
        {
            return Err(GuardError::Git(format!("git {}: {}", arguments.join(" "), output.stderr.trim())));
        }

        return Ok(output.stdout);
    }

    /// The e-mail the commit being made will record for `variable`, `GIT_AUTHOR_IDENT` or
    /// `GIT_COMMITTER_IDENT`, as git resolves it from configuration and environment.
    ///
    /// # Errors
    ///
    /// [`GuardError::Git`] when git cannot answer.
    pub fn Email_Of(&self, variable: &str) -> Result<String, GuardError>
    {
        let ident = self.Run(&["var", variable])?;
        let open = ident.rfind('<');
        let close = ident.rfind('>');
        return match (open, close)
        {
            (Some(open), Some(close)) if open < close => Ok(ident.get(open.saturating_add(1)..close).unwrap_or_default().to_owned()),
            _ => Ok(String::new()),
        };
    }

    /// The staged changes a commit is about to record, in [`DIFF_OPTIONS`]'s shape.
    ///
    /// # Errors
    ///
    /// [`GuardError::Git`] when git cannot answer.
    pub fn Staged_Diff(&self) -> Result<String, GuardError>
    {
        let mut arguments = vec!["diff", "--cached"];
        arguments.extend(DIFF_OPTIONS);
        arguments.push("--diff-filter=ACMR");
        return self.Run(&arguments);
    }

    /// Every remote URL the repository names.
    ///
    /// # Errors
    ///
    /// [`GuardError::Git`] when git cannot answer.
    pub fn Remote_Urls(&self) -> Result<Vec<String>, GuardError>
    {
        let listing = self.Run(&["remote", "-v"])?;
        let mut urls: Vec<String> = listing.lines().filter_map(|line| return line.split_whitespace().nth(1)).map(str::to_owned).collect();
        urls.dedup();
        return Ok(urls);
    }

    /// The working tree's top directory, as git prints it.
    ///
    /// # Errors
    ///
    /// [`GuardError::Git`] when git cannot answer, which outside a repository it cannot.
    pub fn Top_Level(&self) -> Result<String, GuardError>
    {
        return Ok(self.Run(&["rev-parse", "--show-toplevel"])?.trim().to_owned());
    }

    /// Whether `id` names a commit this repository holds.
    #[must_use]
    pub fn Has_Commit(&self, id: &str) -> bool
    {
        return self.Run(&["cat-file", "-e", &format!("{id}^{{commit}}")]).is_ok();
    }

    /// Every commit reachable from `tips` that is not reachable from `excluded` or from any
    /// remote-tracking ref of `remote`, each with its identities, message and added lines, in
    /// [`LOG_FORMAT`].
    ///
    /// # Errors
    ///
    /// [`GuardError::Git`] when git cannot answer.
    pub fn Log_Of(&self, tips: &[String], excluded: &[String], remote: &str) -> Result<String, GuardError>
    {
        let remotes = format!("--remotes={remote}");
        let mut arguments: Vec<&str> = vec!["log", "-p", LOG_FORMAT];
        arguments.extend(DIFF_OPTIONS);
        arguments.extend(tips.iter().map(String::as_str));
        arguments.extend(excluded.iter().map(String::as_str));
        if !remote.is_empty()
        {
            arguments.extend(["--not", remotes.as_str()]);
        }
        return self.Run(&arguments);
    }

    /// Every tracked path, repository-relative.
    ///
    /// # Errors
    ///
    /// [`GuardError::Git`] when git cannot answer.
    pub fn Tracked_Files(&self) -> Result<Vec<String>, GuardError>
    {
        let listing = self.Run(&["ls-files", "-z"])?;
        return Ok(listing.split('\0').filter(|path| return !path.is_empty()).map(str::to_owned).collect());
    }
}
