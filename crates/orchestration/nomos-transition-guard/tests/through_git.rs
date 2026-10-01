//! The transitions against real repositories, through real `git`, for an invented party.
//!
//! Each repository is isolated from the machine's own hooks (`core.hooksPath` points at an empty
//! directory) and identity, so what is judged is only what each test commits.

use std::path::{Path, PathBuf};
use std::process::Command as Process;

use nomos_platform::{Command, DeterminismStrength, ExitOutcome, FileSystem, FileSystemError, ProgramLauncher, ProgramOutput, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_transition_guard::{GitReader, Judge, Judge_Commit_Being_Made, Judge_Push, Party_Policy_From_Json, PartyPolicy, Scan, Verdict};

const POLICY: &str = r#"{
  "party": "Northwind",
  "identities": { "email_domains": ["northwind.example.invalid"] },
  "rules": [
    { "id": "party-name", "why": "names the party", "phrases": ["northwind"], "boundary": "anywhere" },
    { "id": "ticket", "why": "its tracker", "tickets": ["NW"] }
  ],
  "own_repositories": { "remotes": ["https://git.example.invalid/northwind/"] }
}"#;

const PERSONAL: &str = "me@home.example.invalid";

fn Policy() -> PartyPolicy
{
    return Party_Policy_From_Json(POLICY).expect("the test policy is valid");
}

/// Runs git for test setup, failing the test if it fails.
fn Git(directory: &Path, arguments: &[&str]) -> String
{
    let output = Process::new("git").arg("-C").arg(directory).args(arguments).output().expect("git runs");
    assert!(output.status.success(), "git {arguments:?}: {}", String::from_utf8_lossy(&output.stderr));
    return String::from_utf8_lossy(&output.stdout).trim().to_owned();
}

/// A repository with one clean commit, committing as a personal identity, isolated from this
/// machine's hooks.
fn Repository(scratch: &Path) -> PathBuf
{
    let root = scratch.join("repository");
    let hooks = scratch.join("no-hooks");
    std::fs::create_dir_all(&root).expect("create repository");
    std::fs::create_dir_all(&hooks).expect("create hooks");
    Git(&root, &["init", "-q", "-b", "main"]);
    Git(&root, &["config", "user.name", "Me"]);
    Git(&root, &["config", "user.email", PERSONAL]);
    Git(&root, &["config", "commit.gpgsign", "false"]);
    Git(&root, &["config", "core.hooksPath", &hooks.to_string_lossy()]);
    Write(&root, "README.md", "a personal project\n");
    Git(&root, &["add", "."]);
    Git(&root, &["commit", "-q", "-m", "start"]);
    return root;
}

fn Write(root: &Path, name: &str, text: &str)
{
    std::fs::write(root.join(name), text).expect("write file");
}

fn Scratch(name: &str) -> PathBuf
{
    let scratch = std::env::temp_dir().join(format!("nomos-transition-guard-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).expect("create scratch");
    return scratch;
}

#[test]
fn Test_A_Commit_Being_Made_Should_Be_Refused_For_An_Added_Line_And_Admitted_When_Clean()
{
    let scratch = Scratch("commit");
    let root = Repository(&scratch);
    let policy = Policy();
    let judge = Judge::New(&policy);
    let launcher = SystemGit;
    let git = GitReader::New(&launcher, &root);

    Write(&root, "notes.md", "ordinary notes\n");
    Git(&root, &["add", "."]);
    assert_eq!(Judge_Commit_Being_Made(&judge, &git).expect("git answers"), Verdict::Clean);

    Write(&root, "notes.md", "ordinary notes\nported from the Northwind scheduler\n");
    Git(&root, &["add", "."]);
    let Verdict::Refused(found) = Judge_Commit_Being_Made(&judge, &git).expect("git answers") else { panic!("an added party name passed") };
    assert_eq!(found.iter().map(|refusal| return refusal.place.to_string()).collect::<Vec<_>>(), ["notes.md:2 (the commit being made)"]);

    Git(&root, &["config", "user.email", "me@northwind.example.invalid"]);
    Git(&root, &["reset", "-q", "notes.md"]);
    let Verdict::Refused(found) = Judge_Commit_Being_Made(&judge, &git).expect("git answers") else { panic!("the party identity passed") };
    assert!(found.iter().all(|refusal| return refusal.rule == "identity"), "only the identity is refused once nothing is staged");
}

#[test]
fn Test_A_Push_Should_Judge_Only_What_The_Remote_Does_Not_Have()
{
    let scratch = Scratch("push");
    let root = Repository(&scratch);
    let remote = scratch.join("remote.git");
    Git(&scratch, &["init", "-q", "--bare", &remote.to_string_lossy()]);
    Git(&root, &["remote", "add", "origin", &remote.to_string_lossy()]);
    Write(&root, "old.md", "NW-12 from before the guard\n");
    Git(&root, &["add", "."]);
    Git(&root, &["commit", "-q", "-m", "old"]);
    Git(&root, &["push", "-q", "origin", "main"]);

    let policy = Policy();
    let judge = Judge::New(&policy);
    let launcher = SystemGit;
    let git = GitReader::New(&launcher, &root);
    let pushed = |root: &Path| return format!("refs/heads/main {} refs/heads/main {}\n", Git(root, &["rev-parse", "HEAD"]), Git(root, &["rev-parse", "origin/main"]));

    Write(&root, "new.md", "clean\n");
    Git(&root, &["add", "."]);
    Git(&root, &["commit", "-q", "-m", "new"]);
    assert_eq!(Judge_Push(&judge, &git, "origin", &pushed(&root)).expect("git answers"), Verdict::Clean, "published content is not judged again");

    Write(&root, "new.md", "clean\nnorthwind notes\n");
    Git(&root, &["add", "."]);
    Git(&root, &["commit", "-q", "-m", "sneaked past commit-msg with NW-99"]);
    let Verdict::Refused(found) = Judge_Push(&judge, &git, "origin", &pushed(&root)).expect("git answers") else { panic!("an unpublished commit passed") };
    let places: Vec<String> = found.iter().map(|refusal| return refusal.place.to_string()).collect();
    assert!(places.iter().any(|place| return place.starts_with("new.md:2 (commit ")), "the added line: {places:?}");
    assert!(places.iter().any(|place| return place.starts_with("message of commit ")), "the message: {places:?}");
}

/// A new branch has no remote id to exclude by, so what keeps history the remote already holds
/// from being judged again is the remote's own tracking refs.
#[test]
fn Test_A_New_Branch_Should_Not_Judge_History_The_Remote_Already_Holds()
{
    let scratch = Scratch("branch");
    let root = Repository(&scratch);
    let remote = scratch.join("remote.git");
    Git(&scratch, &["init", "-q", "--bare", &remote.to_string_lossy()]);
    Git(&root, &["remote", "add", "origin", &remote.to_string_lossy()]);
    Write(&root, "old.md", "NW-12 from before the guard\n");
    Git(&root, &["add", "."]);
    Git(&root, &["commit", "-q", "-m", "old"]);
    Git(&root, &["push", "-q", "origin", "main"]);
    Git(&root, &["switch", "-q", "-c", "feature"]);
    Write(&root, "feature.md", "clean\n");
    Git(&root, &["add", "."]);
    Git(&root, &["commit", "-q", "-m", "feature"]);

    let policy = Policy();
    let judge = Judge::New(&policy);
    let launcher = SystemGit;
    let git = GitReader::New(&launcher, &root);
    let pushed = format!("refs/heads/feature {} refs/heads/feature 0000000000000000000000000000000000000000\n", Git(&root, &["rev-parse", "HEAD"]));
    assert_eq!(Judge_Push(&judge, &git, "origin", &pushed).expect("git answers"), Verdict::Clean, "origin/main's history is not judged again");
}

#[test]
fn Test_The_Partys_Own_Repository_Should_Be_Stood_Aside_For()
{
    let scratch = Scratch("own");
    let root = Repository(&scratch);
    Git(&root, &["remote", "add", "origin", "https://git.example.invalid/northwind/app.git"]);
    Write(&root, "notes.md", "northwind\n");
    Git(&root, &["add", "."]);
    let policy = Policy();
    let judge = Judge::New(&policy);
    let launcher = SystemGit;
    let git = GitReader::New(&launcher, &root);
    assert_eq!(Judge_Commit_Being_Made(&judge, &git).expect("git answers"), Verdict::OwnRepository);
}

#[test]
fn Test_A_Scan_Should_Count_What_It_Read_And_Find_What_A_Repository_Already_Holds()
{
    let scratch = Scratch("scan");
    let root = Repository(&scratch);
    Write(&root, "northwind-notes.md", "see NW-12\n");
    Git(&root, &["add", "."]);
    Git(&root, &["commit", "-q", "-m", "held"]);
    let policy = Policy();
    let judge = Judge::New(&policy);
    let launcher = SystemGit;
    let git = GitReader::New(&launcher, &root);
    let outcome = Scan(&judge, &git, &SystemFiles, &root).expect("git answers");
    assert_eq!(outcome.judged, 2, "both tracked text files were read");
    let rules: Vec<&str> = outcome.refusals.iter().map(|refusal| return refusal.rule.as_str()).collect();
    assert_eq!(rules, ["party-name", "ticket"], "the file name and the ticket inside it");
}

/// Runs the real `git`.
struct SystemGit;

impl Strategy for SystemGit
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::None;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::NotApplicable;
}

impl ProgramLauncher for SystemGit
{
    fn Run(&self, command: &Command) -> Result<ProgramOutput, String>
    {
        let (program, arguments) = command.argv.split_first().ok_or("empty command")?;
        let mut process = Process::new(program);
        process.args(arguments);
        if let Some(directory) = &command.working_directory
        {
            process.current_dir(directory);
        }
        let output = process.output().map_err(|error| return error.to_string())?;
        let code = output.status.code().unwrap_or(-1);
        return Ok(ProgramOutput {
            outcome: ExitOutcome::Exited { code },
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
}

/// Reads the real file system.
struct SystemFiles;

impl Strategy for SystemFiles
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::None;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::NotApplicable;
}

impl FileSystem for SystemFiles
{
    fn Read_To_String(&self, path: &Path) -> Result<String, FileSystemError>
    {
        return std::fs::read_to_string(path).map_err(|error| return FileSystemError::Other { path: path.display().to_string(), cause: error.to_string() });
    }

    fn Replace_Atomically(&self, path: &Path, contents: &str) -> Result<(), FileSystemError>
    {
        return std::fs::write(path, contents).map_err(|error| return FileSystemError::Other { path: path.display().to_string(), cause: error.to_string() });
    }

    fn Exists(&self, path: &Path) -> bool
    {
        return path.exists();
    }
}
