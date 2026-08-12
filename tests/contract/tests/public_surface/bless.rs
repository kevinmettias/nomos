//! Rewriting a snapshot, and the two things that must never happen while doing it.
//!
//! Blessing is the only writing this suite does, and it is the act that once cost another
//! session its work. So the machinery and the tests that hold it to its word live together:
//! a bless names the crates it rewrites, leaves every other file byte for byte as it was,
//! and refuses a value that names nothing rather than reading it as all of them.

use crate::reading::{Rendered, Snapshot_Path, Surfaces, BLESS};
use nomos_contract_tests::Surface;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// A directory of this suite's own to bless into.
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
    let foreign_now = Rendering_Of(&surfaces, foreign);
    let named_now = Rendering_Of(&surfaces, named);
    let foreign_path = Seeded(&directory, foreign, mid_edit);
    let named_path = Seeded(&directory, named, stale);

    assert_ne!(
        mid_edit, foreign_now,
        "the foreign snapshot must disagree with the tree, or this test proves nothing"
    );

    let written = Rewrite(&[named.to_owned()], &surfaces, &directory);

    Assert_The_Foreign_Snapshot_Survived(&foreign_path, mid_edit, named, foreign);
    Assert_The_Named_Snapshot_Was_Rewritten(&named_path, &named_now);
    assert_eq!(written, [named], "the bless reported writing something it was not asked for");
}

/// What one crate's snapshot would say if it were blessed right now.
fn Rendering_Of(surfaces: &[Surface], package: &str) -> String
{
    let surface = Surface_Of(surfaces, package);

    return Rendered(surface);
}

/// A snapshot file seeded with text, so a bless has something to leave alone or overwrite.
fn Seeded(directory: &Path, package: &str, text: &str) -> PathBuf
{
    let path = Snapshot_Path(directory, package);
    std::fs::write(&path, text).expect("seeds a snapshot");

    return path;
}

/// The file on disk, before the returned list. What the writer *says* it wrote is a weaker
/// claim than what is there afterwards, and it is the file that another session loses.
fn Assert_The_Foreign_Snapshot_Survived(path: &Path, mid_edit: &str, named: &str, foreign: &str)
{
    let after = std::fs::read_to_string(path).expect("the foreign snapshot still exists");

    assert_eq!(
        after, mid_edit,
        "blessing {named} rewrote {foreign}'s snapshot. That is a territory violation \
         committed by a tool: the file was not reverted to what is committed, it was \
         replaced with what a half-finished working tree exports, and the suite would go \
         green on it"
    );
}

/// The crate that was named must have been rewritten.
fn Assert_The_Named_Snapshot_Was_Rewritten(path: &Path, expected: &str)
{
    assert_eq!(
        std::fs::read_to_string(path).expect("the named snapshot still exists"),
        expected,
        "the crate that was actually named did not get rewritten, so the assertion above          is satisfied by a writer that writes nothing at all"
    );
}

/// A value that names nothing rewrites nothing, and says so.
#[test]
fn Test_A_Bless_Naming_No_Crate_Should_Be_Refused_Rather_Than_Widened()
{
    let surfaces = Surfaces();

    Assert_A_Value_Naming_Nothing_Is_Refused(&surfaces);
    Assert_A_Name_Matching_Nothing_Is_Refused(&surfaces);

    // And a real name still resolves, or the assertions above are satisfied by a function
    // that refuses everything.
    assert_eq!(
        Requested("nomos-ledger, nomos-model", &surfaces).expect("both crates are real"),
        ["nomos-ledger".to_owned(), "nomos-model".to_owned()],
        "a list of real crate names did not resolve"
    );
}

/// A value that names no crate at all, which used to mean every crate.
fn Assert_A_Value_Naming_Nothing_Is_Refused(surfaces: &[Surface])
{
    for empty in ["", "   ", ",", " , ,"]
    {
        let refusal = Requested(empty, surfaces)
            .expect_err("a value naming no crate is refused rather than taken as all of them");

        assert!(
            refusal.contains("names no crate") && refusal.contains("nomos-rules"),
            "the refusal for {empty:?} does not tell the author how to name a crate: {refusal}"
        );
    }
}

/// The old spelling. `set to anything` used to mean `rewrite everything`; it now means `you
/// named a crate that does not exist`, which is the truth about what was typed.
fn Assert_A_Name_Matching_Nothing_Is_Refused(surfaces: &[Surface])
{
    for unmatched in ["1", "true", "nomos-rulez"]
    {
        let refusal = Requested(unmatched, surfaces)
            .expect_err("a name matching no crate is refused rather than silently skipped");

        assert!(
            refusal.contains("Nothing was rewritten"),
            "the refusal for {unmatched:?} does not say that nothing happened: {refusal}"
        );
    }
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
        return Err(Names_No_Crate(value, &listed));
    }
    let unmatched: Vec<&str> =
        names.iter().copied().filter(|name| return !known.contains(name)).collect();
    if !unmatched.is_empty()
    {
        return Err(Names_Nothing_Here(&unmatched, &listed));
    }

    return Ok(names.into_iter().map(str::to_owned).collect());
}

/// The refusal for a value that is a flag rather than a list of names.
fn Names_No_Crate(value: &str, listed: &str) -> String
{
    return format!(
        "{BLESS} was set to {value:?}, which names no crate. It is a list of package \
         names now, not a flag: set it to the crate whose snapshot you meant to \
         change, such as {BLESS}=nomos-rules, or to several names separated by \
         commas. There is deliberately no value meaning all of them, because \
         rewriting a snapshot somebody else is mid-way through changing is how this \
         restriction was earned.\nThe crates with snapshots are: {listed}"
    );
}

/// The refusal for a name that matches no workspace member with a public surface.
fn Names_Nothing_Here(unmatched: &[&str], listed: &str) -> String
{
    return format!(
        "{BLESS} named {unmatched:?}, which is not a workspace member with a public \
         surface. Nothing was rewritten: a misspelled name that quietly blessed \
         nothing would look exactly like a bless that worked.\nThe crates with \
         snapshots are: {listed}"
    );
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

        let path = Snapshot_Path(directory, &surface.package);
        std::fs::write(path, Rendered(surface)).expect("writes a snapshot");
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
pub(crate) fn Bless(value: &str, surfaces: &[Surface], directory: &Path)
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
