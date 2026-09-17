use super::*;
use nomos_model::Subject_Of_Path;
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProgramLauncher};

/// A file carrying no claim at all: neither a phantom mirror to strike, nor a line of
/// trailing whitespace to strip.
const CLEAN_FILE: &str = "pub fn Something() -> u32 { return 1; }\n";

/// A universe declaring a mirror for a check that does not exist -- a real Blocking Phantom
/// finding, and the one line a real correction strikes, the second line below.
const PHANTOM_FIXTURE: &str = concat!(
    "/// A list of things this crate owns.\n",
    "/// Mirrored by `Test_Nonexistent_Check_That_Does_Not_Exist`.\n",
    "pub const THINGS: &[&str] = &[\"a\"];\n",
);

/// [`PHANTOM_FIXTURE`] with its own false claim struck, and nothing else changed.
const PHANTOM_CORRECTED: &str = "/// A list of things this crate owns.\npub const THINGS: &[&str] = &[\"a\"];\n";

/// Two lines of one file ending in a space or a tab -- two real `no-trailing-whitespace`
/// findings over one file, the population that makes batching visible at all.
const TRAILING_WHITESPACE_FIXTURE: &str = concat!(
    "pub fn Something() -> u32 \n",
    "{\n",
    "    return 1; \t\n",
    "}\n",
);

/// [`TRAILING_WHITESPACE_FIXTURE`] with both of its flagged lines stripped, and nothing else
/// changed.
const TRAILING_WHITESPACE_CORRECTED: &str = "pub fn Something() -> u32\n{\n    return 1;\n}\n";

#[test]
fn Test_An_Unwalked_Root_Should_Be_Unreadable()
{
    let command = CorrectionCommand { root: "does/not/exist".into(), commit: false };
    let outcome = Run_Correction(None, CorrectionEnvironment { variant: Test_Variant(), launcher: &StdProgramLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &command);

    assert_eq!(outcome, CorrectionOutcome::UnreadableRoot);
}

#[test]
fn Test_An_Empty_Walk_Should_Report_No_Source_Found()
{
    let command = CorrectionCommand { root: "irrelevant".into(), commit: false };
    let outcome = Run_Correction(Some(Vec::new()), CorrectionEnvironment { variant: Test_Variant(), launcher: &StdProgramLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &command);

    assert_eq!(outcome, CorrectionOutcome::NoSourceFound);
}

#[test]
fn Test_A_Tree_With_Neither_Claim_Should_Be_Clean()
{
    let fixture = Fixture::Of("nomos-correction-orchestration-clean-tree").Holding(CLEAN_FILE);

    let outcome = fixture.Ran();

    fixture.Remove();
    assert_eq!(outcome, CorrectionOutcome::Clean);
}

#[test]
fn Test_A_Dry_Run_Should_Stage_And_Validate_Without_Touching_Disk()
{
    let fixture = Fixture::Of("nomos-correction-orchestration-dry-run").Holding(PHANTOM_FIXTURE);

    Assert_Dry_Run(fixture, "Test_Nonexistent_Check_That_Does_Not_Exist");
}

#[test]
fn Test_Committing_Should_Strike_The_Claim_On_Disk_And_Leave_A_Clean_Rerun()
{
    let fixture = Fixture::Of("nomos-correction-orchestration-commit").Holding(PHANTOM_FIXTURE).Committing();

    Assert_Committed_And_Rerun(fixture, PHANTOM_CORRECTED);
}

#[test]
fn Test_A_Trailing_Whitespace_Dry_Run_Should_Batch_Every_Flagged_Line_In_One_Candidate()
{
    let fixture = Fixture::Of("nomos-correction-orchestration-whitespace-dry-run").Holding(TRAILING_WHITESPACE_FIXTURE);

    Assert_Dry_Run(fixture, "2 line(s)");
}

#[test]
fn Test_Committing_A_Trailing_Whitespace_Fix_Should_Strip_Every_Flagged_Line_At_Once()
{
    let fixture = Fixture::Of("nomos-correction-orchestration-whitespace-commit").Holding(TRAILING_WHITESPACE_FIXTURE).Committing();

    Assert_Committed_And_Rerun(fixture, TRAILING_WHITESPACE_CORRECTED);
}

/// A fresh temporary directory and the command that runs the pipeline over it -- the state
/// every fixture-backed test above builds in two steps, [`Fixture::Of`] and then
/// [`Fixture::Holding`], so that the four of them cannot drift apart.
struct Fixture
{
    root: std::path::PathBuf,
    /// Where this fixture's one source file lives: `a.rs`, under `root`.
    path: std::path::PathBuf,
    /// What [`Fixture::Holding`] last wrote to `path`, and so what a dry run must leave
    /// there untouched. Empty until that call.
    written: String,
    command: CorrectionCommand,
}

impl Fixture
{
    /// A fixture rooted in a fresh, empty temporary directory named `name`, with no source
    /// file in it yet.
    fn Of(name: &str) -> Fixture
    {
        let root = std::env::temp_dir().join(name);
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("the temporary root is creatable");
        let path = root.join("a.rs");
        let command = CorrectionCommand { root: root.clone(), commit: false };

        return Fixture { root, path, written: String::new(), command };
    }

    /// The same fixture, its `a.rs` now holding `content`.
    fn Holding(mut self, content: &str) -> Fixture
    {
        std::fs::write(&self.path, content).expect("the temporary root that Of created holds this file");
        self.written = content.to_owned();
        return self;
    }

    /// The same fixture, running the pipeline with `--commit` rather than as a dry run. A
    /// caller writes this rather than passing a `bool`, so the two states are named at the
    /// one place they are decided.
    fn Committing(mut self) -> Fixture
    {
        self.command.commit = true;
        return self;
    }

    /// Runs the real pipeline over this fixture's own tree, exactly as a composition root
    /// hands it over: the walk already done, and the outcome already decided.
    fn Ran(&self) -> CorrectionOutcome
    {
        return Run_Correction(self.Walk(), CorrectionEnvironment { variant: Test_Variant(), launcher: &StdProgramLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &self.command);
    }

    /// The walk a composition root would have handed [`Run_Correction`] for this fixture's
    /// tree: every `.rs` file it holds, read and named relative to this fixture's root.
    fn Walk(&self) -> Option<Vec<SourceFile>>
    {
        let mut sources = Vec::new();
        for entry in std::fs::read_dir(&self.root).ok()?.flatten()
        {
            let path = entry.path();
            if path.extension().is_some_and(|extension| return extension == "rs")
            {
                let text = std::fs::read_to_string(&path).expect("this path came from the read_dir above, so the read succeeds");
                let relative = path.strip_prefix(&self.root).expect("every entry read_dir hands back is under the root it was given").display().to_string();
                let source = SourceFile::New(relative.clone(), Subject_Of_Path(&relative), text);
                sources.push(source);
            }
        }

        return Some(sources);
    }

    /// What this fixture's `a.rs` holds on disk right now.
    fn Read(&self) -> String
    {
        return std::fs::read_to_string(&self.path).expect("the fixture's own root holds this file until Remove is called");
    }

    /// Deletes this fixture's temporary tree.
    fn Remove(&self)
    {
        let _ignored = std::fs::remove_dir_all(&self.root);
    }
}

/// Asserts both halves of what a dry run promises: the staged report names `needle`, and
/// `a.rs` still holds exactly what [`Fixture::Holding`] wrote -- the shape both dry-run
/// tests above share.
fn Assert_Dry_Run(fixture: Fixture, needle: &str)
{
    let outcome = fixture.Ran();
    let on_disk = fixture.Read();

    fixture.Remove();
    Assert_Staged(outcome, needle);
    assert_eq!(on_disk, fixture.written, "a dry run must not touch the file");
}

/// Asserts `outcome` is a dry run's own staged report for `a.rs`, with `needle` named in its
/// summary.
fn Assert_Staged(outcome: CorrectionOutcome, needle: &str)
{
    match outcome
    {
        CorrectionOutcome::Staged { path: staged_path, summary, .. } =>
        {
            assert_eq!(staged_path, "a.rs");
            assert!(summary.contains(needle), "{summary}");
        }
        other => panic!("expected Staged, got {other:?}"),
    }
}

/// Asserts both halves of what a commit promises: `a.rs` now holds `corrected`, and a second
/// run over that corrected tree reports `Clean` rather than the same claim again -- the
/// shape both commit tests above share.
fn Assert_Committed_And_Rerun(fixture: Fixture, corrected: &str)
{
    let outcome = fixture.Ran();
    let after = fixture.Read();
    let second = fixture.Ran();

    fixture.Remove();
    Assert_Committed(outcome);
    assert_eq!(after, corrected, "only the candidate's own fix should have changed the file");
    assert_eq!(second, CorrectionOutcome::Clean, "a corrected file must not still carry the claim on a rerun");
}

/// Asserts `outcome` is the commit path's own report for `a.rs`.
fn Assert_Committed(outcome: CorrectionOutcome)
{
    match &outcome
    {
        CorrectionOutcome::Committed { path: committed_path, .. } => assert_eq!(committed_path, "a.rs"),
        other => panic!("expected Committed, got {other:?}"),
    }
}

/// The build variant every run in this module is handed -- named here rather than built at
/// each call site, so that all of them judge the same target the same way.
fn Test_Variant() -> BuildVariant
{
    return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
}
