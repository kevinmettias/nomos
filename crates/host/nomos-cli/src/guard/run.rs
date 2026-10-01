//! Composes `nomos_transition_guard` with this binary's standard ports and renders the answer.

use std::io::Write;
use std::path::{Path, PathBuf};

use nomos_composer_std::{FILE_SYSTEM, FileSystem, LAUNCHER};
use nomos_transition_guard::{
    GitReader, Hook_Scripts, Judge, Judge_Commit_Being_Made, Judge_Message, Judge_Push, Party_Policy_From_Json, PartyPolicy, Refusal, Scan, Verdict,
};

use super::{ExitCode, GuardCommand, Verb};

/// The variable naming the policy when `--policy` is not given (`OD-POLICY-002` decision 1).
/// `main.rs` reads it, as it reads every variable, and hands it here as
/// [`GuardContext::policy_from_environment`].
pub const POLICY_VARIABLE: &str = "NOMOS_PARTY_POLICY";

/// What the process environment says, read once by `main.rs` and passed in as values, so this
/// module behaves the same under test as in a terminal.
pub struct GuardContext
{
    /// The policy [`POLICY_VARIABLE`] names, if it names one.
    pub policy_from_environment: Option<PathBuf>,
    /// Where git runs a hook: the repository's working tree.
    pub working_directory: PathBuf,
    /// This binary's own path, which `install` writes into the scripts.
    pub executable: Option<PathBuf>,
    /// What git wrote on `pre-push`'s standard input, and empty for every other verb.
    pub push_input: String,
}

/// Runs one `nomos guard` command.
pub fn Run(command: &GuardCommand, context: &GuardContext, stdout: &mut dyn Write, stderr: &mut dyn Write) -> ExitCode
{
    let policy_path = command.policy.clone().or_else(|| return context.policy_from_environment.clone());
    if let Verb::Install { into } = &command.verb
    {
        return Install(into, policy_path.as_deref(), context.executable.as_deref(), Streams { stdout, stderr });
    }
    let Some(policy_path) = policy_path
    else
    {
        let _ = writeln!(stderr, "nomos guard: no policy is named; give --policy <file> or set {POLICY_VARIABLE}");
        return if matches!(command.verb, Verb::Scan { .. }) { ExitCode::NothingJudged } else { ExitCode::Unusable };
    };
    let policy = match Policy_At(&policy_path)
    {
        Ok(policy) => policy,
        Err(reason) =>
        {
            let _ = writeln!(stderr, "nomos guard: refusing, because the policy at {} {reason}", policy_path.display());
            return ExitCode::Unusable;
        },
    };
    let judge = Judge::New(&policy);
    let root = match &command.verb
    {
        Verb::Scan { root } => root.clone(),
        _ => context.working_directory.clone(),
    };
    let git = GitReader::New(&LAUNCHER, &root);
    let verdict = match &command.verb
    {
        Verb::PreCommit => Judge_Commit_Being_Made(&judge, &git),
        Verb::CommitMessage { file } => match FILE_SYSTEM.Read_To_String(file)
        {
            Ok(message) => Judge_Message(&judge, &git, &message),
            Err(error) =>
            {
                let _ = writeln!(stderr, "nomos guard: refusing, because the commit message could not be read: {error:?}");
                return ExitCode::Unusable;
            },
        },
        Verb::PrePush { remote } => Judge_Push(&judge, &git, remote, &context.push_input),
        Verb::Scan { .. } => return Audit(&judge, &git, &root, Streams { stdout, stderr }),
        Verb::Install { .. } => return ExitCode::Usage,
    };

    return match verdict
    {
        Ok(Verdict::Clean | Verdict::OwnRepository) => ExitCode::Clean,
        Ok(Verdict::Refused(refusals)) =>
        {
            Report(&judge, &refusals, stderr);
            ExitCode::Refused
        },
        Err(error) =>
        {
            let _ = writeln!(stderr, "nomos guard: refusing, because {error}");
            ExitCode::Unusable
        },
    };
}

fn Policy_At(path: &Path) -> Result<PartyPolicy, String>
{
    let text = FILE_SYSTEM.Read_To_String(path).map_err(|error| return format!("could not be read ({error:?})"))?;
    return Party_Policy_From_Json(&text).map_err(|error| return format!("cannot be used: {error}"));
}

/// Where a verb's answer goes: what it found to standard output, refusals and failures to
/// standard error.
struct Streams<'output>
{
    stdout: &'output mut dyn Write,
    stderr: &'output mut dyn Write,
}

fn Audit<Launcher: nomos_composer_std::ProgramLauncher>(judge: &Judge<'_>, git: &GitReader<'_, Launcher>, root: &Path, streams: Streams<'_>) -> ExitCode
{
    let Streams { stdout, stderr } = streams;
    let outcome = match Scan(judge, git, &FILE_SYSTEM, root)
    {
        Ok(outcome) => outcome,
        Err(error) =>
        {
            let _ = writeln!(stderr, "nomos guard: {error}");
            return ExitCode::Unusable;
        },
    };
    if outcome.judged == 0
    {
        let _ = writeln!(stderr, "nomos guard: nothing judged at {} -- no tracked text file was read, or it is {}'s own repository", root.display(), judge.Party());
        return ExitCode::NothingJudged;
    }
    if outcome.refusals.is_empty()
    {
        let _ = writeln!(stdout, "nomos guard: {} file(s) judged, nothing refused", outcome.judged);
        return ExitCode::Clean;
    }
    Report(judge, &outcome.refusals, stderr);
    return ExitCode::Refused;
}

fn Install(into: &Path, policy: Option<&Path>, executable: Option<&Path>, streams: Streams<'_>) -> ExitCode
{
    let Streams { stdout, stderr } = streams;
    let Some(policy) = policy
    else
    {
        let _ = writeln!(stderr, "nomos guard install: name the policy the hooks will read, with --policy <file> or {POLICY_VARIABLE}");
        return ExitCode::Usage;
    };
    let Some(nomos) = executable
    else
    {
        let _ = writeln!(stderr, "nomos guard install: this binary cannot name its own path, so the hooks would have nothing to run");
        return ExitCode::Unusable;
    };
    let scripts = Hook_Scripts(&Slashed(nomos), &Slashed(policy));
    for script in &scripts
    {
        let path = into.join(script.name);
        if let Err(error) = FILE_SYSTEM.Replace_Atomically(&path, &script.text)
        {
            let _ = writeln!(stderr, "nomos guard install: could not write {} ({error:?}); does {} exist?", path.display(), into.display());
            return ExitCode::Unusable;
        }
    }
    let _ = writeln!(stdout, "wrote {} hook scripts to {}", scripts.len(), into.display());
    let _ = writeln!(stdout, "now:      git config --global core.hooksPath {}", Slashed(into));
    let _ = writeln!(stdout, "on Unix:  chmod +x {}/*", Slashed(into));
    return ExitCode::Clean;
}

/// A path with `/` separators, which both `sh` and git read on every platform.
fn Slashed(path: &Path) -> String
{
    return path.display().to_string().replace('\\', "/");
}

fn Report(judge: &Judge<'_>, refusals: &[Refusal], stderr: &mut dyn Write)
{
    let _ = writeln!(stderr, "\nnomos guard: REFUSED. This repository must carry nothing {} could claim.\n", judge.Party());
    for refusal in refusals
    {
        let _ = writeln!(stderr, "  [{}] {}\n      > {}\n      {}", refusal.rule, refusal.place, refusal.text, refusal.why);
    }
    let _ = writeln!(stderr, "\n{} refusal(s). Remove them and try again.", refusals.len());
    let _ = writeln!(
        stderr,
        "An identity refusal is fixed by committing as someone else (`git config user.email <address>`, then `git commit --amend --reset-author`). \
         A vetted false positive can be excepted only in the policy, never in a repository."
    );
}
