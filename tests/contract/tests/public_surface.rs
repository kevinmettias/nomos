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
//! To update, run the suite with `NOMOS_SURFACE_BLESS` set to anything. It rewrites every
//! snapshot and then *fails*, so a machine with the variable left set cannot report green
//! — a bless that passes is a check that anybody's environment can switch off.
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

/// Setting this rewrites the snapshots instead of comparing against them.
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

fn Snapshot_Path(package: &str) -> PathBuf
{
    return Snapshot_Directory().join(format!("{package}.txt"));
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

    if std::env::var_os(BLESS).is_some()
    {
        Rewrite(&surfaces);
    }

    let mut wrong = Vec::new();

    for surface in &surfaces
    {
        let path = Snapshot_Path(&surface.package);
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
         A surface widens because somebody decided to widen it. Re-run with {BLESS} set \
         to rewrite the snapshots, and commit the diff with the change that caused it.",
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
         Re-run with {BLESS} set."
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

/// Writes the snapshots and fails, so blessing is never a passing run.
fn Rewrite(surfaces: &[Surface])
{
    let directory = Snapshot_Directory();
    std::fs::create_dir_all(&directory).expect("creates the snapshot directory");

    for surface in surfaces
    {
        std::fs::write(Snapshot_Path(&surface.package), Rendered(surface))
            .expect("writes a snapshot");
    }

    panic!(
        "{BLESS} was set, so {} snapshot(s) were rewritten and nothing was compared. \
         Re-run without it.",
        surfaces.len()
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
