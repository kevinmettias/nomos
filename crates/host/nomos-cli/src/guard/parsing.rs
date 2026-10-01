//! What `nomos guard` was asked for.

use std::path::PathBuf;

const POLICY: &str = "--policy";
const ROOT: &str = "--root";
const INTO: &str = "--into";

pub(super) const USAGE: &str = "usage: nomos guard pre-commit [--policy <file>]...\n\
     \x20      nomos guard commit-msg [--policy <file>]... <message-file>\n\
     \x20      nomos guard pre-push [--policy <file>]... <remote> [<url>]\n\
     \x20      nomos guard scan [--policy <file>]... [--root <path>]\n\
     \x20      nomos guard install --into <directory> [--policy <file>]...\n\n\
     Refuses a commit or push that would carry material a named party could claim: the party's \
e-mail as author or committer, or a phrase or ticket key its policy lists, in an added line, an \
added path or a commit message (OD-POLICY-002). The policy is a JSON file kept outside every \
repository, named by --policy or by NOMOS_PARTY_POLICY. --policy may be given once per party: \
each policy is judged on its own, and a refusal from any of them refuses. pre-commit, commit-msg \
and pre-push are git's hooks; scan audits what a repository already holds; install writes the \
hook scripts and prints the line that points git at them.\n\n\
     exit codes: 0 nothing refused (or the party's own repository, or scripts written), \
1 refused, 2 usage,\n\
     \x20           5 no policy, an unreadable policy, or git could not be asked,\n\
     \x20           6 scan judged nothing";

/// The verb and what it was given.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verb
{
    /// Judge the commit being made.
    PreCommit,
    /// Judge the message in this file.
    CommitMessage { file: PathBuf },
    /// Judge what a push would give this remote.
    PrePush { remote: String },
    /// Audit the repository at this root.
    Scan { root: PathBuf },
    /// Write the hook scripts into this directory.
    Install { into: PathBuf },
}

/// A parsed `nomos guard` command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardCommand
{
    /// What to do.
    pub verb: Verb,
    /// Every policy `--policy` named, in the order named; empty when it named none.
    pub policies: Vec<PathBuf>,
}

/// Reads `nomos guard`'s arguments.
///
/// # Errors
///
/// The usage text, with the reason, for an unknown verb or flag, a flag with no value, or a
/// verb missing what it needs.
pub fn Guard_Command_From_String_Arguments(arguments: &[String]) -> Result<GuardCommand, String>
{
    let (verb, rest) = arguments.split_first().ok_or_else(|| return USAGE.to_owned())?;
    let mut policies = Vec::new();
    let mut root = None;
    let mut into = None;
    let mut positional = Vec::new();
    let mut remaining = rest.iter();
    while let Some(argument) = remaining.next()
    {
        // `--policy` collects, because each one names a different party; the others name one place.
        let slot = match argument.as_str()
        {
            POLICY => None,
            ROOT => Some(&mut root),
            INTO => Some(&mut into),
            other if other.starts_with("--") => return Err(format!("unknown argument `{other}`\n\n{USAGE}")),
            other =>
            {
                positional.push(other.to_owned());
                continue;
            },
        };
        let value = PathBuf::from(remaining.next().ok_or_else(|| return format!("`{argument}` needs a value\n\n{USAGE}"))?);
        match slot
        {
            Some(slot) => *slot = Some(value),
            None => policies.push(value),
        }
    }
    let missing = |what: &str| return format!("`nomos guard {verb}` needs {what}\n\n{USAGE}");
    let verb = match verb.as_str()
    {
        "pre-commit" => Verb::PreCommit,
        "commit-msg" => Verb::CommitMessage { file: positional.first().map(PathBuf::from).ok_or_else(|| return missing("the message file"))? },
        "pre-push" => Verb::PrePush { remote: positional.first().cloned().ok_or_else(|| return missing("the remote"))? },
        "scan" => Verb::Scan { root: root.unwrap_or_else(|| return PathBuf::from(".")) },
        "install" => Verb::Install { into: into.ok_or_else(|| return missing("--into <directory>"))? },
        other => return Err(format!("unknown verb `{other}`\n\n{USAGE}")),
    };

    return Ok(GuardCommand { verb, policies });
}
