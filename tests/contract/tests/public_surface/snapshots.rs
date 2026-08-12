//! The committed snapshots against the tree, in all three directions.
//!
//! A crate that widens its surface, a snapshot describing a crate that no longer has one,
//! and a member that produces no surface and never said it would not — the first is the
//! assertion this suite exists for and the other two are the ways it could go quiet.

use crate::bless::Bless;
use crate::reading::{
    Rendered, Snapshot_Directory, Snapshot_Path, Snapshotted_Packages, Surfaces, BLESS,
};
use nomos_contract_tests::{Surface, Workspace};
use std::collections::BTreeSet;
use std::path::Path;

/// A crate with no library target has no public surface to snapshot.
///
/// Declared rather than derived from the absence of `src/lib.rs`, so that a crate losing
/// its library — which is a large change — cannot quietly lose its snapshot with it.
const WITHOUT_A_LIBRARY: &[&str] = &["nomos-cli"];

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
    let wrong = Disagreeing_With_Their_Snapshots(&surfaces, &Snapshot_Directory());

    assert!(
        wrong.is_empty(),
        "these crates export something other than what is checked in:\n\n{}\n\
         A surface widens because somebody decided to widen it. Re-run with {BLESS} set to \
         the crate you meant to change, and commit the diff with the change that caused it.",
        wrong.join("\n\n")
    );
}

/// Every crate whose source declares something other than what its snapshot says.
fn Disagreeing_With_Their_Snapshots(surfaces: &[Surface], directory: &Path) -> Vec<String>
{
    let mut wrong = Vec::new();
    for surface in surfaces
    {
        let path = Snapshot_Path(directory, &surface.package);
        let expected = std::fs::read_to_string(&path).unwrap_or_default();
        let found = Rendered(surface);
        if expected != found
        {
            wrong.push(format!("{}\n{}", surface.package, Difference(&expected, &found)));
        }
    }

    return wrong;
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
    let on_disk = Snapshot_Stems(&directory);
    let expected = Snapshotted_Packages();
    let orphaned: Vec<&String> = on_disk.difference(&expected).collect();
    let unsnapshotted: Vec<&String> = expected.difference(&on_disk).collect();

    assert!(!on_disk.is_empty(), "no snapshot is checked in under {}", directory.display());
    assert!(
        orphaned.is_empty(),
        "these snapshots name no workspace member with a library: {orphaned:?}"
    );
    assert!(
        unsnapshotted.is_empty(),
        "these crates have a library and no snapshot: {unsnapshotted:?}.\n\
         Re-run with {BLESS} set to those names."
    );
}

/// The package name of every snapshot file in a directory.
fn Snapshot_Stems(directory: &Path) -> BTreeSet<String>
{
    let entries = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));

    return entries
        .flatten()
        .filter_map(|entry| return Snapshot_Stem(&entry.path()))
        .collect();
}

/// A file's package name, or nothing if the file is not a snapshot.
fn Snapshot_Stem(path: &Path) -> Option<String>
{
    if path.extension().is_none_or(|extension| return extension != "txt")
    {
        return None;
    }

    return path
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .map(str::to_owned);
}

/// Every member is either snapshotted or declared to have no library.
///
/// The gap the two assertions above leave between them. A crate whose `src/lib.rs` is
/// deleted stops producing a surface, so `Surfaces()` stops naming it, so both of them
/// go quiet about it at once.
#[test]
fn Test_Every_Member_Should_Be_Snapshotted_Or_Declared_Library_Less()
{
    let snapshotted = Snapshotted_Packages();
    let unaccounted = Members_With_Neither(&snapshotted);
    let contradicted: Vec<&&str> = WITHOUT_A_LIBRARY
        .iter()
        .filter(|name| return snapshotted.contains(**name))
        .collect();

    assert!(
        unaccounted.is_empty(),
        "these members produce no public surface and are not declared library-less: \
         {unaccounted:?}.\n\
         A crate that stops having a library stops being compared, and nothing else here \
         would say so."
    );
    assert!(
        contradicted.is_empty(),
        "these are declared library-less and have a library: {contradicted:?}"
    );
}

/// The members that produce no public surface and are not declared library-less either.
fn Members_With_Neither(snapshotted: &BTreeSet<String>) -> Vec<String>
{
    let workspace = Workspace::Load();

    return workspace
        .Members()
        .iter()
        .map(|member| return member.name.clone())
        .filter(|name| return !snapshotted.contains(name))
        .filter(|name| return !WITHOUT_A_LIBRARY.contains(&name.as_str()))
        .collect();
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
