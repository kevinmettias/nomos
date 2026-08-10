//! Each crate's public surface, checked in and compared.
//!
//! The fifth anti-leak assertion, and the one that was never built. Three are in
//! `boundaries.rs`, module reachability shipped, and the transport check has no transports
//! to check. This is the remaining one: a type that leaks out of a crate's surface was a
//! review question, and the other four exist precisely because review questions are the
//! ones answered wrong quietly.
//!
//! # How it fails
//!
//! Widening a surface fails this test until the snapshot beside it is updated. That is the
//! point: `git diff` then shows the new export as a line somebody added to a committed
//! file, in the commit that added it, rather than as nothing at all.
//!
//! To update, run the suite with `NOMOS_SURFACE_BLESS` set to the crate whose snapshot you
//! meant to change — `NOMOS_SURFACE_BLESS=nomos-rules`, or several names separated by
//! commas. It rewrites those snapshots, leaves every other one alone byte for byte, and
//! then *fails*, so a machine with the variable left set cannot report green — a bless that
//! passes is a check that anybody's environment can switch off.
//!
//! # Why it makes you name the crate
//!
//! Because it once did not, and this tree is worked by several sessions at a time. Blessing
//! used to rewrite every snapshot, and rewrite it from the *working* tree: a snapshot
//! carrying somebody else's deliberate, uncommitted change was not reverted to `HEAD`, it
//! was overwritten with whatever their half-finished crate happened to export — and the
//! result still passed this test afterwards. That happened, between two live claims, and
//! the only reason the work survived is that both authors happened to look. Nothing in the
//! mechanism looked. It is a territory violation committed by a tool rather than by an
//! author, and no ownership rule can express it, because the tool is a test that everybody
//! else also has to run.
//!
//! So the widest action is no longer the easiest one to reach for: there is no spelling that
//! means "all of them", only the list of names you are willing to write down.
//!
//! A value that names no crate — `NOMOS_SURFACE_BLESS=1`, which is what "set to anything"
//! used to look like — is refused, loudly, with the names it could have used. Refusing is
//! not politeness: rewriting nothing while appearing to have been honoured is the same
//! defect as rewriting everything, wearing the other sign. A name that matches no workspace
//! member is refused for the same reason.
//!
//! # Where the snapshots come from
//!
//! `nomos_contract_tests::Public_Surface`, which reads the crate's own source. Not
//! `cargo public-api`: that tool is on no CI runner here and would need a nested `cargo`
//! invocation, which blocks on the target-directory lock held by the `cargo test` that
//! called it. `OD-GATE-002` records the decision and what it gives up.

use nomos_contract_tests::{Public_Surface, Surface, Workspace};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Where the committed snapshots live, relative to this crate.
const SNAPSHOTS: &str = "surface";

/// Naming crates in this rewrites *their* snapshots instead of comparing against them.
///
/// The value is a list of package names, separated by commas or whitespace. It is never a
/// flag: there is no value meaning "all", and a value naming nothing is refused.
const BLESS: &str = "NOMOS_SURFACE_BLESS";

/// A crate with no library target has no public surface to snapshot.
///
/// Declared rather than derived from the absence of `src/lib.rs`, so that a crate losing
/// its library — which is a large change — cannot quietly lose its snapshot with it.
const WITHOUT_A_LIBRARY: &[&str] = &["nomos-cli"];

fn Snapshot_Directory() -> PathBuf
{
    return Path::new(env!("CARGO_MANIFEST_DIR")).join(SNAPSHOTS);
}

/// A snapshot's file inside a directory of snapshots.
///
/// The directory is a parameter so that the bless path can be exercised against a scratch
/// directory. A test that blessed into `surface/` to prove blessing is well-behaved would
/// be committing the very act it is checking for.
fn Snapshot_Path(directory: &Path, package: &str) -> PathBuf
{
    return directory.join(format!("{package}.txt"));
}

/// A snapshot's text: the declarations, then the re-exports nothing here could follow.
fn Rendered(surface: &Surface) -> String
{
    let mut text = String::new();

    for declaration in &surface.declarations
    {
        text.push_str(declaration);
        text.push('\n');
    }

    if !surface.unresolved.is_empty()
    {
        text.push_str("\n# re-exports of items declared outside this crate\n");
        for line in &surface.unresolved
        {
            text.push_str(line);
            text.push('\n');
        }
    }

    return text;
}

/// Every workspace member that has a library, with the surface its source declares.
fn Surfaces() -> Vec<Surface>
{
    let workspace = Workspace::Load();
    let mut found = Vec::new();

    for member in workspace.Members()
    {
        if let Some(surface) = Public_Surface(&member.name, &member.root)
        {
            found.push(surface);
        }
    }

    found.sort_by(|left, right| return left.package.cmp(&right.package));
    return found;
}

/// The assertion.
#[test]
fn Test_Every_Crates_Public_Surface_Should_Match_Its_Snapshot()
{
    let surfaces = Surfaces();

    assert!(
        !surfaces.is_empty(),
        "no workspace member yielded a public surface. Every comparison below iterates \
         over this list, so an empty one passes having read nothing"
    );

    if let Some(value) = std::env::var_os(BLESS)
    {
        Bless(&value.to_string_lossy(), &surfaces, &Snapshot_Directory());
    }

    let mut wrong = Vec::new();

    for surface in &surfaces
    {
        let path = Snapshot_Path(&Snapshot_Directory(), &surface.package);
        let expected = std::fs::read_to_string(&path).unwrap_or_default();
        let found = Rendered(surface);

        if expected == found
        {
            continue;
        }

        wrong.push(format!(
            "{}\n{}",
            surface.package,
            Difference(&expected, &found)
        ));
    }

    assert!(
        wrong.is_empty(),
        "these crates export something other than what is checked in:\n\n{}\n\
         A surface widens because somebody decided to widen it. Re-run with {BLESS} set to \
         the crate you meant to change, and commit the diff with the change that caused it.",
        wrong.join("\n\n")
    );
}

/// Every snapshot on disk belongs to a crate that still has a library.
///
/// The other direction. Without it a crate could be deleted, or lose its library, and its
/// snapshot would sit in the tree describing an API nothing exports — which reads as
/// coverage and is not.
#[test]
fn Test_Every_Snapshot_Should_Belong_To_A_Crate_That_Has_One()
{
    let directory = Snapshot_Directory();
    let entries = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));

    let on_disk: BTreeSet<String> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().is_none_or(|extension| return extension != "txt")
            {
                return None;
            }

            return path
                .file_stem()
                .and_then(std::ffi::OsStr::to_str)
                .map(str::to_owned);
        })
        .collect();

    let expected: BTreeSet<String> = Surfaces()
        .into_iter()
        .map(|surface| return surface.package)
        .collect();

    assert!(!on_disk.is_empty(), "no snapshot is checked in under {}", directory.display());

    let orphaned: Vec<&String> = on_disk.difference(&expected).collect();
    assert!(
        orphaned.is_empty(),
        "these snapshots name no workspace member with a library: {orphaned:?}"
    );

    let unsnapshotted: Vec<&String> = expected.difference(&on_disk).collect();
    assert!(
        unsnapshotted.is_empty(),
        "these crates have a library and no snapshot: {unsnapshotted:?}.\n\
         Re-run with {BLESS} set to those names."
    );
}

/// Every member is either snapshotted or declared to have no library.
///
/// The gap the two assertions above leave between them. A crate whose `src/lib.rs` is
/// deleted stops producing a surface, so `Surfaces()` stops naming it, so both of them
/// go quiet about it at once.
#[test]
fn Test_Every_Member_Should_Be_Snapshotted_Or_Declared_Library_Less()
{
    let workspace = Workspace::Load();
    let snapshotted: BTreeSet<String> = Surfaces()
        .into_iter()
        .map(|surface| return surface.package)
        .collect();

    let unaccounted: Vec<String> = workspace
        .Members()
        .iter()
        .map(|member| return member.name.clone())
        .filter(|name| return !snapshotted.contains(name))
        .filter(|name| return !WITHOUT_A_LIBRARY.contains(&name.as_str()))
        .collect();

    assert!(
        unaccounted.is_empty(),
        "these members produce no public surface and are not declared library-less: \
         {unaccounted:?}.\n\
         A crate that stops having a library stops being compared, and nothing else here \
         would say so."
    );

    let contradicted: Vec<&&str> = WITHOUT_A_LIBRARY
        .iter()
        .filter(|name| return snapshotted.contains(**name))
        .collect();
    assert!(
        contradicted.is_empty(),
        "these are declared library-less and have a library: {contradicted:?}"
    );
}

/// The negative control.
///
/// Everything above is satisfied by a scanner that returns nothing for every crate: the
/// snapshots would be empty, they would match, and the suite would be green having
/// checked that nothing equals nothing. This asserts the scanner finds real exports, and
/// that it draws the line where the language draws it.
#[test]
fn Test_The_Scanner_Should_Find_Real_Exports_And_Stop_At_Restricted_Ones()
{
    let workspace = Workspace::Load();
    let member = workspace
        .Get("nomos-spec-project")
        .expect("nomos-spec-project is a workspace member");
    let surface =
        Public_Surface(&member.name, &member.root).expect("nomos-spec-project has a library");

    let says = |needle: &str| {
        return surface
            .declarations
            .iter()
            .any(|declaration| return declaration.contains(needle));
    };

    // Declared in a private module and re-exported by the crate root. If the resolver
    // stopped at `pub mod`, this crate's entire API would be invisible.
    assert!(says("Build("), "the re-exported Build function is not in the surface");
    assert!(says("pub struct"), "no struct reached the surface");
    assert!(says("pub enum"), "no enum reached the surface");

    // `Without_Test_Modules` blanks unit-test bodies before the scan. A helper written
    // inside one is not an export, and counting it would make every crate's surface grow
    // with its test suite.
    assert!(
        !says("::tests::"),
        "a unit test module reached the public surface: {:?}",
        surface.declarations
    );

    for restricted in ["pub(crate)", "pub(super)", "pub(in "]
    {
        assert!(
            !surface
                .declarations
                .iter()
                .any(|declaration| return declaration.contains(restricted)),
            "{restricted} reached the surface, which is not public"
        );
    }
}

/// Restricted visibility elsewhere in the workspace is genuinely excluded.
///
/// `gates.rs` in this very crate declares `pub(crate) fn Source_Files` and
/// `pub(crate) fn Without_Test_Modules`. Neither is an export, and a scanner that treated
/// any `pub` prefix as public would list both — so this is the case that would catch it.
#[test]
fn Test_A_Crate_Visible_Helper_Should_Not_Be_An_Export()
{
    let workspace = Workspace::Load();
    let member = workspace
        .Get("nomos-contract-tests")
        .expect("this crate is a workspace member");
    let surface = Public_Surface(&member.name, &member.root).expect("it has a library");

    for hidden in ["Source_Files", "Without_Test_Modules", "Matching_Brace"]
    {
        assert!(
            !surface
                .declarations
                .iter()
                .any(|declaration| return declaration.contains(hidden)),
            "{hidden} is pub(crate) and appears in the exported surface"
        );
    }

    assert!(
        surface
            .declarations
            .iter()
            .any(|declaration| return declaration.contains("Corpus_Gates")),
        "the crate's real exports are missing, so the assertions above proved nothing"
    );
}

/// A directory of this test's own to bless into.
///
/// Never `surface/`. A test that proved blessing is well-behaved by blessing into the
/// committed snapshots would be performing, on every run and in every session, exactly the
/// act it exists to rule out.
fn Scratch(name: &str) -> PathBuf
{
    let directory = std::env::temp_dir().join(format!("nomos-surface-bless-{name}"));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("creates a scratch snapshot directory");
    return directory;
}

fn Surface_Of<'a>(surfaces: &'a [Surface], package: &str) -> &'a Surface
{
    return surfaces
        .iter()
        .find(|surface| return surface.package == package)
        .unwrap_or_else(|| panic!("{package} is a workspace member with a public surface"));
}

/// Blessing one crate leaves every other snapshot byte for byte as it was.
///
/// The case that actually happened, reproduced: the foreign snapshot here *disagrees with
/// the tree*, because its owner is part-way through a deliberate change to it. That detail
/// is the whole test. A version of this where both snapshots already matched the tree would
/// pass under the old rewrite-everything behaviour too, since overwriting a file with its
/// own contents leaves no trace — and it was precisely the disagreeing snapshot, rewritten
/// from a half-finished tree and still passing afterwards, that cost another session its
/// work.
#[test]
fn Test_Blessing_One_Crate_Should_Leave_Another_Crates_Snapshot_Byte_Identical()
{
    let surfaces = Surfaces();
    let directory = Scratch("leaves-another-alone");

    let named = "nomos-contract-tests";
    let foreign = "nomos-spec-project";

    let mid_edit = "# a deliberate, uncommitted change somebody else has not finished\n";
    let stale = "# whatever this crate used to export\n";
    assert_ne!(
        mid_edit,
        Rendered(Surface_Of(&surfaces, foreign)),
        "the foreign snapshot must disagree with the tree, or this test proves nothing"
    );

    std::fs::write(Snapshot_Path(&directory, foreign), mid_edit).expect("seeds the foreign one");
    std::fs::write(Snapshot_Path(&directory, named), stale).expect("seeds the named one");

    let written = Rewrite(&[named.to_owned()], &surfaces, &directory);

    // The file on disk, before the returned list. What the writer *says* it wrote is a
    // weaker claim than what is there afterwards, and it is the file that another session
    // loses.
    let after = std::fs::read_to_string(Snapshot_Path(&directory, foreign))
        .expect("the foreign snapshot still exists");
    assert_eq!(
        after, mid_edit,
        "blessing {named} rewrote {foreign}'s snapshot. That is a territory violation \
         committed by a tool: the file was not reverted to what is committed, it was \
         replaced with what a half-finished working tree exports, and the suite would go \
         green on it"
    );

    let rewritten = std::fs::read_to_string(Snapshot_Path(&directory, named))
        .expect("the named snapshot still exists");
    assert_eq!(
        rewritten,
        Rendered(Surface_Of(&surfaces, named)),
        "the crate that was actually named did not get rewritten, so the assertion above \
         is satisfied by a writer that writes nothing at all"
    );

    assert_eq!(written, [named], "the bless reported writing something it was not asked for");
}

/// A value that names nothing rewrites nothing, and says so.
#[test]
fn Test_A_Bless_Naming_No_Crate_Should_Be_Refused_Rather_Than_Widened()
{
    let surfaces = Surfaces();

    for empty in ["", "   ", ",", " , ,"]
    {
        let refusal = Requested(empty, &surfaces)
            .expect_err("a value naming no crate is refused rather than taken as all of them");
        assert!(
            refusal.contains("names no crate") && refusal.contains("nomos-rules"),
            "the refusal for {empty:?} does not tell the author how to name a crate: {refusal}"
        );
    }

    // The old spelling. `set to anything` used to mean `rewrite everything`; it now means
    // `you named a crate that does not exist`, which is the truth about what was typed.
    for unmatched in ["1", "true", "nomos-rulez"]
    {
        let refusal = Requested(unmatched, &surfaces)
            .expect_err("a name matching no crate is refused rather than silently skipped");
        assert!(
            refusal.contains("Nothing was rewritten"),
            "the refusal for {unmatched:?} does not say that nothing happened: {refusal}"
        );
    }

    // And a real name still resolves, or the assertions above are satisfied by a function
    // that refuses everything.
    assert_eq!(
        Requested("nomos-ledger, nomos-model", &surfaces).expect("both crates are real"),
        ["nomos-ledger".to_owned(), "nomos-model".to_owned()],
        "a list of real crate names did not resolve"
    );
}

/// The crates a bless was asked for, or the reason the request is refused.
///
/// Returns the refusal rather than panicking with it, so that the refusals are themselves
/// assertable — a guard whose message nothing reads is a guard nobody can tell is still
/// wired up.
fn Requested(value: &str, surfaces: &[Surface]) -> Result<Vec<String>, String>
{
    let known: BTreeSet<&str> =
        surfaces.iter().map(|surface| return surface.package.as_str()).collect();
    let names: BTreeSet<&str> = value
        .split(|character: char| return character == ',' || character.is_whitespace())
        .map(str::trim)
        .filter(|name| return !name.is_empty())
        .collect();

    let listed = known.iter().copied().collect::<Vec<&str>>().join(", ");

    if names.is_empty()
    {
        return Err(format!(
            "{BLESS} was set to {value:?}, which names no crate. It is a list of package \
             names now, not a flag: set it to the crate whose snapshot you meant to \
             change, such as {BLESS}=nomos-rules, or to several names separated by \
             commas. There is deliberately no value meaning all of them, because \
             rewriting a snapshot somebody else is mid-way through changing is how this \
             restriction was earned.\nThe crates with snapshots are: {listed}"
        ));
    }

    let unmatched: Vec<&str> =
        names.iter().copied().filter(|name| return !known.contains(name)).collect();
    if !unmatched.is_empty()
    {
        return Err(format!(
            "{BLESS} named {unmatched:?}, which is not a workspace member with a public \
             surface. Nothing was rewritten: a misspelled name that quietly blessed \
             nothing would look exactly like a bless that worked.\nThe crates with \
             snapshots are: {listed}"
        ));
    }

    return Ok(names.into_iter().map(str::to_owned).collect());
}

/// Writes the named snapshots into a directory, and only those. Returns what it wrote.
fn Rewrite(targets: &[String], surfaces: &[Surface], directory: &Path) -> Vec<String>
{
    std::fs::create_dir_all(directory).expect("creates the snapshot directory");

    let mut written = Vec::new();
    for surface in surfaces
    {
        if !targets.contains(&surface.package)
        {
            continue;
        }

        std::fs::write(Snapshot_Path(directory, &surface.package), Rendered(surface))
            .expect("writes a snapshot");
        written.push(surface.package.clone());
    }

    return written;
}

/// Resolves a bless request, honours it, and then fails whatever the answer was.
///
/// Every exit from here is a panic. A refused bless must not fall through to the
/// comparison, or an author who typed the name wrong would get a green run and read it as
/// their snapshot having been updated; and an honoured bless must not pass either, for the
/// reason the module doc gives.
fn Bless(value: &str, surfaces: &[Surface], directory: &Path)
{
    let targets = match Requested(value, surfaces)
    {
        Ok(targets) => targets,
        Err(refusal) => panic!("{refusal}"),
    };

    let written = Rewrite(&targets, surfaces, directory);

    panic!(
        "{BLESS} was set, so these snapshots were rewritten and nothing was compared: \
         {written:?}. Every other snapshot was left untouched. Re-run without it."
    );
}

/// The lines that differ, in both directions.
fn Difference(expected: &str, found: &str) -> String
{
    let before: BTreeSet<&str> = expected.lines().collect();
    let after: BTreeSet<&str> = found.lines().collect();

    let mut said = Vec::new();
    for line in after.difference(&before)
    {
        said.push(format!("  + {line}"));
    }
    for line in before.difference(&after)
    {
        said.push(format!("  - {line}"));
    }

    if said.is_empty()
    {
        return "  the same lines in a different order".to_owned();
    }

    return said.join("\n");
}
