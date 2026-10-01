//! Composes `nomos_transition_guard` with this binary's standard ports and renders the answer.

use std::io::Write;
use std::path::{Path, PathBuf};

use nomos_composer_std::{FILE_SYSTEM, FileSystem, LAUNCHER};
use nomos_transition_guard::{
    GitReader, GuardError, Hook_Scripts, Judge, Judge_Commit_Being_Made, Judge_Message, Judge_Push, Party_Policy_From_Json, PartyPolicy, Refusal, Scan,
    Verdict,
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

/// The answers several policies can give, most serious first: a guard that could not judge
/// outranks a refusal, a refusal outranks a scan that judged nothing, and that outranks clean.
const MOST_SERIOUS_FIRST: [ExitCode; 4] = [ExitCode::Unusable, ExitCode::Refused, ExitCode::NothingJudged, ExitCode::Clean];

/// Runs one `nomos guard` command.
///
/// Every named policy is read before anything is judged, so one that cannot be used refuses
/// the transition on its own (`OD-POLICY-002` decision 4). Each is then judged on its own --
/// its own identities, rules, exceptions and own repositories -- so a policy that stands aside
/// for its party's repository silences no other, and the answer is the most serious one given.
pub fn Run(command: &GuardCommand, context: &GuardContext, stdout: &mut dyn Write, stderr: &mut dyn Write) -> ExitCode
{
    let policy_paths = if command.policies.is_empty() { context.policy_from_environment.iter().cloned().collect() } else { command.policies.clone() };
    if let Verb::Install { into } = &command.verb
    {
        return Install(into, &policy_paths, context.executable.as_deref(), Streams { stdout, stderr });
    }
    if policy_paths.is_empty()
    {
        let _ = writeln!(stderr, "nomos guard: no policy is named; give --policy <file> or set {POLICY_VARIABLE}");
        return if matches!(command.verb, Verb::Scan { .. }) { ExitCode::NothingJudged } else { ExitCode::Unusable };
    }
    let mut policies = Vec::new();
    for policy_path in &policy_paths
    {
        match Policy_At(policy_path)
        {
            Ok(policy) => policies.push(policy),
            Err(reason) =>
            {
                let _ = writeln!(stderr, "nomos guard: refusing, because the policy at {} {reason}", policy_path.display());
                return ExitCode::Unusable;
            },
        }
    }
    let root = match &command.verb
    {
        Verb::Scan { root } => root.clone(),
        _ => context.working_directory.clone(),
    };
    let git = GitReader::New(&LAUNCHER, &root);
    let message = match &command.verb
    {
        Verb::CommitMessage { file } => match FILE_SYSTEM.Read_To_String(file)
        {
            Ok(message) => message,
            Err(error) =>
            {
                let _ = writeln!(stderr, "nomos guard: refusing, because the commit message could not be read: {error:?}");
                return ExitCode::Unusable;
            },
        },
        _ => String::new(),
    };

    let mut answers = Vec::new();
    for (index, policy) in policies.iter().enumerate()
    {
        let judge = Judge::New(policy);
        let answer = match &command.verb
        {
            Verb::PreCommit => Answer(&judge, Judge_Commit_Being_Made(&judge, &git), stderr),
            Verb::CommitMessage { .. } => Answer(&judge, Judge_Message(&judge, &git, &message), stderr),
            Verb::PrePush { remote } => Answer(&judge, Judge_Push(&judge, &git, remote, &context.push_input), stderr),
            Verb::Scan { .. } =>
            {
                let which = if policies.len() > 1 { format!("policy {} of {}: ", index.saturating_add(1), policies.len()) } else { String::new() };
                Audit(&judge, &git, &Audited { root: &root, which: &which }, Streams { stdout: &mut *stdout, stderr: &mut *stderr })
            },
            Verb::Install { .. } => return ExitCode::Usage,
        };
        answers.push(answer);
    }

    return MOST_SERIOUS_FIRST.into_iter().find(|code| return answers.contains(code)).unwrap_or(ExitCode::Clean);
}

/// What one policy's verdict on a transition tells the shell, with its refusals printed.
fn Answer(judge: &Judge<'_>, verdict: Result<Verdict, GuardError>, stderr: &mut dyn Write) -> ExitCode
{
    return match verdict
    {
        Ok(Verdict::Clean | Verdict::OwnRepository) => ExitCode::Clean,
        Ok(Verdict::Refused(refusals)) =>
        {
            Report(judge, &refusals, stderr);
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

/// Where one policy's audit runs, and which policy it is.
struct Audited<'scan>
{
    /// The repository audited.
    root: &'scan Path,
    /// `policy 1 of 2: ` when several policies are judged, and empty for one, so the line on
    /// standard output tells them apart without naming a party.
    which: &'scan str,
}

fn Audit<Launcher: nomos_composer_std::ProgramLauncher>(judge: &Judge<'_>, git: &GitReader<'_, Launcher>, audited: &Audited<'_>, streams: Streams<'_>) -> ExitCode
{
    let Streams { stdout, stderr } = streams;
    let Audited { root, which } = *audited;
    let outcome = match Scan(judge, git, &FILE_SYSTEM, root)
    {
        Ok(outcome) => outcome,
        Err(error) =>
        {
            let _ = writeln!(stderr, "nomos guard: {which}{error}");
            return ExitCode::Unusable;
        },
    };
    if outcome.judged == 0
    {
        let _ = writeln!(
            stderr,
            "nomos guard: {which}nothing judged at {} -- no tracked text file was read, or it is {}'s own repository",
            root.display(),
            judge.Party()
        );
        return ExitCode::NothingJudged;
    }
    if outcome.refusals.is_empty()
    {
        let _ = writeln!(stdout, "nomos guard: {which}{} file(s) judged, nothing refused", outcome.judged);
        return ExitCode::Clean;
    }
    Report(judge, &outcome.refusals, stderr);
    return ExitCode::Refused;
}

fn Install(into: &Path, policies: &[PathBuf], executable: Option<&Path>, streams: Streams<'_>) -> ExitCode
{
    let Streams { stdout, stderr } = streams;
    if policies.is_empty()
    {
        let _ = writeln!(stderr, "nomos guard install: name the policy the hooks will read, with --policy <file> or {POLICY_VARIABLE}");
        return ExitCode::Usage;
    }
    let Some(nomos) = executable
    else
    {
        let _ = writeln!(stderr, "nomos guard install: this binary cannot name its own path, so the hooks would have nothing to run");
        return ExitCode::Unusable;
    };
    let policies: Vec<String> = policies.iter().map(|policy| return Slashed(policy)).collect();
    let scripts = Hook_Scripts(&Slashed(nomos), &policies);
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
