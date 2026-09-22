//! Zone: Application Service — the one walk every composition root shares.
//!
//! Measured directly at `c0e5c79f`: four composition roots (`nomos-api`, `nomos-cli`'s
//! `check` and `gate`, `nomos-lsp`) each carried their own copy of the same walk -- an
//! explicit directory stack over `std::fs`, `target`/`.git` and nested-root
//! exclusion, and a hand-typed `"rs"`/`"go"` extension check -- each citing `OD-HOST-002`'s
//! family 3 ("the directory walk itself stays a composition-root concern") as why sharing
//! it was declined. `OD-HOST-008` reverses that: the walk mechanics were never the
//! composition-root-specific half of that decision, only *which* extensions the walk kept
//! ever varied per caller, and even that varied by a fixed, repeated literal
//! (`nomos-lsp` alone omitted `check-script-discipline`'s five script extensions -- a real
//! behavioral gap this crate does not paper over, since [`Registered_Extensions`] answers
//! only what a registered language package recognizes, the same population every host's
//! literal `"rs" || "go"` check already named).
//!
//! [`Registered_Extensions`] is what "recognition comes from registered packages" means in
//! this crate: [`nomos_lang_rust::RUST_EXTENSION`] and [`nomos_lang_go::GO_EXTENSION`],
//! read from those crates rather than retyped -- the one edit a third language package
//! costs, instead of a fifth host-side copy of a literal. [`Walked_Sources`] takes the
//! recognized set as its own parameter rather than hard-coding
//! [`Registered_Extensions`]'s answer, because a caller that also needs
//! `check-script-discipline`'s script extensions (three of the four hosts do; `nomos-lsp`
//! does not) composes its own wider set rather than this crate inventing a second
//! registration surface for a rule's own applicability data, which is not a language
//! package or provider and is out of this record's scope.
//!
//! [`WorkspaceProfile`] is the second thing this crate answers, built on the first: what a
//! root contains before anything judges it -- how many sources of each registered language
//! the walk finds, which language manifests and which repository policy files sit at the
//! root, and whether the root carries the marker the walk already uses. A pure function of
//! the walk plus existence checks at the root, so it lives here rather than in the host that
//! will render it. `P123-WORKSPACE-PROFILE-FOR-ADOPTION`.

#![forbid(unsafe_code)]

use nomos_model::Subject_Of_Path;
use nomos_rules::SourceFile;
use std::path::{Path, PathBuf};

mod language_manifest;
mod policy_file;
mod policy_file_presence;
mod profile_refusal;
mod source_count;
mod workspace_profile;

pub use language_manifest::LanguageManifest;
pub use policy_file::PolicyFile;
pub use policy_file_presence::PolicyFilePresence;
pub use profile_refusal::ProfileRefusal;
pub use source_count::SourceCount;
pub use workspace_profile::WorkspaceProfile;

/// The file a repository declares its own conventions in, and so the mark of a repository
/// root.
///
/// A literal here rather than `nomos_repo_policy`'s own `STANDARDS_JSON`, and the reason is
/// not reach: `nomos-repo-policy` keeps `standards_document` a private module, because
/// `OD-RULES-019` and `OD-PACKAGE-015` put the shared read beneath its five providers rather
/// than beside them, so the constant is not exported and making it so would widen that
/// crate's public surface to publish a filename. This walk never reads the file. It asks
/// whether one is there, which is the same distinction `OD-HOST-008` already drew for the
/// extension check: this walk decides which directories are somebody else's root, not how a
/// root's conventions are parsed. `nomos_rules::checks::goals`'s own `DECLARATION_FILE`
/// carries the identical literal for the identical reason.
const ROOT_MARKER: &str = "standards.json";

/// Every extension a registered language package recognizes.
///
/// The one edit adding a third language package costs this crate, instead of a fifth host
/// carrying its own hard-coded literal -- `OD-HOST-008`'s own "adding a language changes no
/// host" clause.
#[must_use]
pub fn Registered_Extensions() -> Vec<&'static str>
{
    return vec![nomos_lang_rust::RUST_EXTENSION, nomos_lang_go::GO_EXTENSION];
}

/// `check-script-discipline`'s own five recognized script extensions --
/// `standards.json`'s `forbidden_extensions`, the same set a real shebang script in this
/// repository would carry. Not a registered language package's recognition -- a rule's own
/// applicability data instead, so kept separate from [`Registered_Extensions`] rather than
/// folded into it. Named here once instead of duplicated per caller: three of this
/// workspace's four hosts (`nomos-api`, `nomos-cli`'s `check` and `gate`) already needed the
/// identical literal; `nomos-lsp` alone does not walk for it, unchanged by this crate's own
/// arrival.
pub const SCRIPT_EXTENSIONS: [&str; 5] = ["sh", "ps1", "psm1", "bat", "cmd"];

/// Every file under `root` whose extension `recognized` names, with its text and the
/// subject its facts are filed under -- or `None` if `root` is not a directory.
///
/// A directory that is walked and turns out empty is not this function's decision: a
/// caller's own outcome type (`nomos_check_orchestration::CheckOutcome`, for one) is where
/// "not a directory" and "found nothing" become distinguishable typed answers.
#[must_use]
pub fn Walked_Sources(root: &Path, recognized: &[&str]) -> Option<Vec<SourceFile>>
{
    if !root.is_dir()
    {
        return None;
    }

    return Some(Read_Sources(root, recognized));
}

/// Every file under `root` whose extension `recognized` names, with its text and the
/// subject its facts are filed under.
///
/// `target` and `.git` are skipped: the first holds generated source nobody authored, and
/// the second is not source at all. So are two kinds of directory that are somebody else's
/// root rather than a part of this one -- see [`Is_A_Nested_Git_Worktree`] and
/// [`Is_A_Nested_Repository_Root`].
#[must_use]
pub fn Read_Sources(root: &Path, recognized: &[&str]) -> Vec<SourceFile>
{
    let mut sources = Vec::new();
    let mut pending = vec![root.to_path_buf()];

    while let Some(directory) = pending.pop()
    {
        let Ok(entries) = std::fs::read_dir(&directory)
        else
        {
            continue;
        };

        for entry in entries.flatten()
        {
            let path = entry.path();
            Read_Entry(root, path, recognized, &mut Collected { pending: &mut pending, sources: &mut sources });
        }
    }

    sources.sort_by(|left, right| return left.path.cmp(&right.path));
    return sources;
}

/// [`Read_Sources`]'s own accumulator: the directories still to walk and the sources found
/// so far, grouped into one value so [`Read_Entry`] stays within this crate's own
/// parameter-count limit.
struct Collected<'a>
{
    pending: &'a mut Vec<PathBuf>,
    sources: &'a mut Vec<SourceFile>,
}

/// One entry of a walked directory: queued if it is a directory worth descending into, read
/// if `recognized` names its extension, and ignored otherwise.
fn Read_Entry(root: &Path, path: PathBuf, recognized: &[&str], collected: &mut Collected<'_>)
{
    if path.is_dir()
    {
        let skipped = path.file_name().is_some_and(|name| return name == "target" || name == ".git")
            || Is_A_Nested_Git_Worktree(&path)
            || Is_A_Nested_Repository_Root(&path);

        if !skipped
        {
            collected.pending.push(path);
        }

        return;
    }

    if path.extension().and_then(std::ffi::OsStr::to_str).is_some_and(|extension| return recognized.contains(&extension))
        && let Ok(text) = std::fs::read_to_string(&path)
    {
        let source = Read_Source(root, &path, text);
        collected.sources.push(source);
    }
}

/// Whether `path` is itself a git worktree's own root — a directory whose immediate `.git`
/// entry is a file (naming another repository's own `.git/worktrees/<name>` directory)
/// rather than the ordinary `.git` directory a real checkout has. A linked worktree checked
/// out under an already-walked root is a second copy of a tree this walk must not descend
/// into, the same reason `target` already is: it is not source anyone at this address
/// authored, it is another checkout of code that may be stale or from a different point in
/// this repository's own history — `P67-SELF-CHECK-WALK-CROSSES-NESTED-WORKTREE-BOUNDARY-2`
/// measured exactly this against a leftover build-agent worktree left under
/// `.claude/worktrees/`.
fn Is_A_Nested_Git_Worktree(path: &Path) -> bool
{
    return path.join(".git").is_file();
}

/// Whether `path` is itself a repository root -- a directory carrying its own
/// `standards.json`, which is the file a run reads a repository's declared conventions from.
///
/// Skipped for the reason [`Is_A_Nested_Git_Worktree`] is skipped, one step further out: it
/// is not source anyone at *this* address authored. A vendored third-party tree is judged
/// against the conventions it declares or it is not judged here at all, and the enclosing
/// root's conventions are not a claim about it.
///
/// `P45-RULES-CALIBRATED-AGAINST-CODE-THEY-WERE-NOT-TUNED-ON` is what made this concrete
/// and is also what makes it measurable. It committed an unmodified excerpt of `hex` 0.4.3
/// under `tests/integration/fixtures/third-party/`, with a `standards.json` beside it
/// declaring that crate's own real convention, so the composed rule set could be measured
/// against code this repository did not write. `tests/integration/tests/calibration.rs`
/// measured that declaration working -- 13 `function-naming-convention` findings without it,
/// 0 with it. This walk reached the same files from the repository root, read the
/// repository's own `standards.json` instead, and reported those 13 back with a reason
/// citing this repository's README at somebody else's crate. Two more of its findings could
/// fail a build, which is how a fixture whose whole purpose is to produce findings came to
/// be the reason `nomos gate run --root .` was red.
///
/// The root of the run is not reachable here: [`Read_Sources`] seeds it directly and only
/// entries below it are judged, so a repository's own marker configures it rather than
/// excluding it. `Test_Read_Sources_Should_Not_Descend_Into_A_Nested_Repository_Root` holds
/// both halves of that.
fn Is_A_Nested_Repository_Root(path: &Path) -> bool
{
    return Carries_Root_Marker(path);
}

/// Whether `directory` holds its own [`ROOT_MARKER`] as a file -- the one test both the
/// walk and [`WorkspaceProfile`] make of a directory, so the profile's "this is a
/// repository root" and the walk's "that is somebody else's" cannot come to differ about
/// which file decides.
pub(crate) fn Carries_Root_Marker(directory: &Path) -> bool
{
    return directory.join(ROOT_MARKER).is_file();
}

/// One source file as a caller takes it.
///
/// This root files a fact under the subject and hands the same value on
/// `SourceFile::subject`, so the two cannot disagree about addressing -- the kernel's rule,
/// not a local one. `OD-MODEL-001`.
#[must_use]
pub fn Read_Source(root: &Path, path: &Path, text: String) -> SourceFile
{
    let relative = Relative_Path(root, path);
    let subject = Subject_Of_Path(&relative);

    return SourceFile::New(relative, subject, text);
}

/// A path as it should be reported: relative to the tree, forward slashes.
///
/// Forward slashes on every platform, because a finding's location appears in output that
/// gets pasted between machines, and the same file must not render two ways.
#[must_use]
pub fn Relative_Path(root: &Path, path: &Path) -> String
{
    return path.strip_prefix(root).unwrap_or(path).display().to_string().replace('\\', "/");
}

#[cfg(test)]
mod test_support;

#[cfg(test)]
mod tests
{
    use super::{Read_Source, Read_Sources, Registered_Extensions, Relative_Path, ROOT_MARKER, Walked_Sources, SCRIPT_EXTENSIONS};
    use crate::test_support::{Fresh_Root, Make_Fixture_Directory, Write_Fixture};
    use std::path::{Path, PathBuf};

    #[test]
    fn Test_Registered_Extensions_Should_Name_Rust_And_Go()
    {
        assert_eq!(Registered_Extensions(), vec![nomos_lang_rust::RUST_EXTENSION, nomos_lang_go::GO_EXTENSION]);
    }

    #[test]
    fn Test_Script_Extensions_Should_Not_Overlap_Registered_Extensions()
    {
        for extension in SCRIPT_EXTENSIONS
        {
            assert!(!Registered_Extensions().contains(&extension), "{extension} must not double-count as both a language and a script extension");
        }
    }

    #[test]
    fn Test_Walked_Sources_Should_Be_None_When_The_Root_Is_Not_A_Directory()
    {
        let root = std::env::temp_dir().join("nomos-workspace-discovery-walked-sources-missing");
        let _ignored = std::fs::remove_dir_all(&root);

        assert!(Walked_Sources(&root, &Registered_Extensions()).is_none());
    }

    #[test]
    fn Test_Walked_Sources_Should_Discover_A_Go_File_Alongside_A_Rust_One()
    {
        let root = Fresh_Root("nomos-workspace-discovery-go-discovery");
        Write_Fixture(root.join("a.rs"), "pub fn One() {}\n");
        Write_Fixture(root.join("main.go"), "package main\n\nfunc One() {}\n");

        let sources = Walked_Sources(&root, &Registered_Extensions()).expect("a directory returns Some");

        let _ignored = std::fs::remove_dir_all(&root);
        let paths: Vec<String> = sources.iter().map(|source| return source.path.clone()).collect();
        assert_eq!(paths, vec!["a.rs", "main.go"], "{paths:?}");
    }

    #[test]
    fn Test_Read_Sources_Should_Only_Recognize_The_Extensions_It_Was_Given()
    {
        let root = Fresh_Root("nomos-workspace-discovery-caller-supplied-recognition");
        Write_Fixture(root.join("a.rs"), "pub fn One() {}\n");
        Write_Fixture(root.join("deploy.sh"), "#!/bin/bash\n");

        let rust_only = Read_Sources(&root, &["rs"]);
        let rust_and_scripts = Read_Sources(&root, &["rs", "sh"]);

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(rust_only.iter().map(|source| return source.path.clone()).collect::<Vec<_>>(), vec!["a.rs"]);
        assert_eq!(
            rust_and_scripts.iter().map(|source| return source.path.clone()).collect::<Vec<_>>(),
            vec!["a.rs", "deploy.sh"]
        );
    }

    #[test]
    fn Test_Read_Sources_Should_Skip_Target_And_Git_Directories()
    {
        let root = Fresh_Root("nomos-workspace-discovery-skip-generated");
        Make_Fixture_Directory(root.join("target"));
        Write_Fixture(root.join("target/built.rs"), "pub fn Built() {}\n");
        Write_Fixture(root.join("a.rs"), "pub fn One() {}\n");

        let paths = Read_Paths(&root);

        assert_eq!(paths, vec!["a.rs"], "{paths:?}");
    }

    /// `P67-SELF-CHECK-WALK-CROSSES-NESTED-WORKTREE-BOUNDARY-2`: a subdirectory that is
    /// itself a git worktree's own root -- a `.git` *file*, not a `.git` directory -- must
    /// not be descended into, the same as `target` already is not.
    #[test]
    fn Test_Read_Sources_Should_Not_Descend_Into_A_Nested_Git_Worktree()
    {
        let root = Fresh_Root("nomos-workspace-discovery-nested-worktree");
        Write_Fixture(root.join("a.rs"), "pub fn One() {}\n");
        Make_Fixture_Directory(root.join("worktree"));
        Write_Fixture(root.join("worktree/.git"), "gitdir: /elsewhere/.git/worktrees/example\n");
        Write_Fixture(root.join("worktree/stale.rs"), "pub fn Stale() {}\n");

        let paths = Read_Paths(&root);

        assert_eq!(paths, vec!["a.rs"], "{paths:?}");
    }

    /// Both halves in one tree, because either alone would pass while the other was wrong.
    ///
    /// `a.rs` is found *because* the root's own `standards.json` does not exclude the root --
    /// a walk that treated the marker as "skip this directory" without regard to which
    /// directory would find nothing at all and still look like it was working, since
    /// "excluded everything" and "found nothing" are the same empty list here.
    #[test]
    fn Test_Read_Sources_Should_Not_Descend_Into_A_Nested_Repository_Root()
    {
        let root = Fresh_Root("nomos-workspace-discovery-nested-root");
        Write_Fixture(root.join(ROOT_MARKER), "{}\n");
        Write_Fixture(root.join("a.rs"), "pub fn One() {}\n");
        Make_Fixture_Directory(root.join("vendored"));
        Write_Fixture(root.join("vendored/standards.json"), "{}\n");
        Write_Fixture(root.join("vendored/foreign.rs"), "pub fn two() {}\n");

        let paths = Read_Paths(&root);

        assert_eq!(paths, vec!["a.rs"], "{paths:?}");
    }

    /// A directory with no marker of its own is still descended into, so the skip above is a
    /// statement about the marker rather than about being nested.
    #[test]
    fn Test_Read_Sources_Should_Descend_Into_An_Ordinary_Nested_Directory()
    {
        let root = Fresh_Root("nomos-workspace-discovery-ordinary-nesting");
        Write_Fixture(root.join(ROOT_MARKER), "{}\n");
        Make_Fixture_Directory(root.join("nested"));
        Write_Fixture(root.join("nested/b.rs"), "pub fn Two() {}\n");

        let paths = Read_Paths(&root);

        assert_eq!(paths, vec!["nested/b.rs"], "{paths:?}");
    }

    #[test]
    fn Test_Relative_Path_Should_Use_Forward_Slashes_On_Every_Platform()
    {
        let root = PathBuf::from("root");
        let path = root.join("nested").join("file.rs");

        assert_eq!(Relative_Path(&root, &path), "nested/file.rs");
    }

    #[test]
    fn Test_Read_Source_Should_File_The_Relative_Path()
    {
        let root = Fresh_Root("nomos-workspace-discovery-read-source");
        let path = root.join("a.rs");

        let source = Read_Source(&root, &path, "pub fn One() {}\n".to_owned());

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(source.path, "a.rs");
    }

    /// The paths [`Read_Sources`] reports under `root`, with the fixture root removed again.
    ///
    /// Every test above asserts against this list, so removing the temporary tree lives in one
    /// place rather than once per test.
    fn Read_Paths(root: &Path) -> Vec<String>
    {
        let sources = Read_Sources(root, &Registered_Extensions());
        let _ignored = std::fs::remove_dir_all(root);
        return sources.iter().map(|source| return source.path.clone()).collect();
    }
}
