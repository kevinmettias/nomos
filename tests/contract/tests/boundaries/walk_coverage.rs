//! The walk every host shares reaches every source this repository tracks.
//!
//! `OD-ANALYSIS-012` version 2 rules that a rule which judged an empty population never flips
//! a run's claim, and one of its reasons is that no population count can tell a repository
//! with no file of a language from a walker that dropped files which exist. That is right, and
//! it removes the one signal that would have hinted at a walker defect. A walk that silently
//! skipped a directory or an extension now reads as clean judgments plus a longer
//! empty-population list, with every test green, because the walk's own tests are
//! fixture-sized and nothing compared it with this repository.
//!
//! So this compares it. One side is `nomos_workspace_discovery::Walked_Sources` over this
//! repository's own root, called with the widest set of extensions any host walks with -- the
//! registered languages plus `check-script-discipline`'s scripts, read from that crate rather
//! than retyped. The other is `git ls-files`, filtered to those same extensions. Every tracked
//! source must be walked unless [`EXCLUSIONS`] declares, with a reason, why the walk is right
//! not to reach it.
//!
//! # What is deliberately not compared
//!
//! Files on disk that git does not track. This tree is shared with sessions whose in-flight
//! files are untracked, so a guard requiring the walk to agree with git about them would be
//! red for reasons nobody committed. The reverse direction is asserted over tracked files
//! only, for the same reason. A tracked file missing from the disk is left out too: the walk
//! can only read what is there, and a deletion nobody has staged yet is not its defect.
//!
//! # What it cannot see
//!
//! An extension this repository tracks no file of. At the commit that added this guard that
//! was `cs` and all five script extensions -- only `rs` and `go` had tracked files -- so a
//! walk that dropped `.cs` passes here, and the walker's own fixture tests are what hold it.
//! The guard grows a reach the day a tracked file of that kind arrives, with no edit here.
//!
//! # Why it fails rather than passes when it cannot look
//!
//! It reads this repository rather than an outside corpus, so CI runs it. If git cannot list
//! the tracked files, if the root is not a directory, or if git lists no tracked source at
//! all, it panics: each of those would otherwise compare an empty side and report a clean
//! walk, which is `OD-GATE-001`'s defect in its plainest form.

use nomos_contract_tests::Workspace;
use std::collections::BTreeSet;
use std::path::Path;

/// One tracked path the walk is meant not to reach, and why it is right not to.
struct Exclusion
{
    /// A directory or file, relative to the root, spelled with forward slashes as git spells it.
    path: &'static str,
    /// Why the walk does not reach it, quoted in every failure that names it.
    reason: &'static str,
}

/// Every tracked path the walk is meant not to reach.
///
/// Held both ways. A declared path the walk reaches fails, because the declaration then
/// claims a skip that no longer happens; a declared path that covers no tracked source fails,
/// because it has stopped excluding anything and would quietly excuse whatever arrives there
/// next.
const EXCLUSIONS: &[Exclusion] = &[Exclusion {
    path: "tests/integration/fixtures/third-party/hex-0.4.3",
    reason: "an unmodified excerpt of the hex crate carrying its own standards.json, which makes \
             it somebody else's repository root: it is judged against the conventions it \
             declares by tests/integration/tests/calibration.rs, and the walk's nested-root skip \
             exists so that a run of this repository never judges it as its own source",
}];

#[test]
fn Test_The_Walk_Should_Reach_Every_Source_The_Repository_Tracks()
{
    let root = Workspace::Workspace_Root();
    let expected = Expected_Sources(&Tracked_Files(&root), &Recognized_Extensions(), EXCLUSIONS, |path| {
        return root.join(path).is_file();
    });
    let walked = Walked_Paths(&root);
    let unwalked: Vec<&String> = expected.difference(&walked).collect();

    assert!(
        unwalked.is_empty(),
        "the walk over {} does not reach {} of the {} sources git tracks: {unwalked:#?}.\n\
         Every rule reads the walk, so a source it drops is judged by nothing, and since \
         OD-ANALYSIS-012 version 2 an empty population no longer hints at it. Either the walk \
         dropped these, which is a defect in nomos-workspace-discovery to board rather than \
         excuse, or the walk is right not to reach them, which belongs in EXCLUSIONS with the \
         reason.",
        root.display(),
        unwalked.len(),
        expected.len()
    );
}

#[test]
fn Test_The_Walk_Should_Reach_No_Tracked_Source_A_Declared_Exclusion_Covers()
{
    let root = Workspace::Workspace_Root();
    let tracked = Tracked_Files(&root);
    let walked = Walked_Paths(&root);
    let reached: Vec<String> = walked
        .intersection(&tracked)
        .filter_map(|path| return Covering_Exclusion(path, EXCLUSIONS).map(|exclusion| return Described(path, exclusion)))
        .collect();

    assert!(
        reached.is_empty(),
        "the walk reaches tracked sources that EXCLUSIONS declares it does not: {reached:#?}.\n\
         The declaration claims a skip that no longer happens. If the walk is now meant to reach \
         them, remove the exclusion; if it is not, the walk lost the skip."
    );
}

#[test]
fn Test_Every_Declared_Exclusion_Should_Cover_A_Source_The_Repository_Tracks()
{
    let root = Workspace::Workspace_Root();
    let sources = Expected_Sources(&Tracked_Files(&root), &Recognized_Extensions(), &[], |_| return true);
    let stale: Vec<String> = EXCLUSIONS
        .iter()
        .filter(|exclusion| return !sources.iter().any(|path| return Is_Beneath(path, exclusion.path)))
        .map(|exclusion| return format!("{} ({})", exclusion.path, exclusion.reason))
        .collect();

    assert!(
        stale.is_empty(),
        "these exclusions cover no source git tracks: {stale:#?}.\n\
         An exclusion that excludes nothing would excuse whatever arrives at that path next \
         without anyone deciding it should. Delete it, or correct the path it names."
    );
}

#[test]
fn Test_The_Walk_Should_Reach_No_Tracked_File_Whose_Extension_It_Does_Not_Recognize()
{
    let root = Workspace::Workspace_Root();
    let tracked = Tracked_Files(&root);
    let recognized = Recognized_Extensions();
    let walked = Walked_Paths(&root);
    let unrecognized: Vec<&String> = walked
        .intersection(&tracked)
        .filter(|path| return !Has_Recognized_Extension(path, &recognized))
        .collect();

    assert!(
        unrecognized.is_empty(),
        "the walk reaches tracked files whose extension is not among {recognized:?}: \
         {unrecognized:#?}.\nEvery rule reads the walk, so a file it was not asked for is judged \
         as source it is not."
    );
}

/// The tracked sources the walk is expected to reach: those with an extension it recognizes,
/// beneath no declared exclusion, and present on the disk it reads.
///
/// Over its inputs rather than over this repository, so the controls below can show each of
/// the three filters removing what it is for.
fn Expected_Sources(
    tracked: &BTreeSet<String>,
    recognized: &[&str],
    exclusions: &[Exclusion],
    present: impl Fn(&str) -> bool,
) -> BTreeSet<String>
{
    return tracked
        .iter()
        .filter(|path| return Has_Recognized_Extension(path, recognized))
        .filter(|path| return Covering_Exclusion(path, exclusions).is_none())
        .filter(|path| return present(path))
        .cloned()
        .collect();
}

/// Every extension the widest host walk recognizes: the registered languages and the scripts.
///
/// The set `nomos gate run` and `nomos check` walk with, so a source either of them would judge
/// is a source this guard expects. Read from the walk's own crate, as the hosts read it.
fn Recognized_Extensions() -> Vec<&'static str>
{
    let mut recognized = nomos_workspace_discovery::Registered_Extensions();
    recognized.extend(nomos_workspace_discovery::SCRIPT_EXTENSIONS);
    return recognized;
}

/// Whether `path` carries one of `recognized`, judged the way the walk judges it: the final
/// extension, compared exactly.
fn Has_Recognized_Extension(path: &str, recognized: &[&str]) -> bool
{
    return Path::new(path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|extension| return recognized.contains(&extension));
}

/// The declared exclusion `path` falls beneath, if any.
fn Covering_Exclusion<'declared>(path: &str, exclusions: &'declared [Exclusion]) -> Option<&'declared Exclusion>
{
    return exclusions.iter().find(|exclusion| return Is_Beneath(path, exclusion.path));
}

/// Whether `path` is `excluded` itself or lies beneath it as a directory.
///
/// On a whole path component, so that excluding `hex-0.4.3` does not also exclude a sibling
/// named `hex-0.4.30`.
fn Is_Beneath(path: &str, excluded: &str) -> bool
{
    return path == excluded || path.strip_prefix(excluded).is_some_and(|rest| return rest.starts_with('/'));
}

/// A walked path and the exclusion it contradicts, for a failure message.
fn Described(path: &str, exclusion: &Exclusion) -> String
{
    return format!("{path}, declared out of reach by {} because {}", exclusion.path, exclusion.reason);
}

/// Every file git tracks under `root`, relative to it with forward slashes.
///
/// The index rather than `HEAD`, as `agent_harness`'s reader takes it and for its reason: a
/// source staged in the same change that the walk must reach is judged in that change.
///
/// # Panics
///
/// If git cannot be run, refuses, or lists nothing. Each would leave one side of the
/// comparison empty, and an empty side agrees with any walk at all.
fn Tracked_Files(root: &Path) -> BTreeSet<String>
{
    let output = std::process::Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(root)
        .output()
        .unwrap_or_else(|error| panic!("cannot run git to list the tracked files: {error}"));
    assert!(
        output.status.success(),
        "git could not list the tracked files in {} ({}). This guard compares the walk with \
         what the repository tracks, so without git it has nothing to compare against.",
        root.display(),
        String::from_utf8_lossy(&output.stderr).trim()
    );
    let tracked: BTreeSet<String> =
        String::from_utf8_lossy(&output.stdout).split('\0').filter(|path| return !path.is_empty()).map(str::to_owned).collect();
    assert!(!tracked.is_empty(), "git lists no tracked file in {}, so this is not the repository the guard is about", root.display());

    return tracked;
}

/// Every path the shared walk reaches under `root`, as the walk reports it.
///
/// # Panics
///
/// If `root` is not a directory, which the walk answers with `None` rather than an empty list.
fn Walked_Paths(root: &Path) -> BTreeSet<String>
{
    let sources = nomos_workspace_discovery::Walked_Sources(root, &Recognized_Extensions())
        .unwrap_or_else(|| panic!("{} is not a directory, so the walk read nothing to compare", root.display()));

    return sources.into_iter().map(|source| return source.path).collect();
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Paths(paths: &[&str]) -> BTreeSet<String>
    {
        return paths.iter().map(|path| return (*path).to_owned()).collect();
    }

    #[test]
    fn Test_An_Exclusion_Should_Cover_Only_Itself_And_What_Lies_Beneath_It()
    {
        assert!(Is_Beneath("vendored/hex-0.4.3/lib.rs", "vendored/hex-0.4.3"));
        assert!(Is_Beneath("vendored/hex-0.4.3", "vendored/hex-0.4.3"));
        assert!(!Is_Beneath("vendored/hex-0.4.30/lib.rs", "vendored/hex-0.4.3"));
        assert!(!Is_Beneath("vendored/lib.rs", "vendored/hex-0.4.3"));
    }

    /// Each filter removes the one path it is for and nothing else.
    #[test]
    fn Test_The_Expected_Sources_Should_Leave_Out_Only_What_Each_Filter_Is_For()
    {
        let tracked = Paths(&["src/kept.rs", "README.md", "src/kept.rs.orig", "vendored/foreign.rs", "src/deleted.rs"]);
        let exclusions = [Exclusion { path: "vendored", reason: "a control" }];

        let expected = Expected_Sources(&tracked, &["rs"], &exclusions, |path| return path != "src/deleted.rs");

        assert_eq!(expected, Paths(&["src/kept.rs"]));
    }
}
