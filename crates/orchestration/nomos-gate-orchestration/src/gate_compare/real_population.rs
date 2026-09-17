//! The one test that judges `compare` against the findings this repository's own rules really
//! emit, and the walk that collects them.
//!
//! Split out of `gate_compare.rs` when that file passed the workspace's own 500-line review
//! trigger: a walk of the filesystem and the identity claim made about its result are a
//! responsibility of their own, and the tests above them are about fixtures.

use super::tests::Run_Id_Of;
use crate::{GateCommand, GateEnvironment, GateRunResult, Run_Gate};
use nomos_contracts::{Finding, RunId};
use nomos_model::{FindingOccurrenceId, Occurrence_Collisions_In};
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProgramLauncher};
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;
use std::collections::BTreeSet;

/// The identity the real-tree run is recorded under.
const REAL_TREE_RUN: RunId = Run_Id_Of(9);

/// How many levels below the workspace root this crate's own manifest directory sits.
const WORKSPACE_DEPTH: usize = 3;

/// The fewest findings a real run must emit for the walk to be evidence of anything.
const REAL_POPULATION_FLOOR: usize = 50;

/// Every finding a real run of this repository's own rules produces gets its own identity.
///
/// # Why a real population and not a bigger fixture
///
/// Because the property is about the findings this workspace's rules actually emit, and a
/// fixture proves only that the author of the fixture believed it. The measured shape before
/// this increment was 253 findings reduced to 150 keys; the claim afterwards is that the
/// reduction is gone, and only real findings can carry that claim.
#[test]
fn Test_Every_Finding_Of_A_Real_Run_Should_Get_Its_Own_Identity()
{
    let findings = Real_Findings();

    Assert_One_Identity_Each(&findings);
}

/// Every finding a real run over this repository's own source produces.
///
/// # Why the walk is here rather than passed in
///
/// Because this crate does not walk. [`Run_Gate`] takes an already-walked source set and
/// reads `None` as *unreadable*, not as *go and look* -- the division that keeps a walk in
/// the composition root. So a test that wants a real population has to do what `nomos-cli`
/// and `nomos-api` do, and collect one.
///
/// The whole of `crates/` is collected rather than one crate's worth: a cross-file rule
/// handed a truncated world answers a different question and labels it the same, which
/// `OD-GATE-025` records and which this repository has already been bitten by.
fn Real_Findings() -> Vec<Finding>
{
    let root = Workspace_Root();
    let sources = Rust_Under(&root, &root);

    assert!(!sources.is_empty(), "no source collected under {}", root.display());

    let collected = sources.len();
    let command = GateCommand { root: root.clone(), ..GateCommand::default() };
    let environment = GateEnvironment {
        variant: BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>()),
        launcher: &StdProgramLauncher,
        filesystem: &StdFileSystem,
        environment: &StdEnvironment,
        now: nomos_platform::Timestamp::From_Unix_Seconds(0),
    };
    let result = Run_Gate(Some(sources), environment, &command, REAL_TREE_RUN);

    return Judged_Findings(result, collected, &root);
}

/// The workspace root this crate's own sources sit under.
///
/// The workspace root, not `crates/`: the check beneath a run resolves a workspace from the
/// root it is given, and a directory with no manifest above it comes back `Unreadable` --
/// which is an empty finding list, which is a vacuous pass. Climbed by removing components
/// rather than joined with `..`, so the spellings handed to the ingest are the ones a real
/// caller would pass. A root carrying `..` segments reaches the subject derivation as a
/// different spelling of the same file, and this repository has already recorded what a path
/// spelling mismatch does to matching.
fn Workspace_Root() -> std::path::PathBuf
{
    let mut root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    for _ in 0..WORKSPACE_DEPTH
    {
        root.pop();
    }

    return root;
}

/// Every `.rs` file under `directory`, as sources a run can judge, spelled relative to `root`.
///
/// # Why relative and forward-slashed
///
/// Because that is what a source path is in this system, and neither half is cosmetic. A
/// `Finding`'s locations are documented repo-relative with forward slashes, and the workspace
/// ingest validates every path it is handed: an absolute Windows spelling carries a
/// drive-letter segment the ingest will not name, and the whole change set is refused rather
/// than half-applied. A test that passed absolute paths got `CheckOutcome::Unreadable` and an
/// empty finding list -- which is to say, a vacuous pass, if the assertion above had not been
/// there to catch it.
fn Rust_Under(directory: &std::path::Path, root: &std::path::Path) -> Vec<SourceFile>
{
    let mut sources = Vec::new();
    let mut pending = vec![directory.to_path_buf()];

    while let Some(next) = pending.pop()
    {
        let entries = Children_Of(&next);
        pending.extend(entries.directories);

        for path in entries.files
        {
            if let Some(source) = Source_Of(&path, root)
            {
                sources.push(source);
            }
        }
    }

    return sources;
}

/// The subdirectories of `directory` worth walking, and the files directly inside it.
///
/// Split from the walk so neither half nests: sorting entries into two kinds and deciding what
/// each kind is are two jobs, and doing both in one loop is what `nesting-depth` reported
/// here. An unreadable directory yields nothing rather than failing the walk -- a tree this
/// test cannot fully read still answers the question it asks, and the floor assertion above is
/// what stops a half-read tree passing as a clean one.
fn Children_Of(directory: &std::path::Path) -> DirectoryEntries
{
    let mut entries = DirectoryEntries { directories: Vec::new(), files: Vec::new() };

    let Ok(read) = std::fs::read_dir(directory)
    else
    {
        return entries;
    };

    for entry in read.flatten()
    {
        entries.Add(entry.path());
    }

    return entries;
}

/// The two kinds of entry one directory holds, kept apart so a caller names which it means.
///
/// A named result rather than a tuple of two `Vec<PathBuf>`s: the caller must otherwise
/// remember the order, and the compiler cannot help -- swap the two members and the walk still
/// builds, ships, and is found by a human staring at a wrong answer.
struct DirectoryEntries
{
    directories: Vec<std::path::PathBuf>,
    files: Vec<std::path::PathBuf>,
}

impl DirectoryEntries
{
    /// Files `path` under the kind of entry it is.
    fn Add(&mut self, path: std::path::PathBuf)
    {
        if !path.is_dir()
        {
            self.files.push(path);
            return;
        }

        // `target/` holds the compiled world, including every dependency's own source, which
        // is neither this repository's code nor something its rules have any business judging.
        // Walking it would also take minutes.
        let name = path.file_name();

        if !name.is_some_and(|name| return name == "target" || name == ".git" || name == "node_modules")
        {
            self.directories.push(path);
        }
    }
}

/// `path` as a source a run can judge, when it is Rust this walk can read.
fn Source_Of(path: &std::path::Path, root: &std::path::Path) -> Option<SourceFile>
{
    if !path.extension().is_some_and(|extension| return extension.eq_ignore_ascii_case("rs"))
    {
        return None;
    }

    let text = std::fs::read_to_string(path).ok()?;
    let relative = path.strip_prefix(root).unwrap_or(path);
    let spelling = relative.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");

    return Some(SourceFile::New(&spelling, nomos_model::Subject_Of_Path(&spelling), text));
}

/// The judged population `result` produced, refused when the run never reached a judgment.
///
/// A run that never reached a judgment would report an empty finding list, and an empty
/// finding list trivially has no colliding identities. That is the vacuous pass this whole
/// module exists to refuse, and the outcome is the only thing that distinguishes it from a
/// real clean result.
///
/// The judged population, not the four reduced buckets: the gate composes four rules and the
/// check beneath it runs the whole registry, so the buckets hold single digits while the run
/// *emits* hundreds -- and "as many identities as findings emitted" is a claim about what the
/// analysis produced, not about the slice this crate's own policy kept. The 253-to-150
/// collapse this increment removes was measured over exactly this set.
fn Judged_Findings(result: GateRunResult, collected: usize, root: &std::path::Path) -> Vec<Finding>
{
    assert!(
        matches!(result.check_outcome, nomos_check_orchestration::CheckOutcome::Judged { .. }),
        "{collected} source(s) collected under {} and the run still did not judge: {:?}",
        root.display(),
        result.check_outcome
    );

    let nomos_check_orchestration::CheckOutcome::Judged { findings, .. } = result.check_outcome
    else
    {
        // The assertion above is this branch's only reachability guard: a run that never
        // judged has already been refused, so no half-read tree reaches here as a clean list.
        return Vec::new();
    };

    return findings;
}

/// Asserts that `findings` came back at a size worth reasoning about, and that no two of them
/// reach one occurrence identity.
///
/// The floor is asserted, not the exact count. Peers edit this tree while tests run, so an
/// exact number would be a flake rather than a stronger claim; a floor plus zero collisions is
/// the property, and the floor is what stops an empty walk from passing as a clean result --
/// the vacuity failure this repository's own records name repeatedly.
fn Assert_One_Identity_Each(findings: &[Finding])
{
    assert!(
        findings.len() >= REAL_POPULATION_FLOOR,
        "only {} finding(s) came back from a real run, which is too few to be evidence of \
         anything. A clean result over an empty population is the lie this assertion exists \
         to refuse.",
        findings.len()
    );

    let borrowed: Vec<&Finding> = findings.iter().collect();
    let collisions = Occurrence_Collisions_In(&borrowed);
    let identities: BTreeSet<FindingOccurrenceId> = findings.iter().map(FindingOccurrenceId::Of).collect();

    assert!(collisions.is_empty(), "{} colliding pair(s): {collisions:?}", collisions.len());
    assert_eq!(
        identities.len(),
        findings.len(),
        "the number of findings entering a comparison must equal the number emitted; \
         {} findings produced {} identities",
        findings.len(),
        identities.len()
    );
}
