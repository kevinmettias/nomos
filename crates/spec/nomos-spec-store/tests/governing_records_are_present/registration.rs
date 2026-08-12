//! Two independently authored sets — the records on disk and the registrations that name
//! them — compared in both directions, with the controls that keep the comparison honest.

use crate::queries::Seeded;
use nomos_spec_store::{GOVERNING_RECORD_IDS, SpecificationStore};
use std::path::Path;

/// The fewest governing records this build accepts.
///
/// A floor, not a count, and the difference is the whole of what `OD-SPEC-007` traded. It
/// is **not raised when a record is added** — adding one costs
/// `crates/spec/nomos-spec-store/records/<ID>.record` and nothing any other record writer
/// edits, which is the point. It is lowered by a deliberate removal.
///
/// What it still buys is the one thing nothing else here can see. A record file and its
/// registration deleted *together* pass both directions of
/// [`Test_Every_Canonical_Record_On_Disk_Should_Be_Governing`], because the two sets still
/// agree — they agree about a record that is gone. The literal count that stood here caught
/// that by requiring the deleter to remember a third artifact, and this keeps exactly that
/// mechanism: a deletion has to be accompanied by lowering this number, and a deleter who
/// forgets goes red.
///
/// What is given up is the ceiling, and the cost is drift. The guarantee is exact only
/// while the count sits on the floor; once the count has risen above it, a coordinated
/// deletion inside the slack is caught by nothing here. An upper bound was considered and
/// rejected in `OD-SPEC-007`: it reintroduces the shared edit on an unpredictable schedule,
/// so instead of every record writer colliding, one unforeseeable record writer in every N
/// collides with all the others — which is worse to author against than one that fires
/// always. This number may be raised for free by any item that already has this file open
/// for another reason.
const FEWEST_GOVERNING_RECORDS: usize = 32;

/// Where a record's registration lives, from this crate's manifest directory.
fn Registration_Directory() -> std::path::PathBuf
{
    return std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("records");
}

/// Where the records themselves live.
fn Record_Directory() -> std::path::PathBuf
{
    return std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../docs/records");
}

/// Every identifier under `docs/records` that claims canonical normative authority.
///
/// Side A of the comparison this module exists for, read at test time from the directory
/// records are authored in. Side B is `GOVERNING_RECORD_IDS`, assembled from a directory of
/// registration files somebody wrote by hand. The two sides are independently authored, and
/// that — not where either of them is spelled — is what makes the comparison a check.
fn Canonical_Records() -> Vec<String>
{
    let directory = Record_Directory();
    let entries = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));
    let mut canonical = Vec::new();

    for entry in entries.flatten()
    {
        let path = entry.path();
        let declared = Canonical_Id(&path);

        canonical.extend(declared);
    }

    return canonical;
}

/// The identifier a file declares, if it is a record claiming canonical normative authority.
///
/// # Panics
///
/// Panics if a file claims that authority and declares no `id:`. That is a record no
/// registration can name, which is the state this whole module exists to make visible.
fn Canonical_Id(path: &Path) -> Option<String>
{
    if path.extension().is_none_or(|extension| return extension != "md")
    {
        return None;
    }
    let Ok(text) = std::fs::read_to_string(path)
    else
    {
        return None;
    };
    if !text.contains("authority: canonical-normative-record")
    {
        return None;
    }
    let Some(id) = text.lines().find_map(|line| return line.strip_prefix("id: "))
    else
    {
        panic!("{} claims canonical authority and declares no id", path.display());
    };

    return Some(id.trim().to_owned());
}

/// The two directions of a set disagreement: on side A and not side B, and the reverse.
///
/// Named rather than a pair. Both members are `Vec<String>` and the compiler cannot tell
/// them apart, so a call site that swapped them would report an unseeded record as a
/// phantom one and still build.
struct Disagreement
{
    unseeded: Vec<String>,
    phantom: Vec<String>,
}

/// The comparison the guard makes, over two sets handed to it.
///
/// Extracted so that a control claiming the guard would have caught something is exercising
/// **the guard** rather than a second implementation of it written beside the first.
fn Disagreements(canonical: &[String], governing: &[&str]) -> Disagreement
{
    let unseeded: Vec<String> = canonical
        .iter()
        .filter(|id| return !governing.contains(&id.as_str()))
        .cloned()
        .collect();

    let phantom: Vec<String> = governing
        .iter()
        .filter(|id| return !canonical.iter().any(|found| return found == *id))
        .map(|id| return (*id).to_owned())
        .collect();

    return Disagreement { unseeded, phantom };
}

#[test]
fn Test_Every_Governing_Record_Should_Resolve_By_Id()
{
    let store = Seeded();

    let missing: Vec<&str> = GOVERNING_RECORD_IDS
        .iter()
        .filter(|id| store.Node_Uid(id).expect("queries").is_none())
        .copied()
        .collect();

    assert!(missing.is_empty(), "not in the store: {missing:?}");

    // A floor rather than a count. What that keeps, what it gives up, and why there is no
    // ceiling are on `FEWEST_GOVERNING_RECORDS` above.
    assert!(
        GOVERNING_RECORD_IDS.len() >= FEWEST_GOVERNING_RECORDS,
        "{} governing record(s), and this build accepts no fewer than \
         {FEWEST_GOVERNING_RECORDS}. A record left the governing set: restore its \
         registration under crates/spec/nomos-spec-store/records/, or lower the floor in \
         the same commit that removes it. Nothing else here can see a record and its \
         registration deleted together.",
        GOVERNING_RECORD_IDS.len()
    );
}

/// Every record on disk that claims to be canonical and normative is in the store.
///
/// The direction nothing checked. `GOVERNING_RECORD_IDS` was compared against a seeded store
/// and never against `docs/records`, so a record could be written, declare itself
/// `canonical-normative-record`, be cited in commits and other records, and never reach the
/// store — which is what happened to six of them, including `OD-STORE-001` and
/// `OD-ANALYSIS-001`, two records that decide how the store and the analysis kernel behave.
///
/// `D-129` says the store holds identity and markdown is an editing surface. A governing
/// record that exists only as a file is that decision failing in the one place it is easiest
/// to check.
#[test]
fn Test_Every_Canonical_Record_On_Disk_Should_Be_Governing()
{
    let canonical = Canonical_Records();
    assert!(
        !canonical.is_empty(),
        "no canonical record was found under {}. Every assertion here iterates over that \
         set, so an empty one passes having checked nothing",
        Record_Directory().display()
    );
    let Disagreement { unseeded, phantom } = Disagreements(&canonical, GOVERNING_RECORD_IDS);

    assert!(
        unseeded.is_empty(),
        "these records claim canonical normative authority and are not seeded into the \
         store: {unseeded:?}.\n\
         Add crates/spec/nomos-spec-store/records/<ID>.record naming the file. A governing \
         record that lives only as a file is D-129 failing where it is easiest to check."
    );

    assert!(
        phantom.is_empty(),
        "the store seeds these and no file under docs/records declares them: {phantom:?}"
    );
}

/// The control this whole arrangement is measured against: the guard did not go vacuous.
///
/// Runs the real comparison with the real canonical set and the real governing set **minus
/// one element**, and asserts the missing one is reported. That is what a record written
/// without its registration looks like from here, and it is exactly `OD-SPEC-005`'s defect:
/// six records were authored as files and never declared.
///
/// It is done over the sets rather than by deleting the file because side B is assembled at
/// compile time and no `#[test]` causes a rebuild. `OD-SPEC-007` records the one-off manual
/// check that closed that gap end to end, with its result, rather than a test claiming to
/// have done it. The complement is
/// [`Test_The_Governing_List_Should_Be_The_Registration_Directory`], which pins side B to the
/// registration directory as it is on disk.
#[test]
fn Test_A_Record_Whose_Registration_Is_Missing_Should_Be_Unseeded()
{
    let canonical = Canonical_Records();
    let Some(dropped) = GOVERNING_RECORD_IDS.first()
    else
    {
        panic!("nothing governs this build, so this control has nothing to remove")
    };

    let governing: Vec<&str> = GOVERNING_RECORD_IDS
        .iter()
        .filter(|id| return *id != dropped)
        .copied()
        .collect();

    let Disagreement { unseeded, phantom } = Disagreements(&canonical, &governing);

    assert_eq!(
        unseeded,
        vec![(*dropped).to_owned()],
        "a canonical record with no registration was not reported as unseeded, so the \
         guard no longer sees the thing OD-SPEC-005 exists about"
    );
    assert!(phantom.is_empty(), "removing a registration invented a phantom: {phantom:?}");
}

/// The other direction: a registration nothing on disk declares.
#[test]
fn Test_A_Registration_With_No_Record_Should_Be_A_Phantom()
{
    let canonical = Canonical_Records();
    let invented = "OD-INVENTED-404";

    let mut governing: Vec<&str> = GOVERNING_RECORD_IDS.to_vec();
    governing.push(invented);

    let Disagreement { unseeded, phantom } = Disagreements(&canonical, &governing);

    assert_eq!(phantom, vec![invented.to_owned()]);
    assert!(unseeded.is_empty(), "inventing a registration unseeded a record: {unseeded:?}");
}

/// `GOVERNING_RECORD_IDS` is the registration directory, and nothing else.
///
/// The vacuity boundary from the other side. [`Test_Every_Canonical_Record_On_Disk_Should_Be_Governing`]
/// compares side B against `docs/records`; this compares side B against the directory it is
/// supposed to have been assembled from, read at test time. Delete
/// `records/OD-FOO-001.record` and rebuild, and this goes red before that one does, naming
/// the file.
///
/// What it does **not** prove is that the build script reads `records/` rather than
/// `docs/records`: in a consistent tree both produce the same identifiers. That is guarded
/// structurally in `src/registration/mod.rs` and textually by
/// [`Test_The_Build_Script_Should_Not_Enumerate_The_Record_Directory`].
#[test]
fn Test_The_Governing_List_Should_Be_The_Registration_Directory()
{
    let directory = Registration_Directory();
    let stems = Registered_Stems(&directory);
    assert!(
        !stems.is_empty(),
        "no registration was found under {}, so this compared nothing",
        directory.display()
    );
    let Disagreement {
        unseeded: unregistered,
        phantom: undeclared,
    } = Disagreements(&stems, GOVERNING_RECORD_IDS);

    assert!(
        unregistered.is_empty() && undeclared.is_empty(),
        "the governing list and the registration directory disagree. Registered and not \
         governing: {unregistered:?}. Governing and not registered: {undeclared:?}.\n\
         The list is generated from that directory, so a disagreement means the build ran \
         against a different one."
    );
}

/// Every registration stem in a directory, read at test time.
fn Registered_Stems(directory: &Path) -> Vec<String>
{
    let entries = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));
    let mut stems = Vec::new();

    for entry in entries.flatten()
    {
        let path = entry.path();
        let stem = Registration_Stem(&path);

        stems.extend(stem);
    }

    return stems;
}

/// The stem of a registration file, or `None` for anything else in the directory.
fn Registration_Stem(path: &Path) -> Option<String>
{
    if path.extension().is_none_or(|extension| return extension != "record")
    {
        return None;
    }

    return path
        .file_stem()
        .and_then(|stem| return stem.to_str())
        .map(str::to_owned);
}

/// What the refused derivation would check, exhibited rather than argued.
///
/// `OD-LEDGER-007` declined to derive `GOVERNING_RECORD_IDS` from `docs/records` because
/// the guard above would then compare that directory against itself. This is that
/// comparison, made with both sides taken from `docs/records` — including a record nobody
/// registered — and it comes back clean. It asserts nothing about the arrangement that is
/// actually in place; it exists so that the next person who proposes globbing `docs/records`
/// finds a test that already says what would happen.
#[test]
fn Test_Comparing_The_Directory_Against_Itself_Would_Check_Nothing()
{
    let mut would_be_canonical = Canonical_Records();
    would_be_canonical.push("OD-NOBODY-DECLARED-THIS-001".to_owned());

    // Side B, as the refused derivation would have produced it: read off the very directory
    // side A was read from.
    let would_be_governing: Vec<&str> = would_be_canonical
        .iter()
        .map(|id| return id.as_str())
        .collect();

    let Disagreement {
        unseeded: would_be_unseeded,
        phantom: would_be_phantom,
    } = Disagreements(&would_be_canonical, &would_be_governing);

    assert!(
        would_be_unseeded.is_empty() && would_be_phantom.is_empty(),
        "a directory compared against itself disagreed with itself, which would mean this \
         demonstration is no longer demonstrating the failure OD-LEDGER-007 refused"
    );
}

/// The build script's input directory is the registration directory.
///
/// **This is a textual check and it is nothing more than that.** It reads `build.rs` and
/// `src/registration/mod.rs` as text and asserts that no directory enumeration in either of
/// them is applied to a path built from `docs/records`. It cannot detect a rewrite that
/// reaches the same directory by another spelling, and it is not evidence about behaviour.
///
/// It exists because the regression it watches for — "simplifying" the generator into
/// globbing `docs/records`, the exact move `OD-LEDGER-007` refused — is invisible to every
/// other test in the tree: in a consistent tree the two directories yield the same
/// identifiers, so everything still passes while the guard checks nothing. The behavioural
/// half of the guard is `Test_The_Reader_Should_Enumerate_The_Directory_It_Is_Given` in
/// `src/registration/mod.rs`, which hands the reader a directory of one and catches it if it
/// returns more.
#[test]
fn Test_The_Build_Script_Should_Not_Enumerate_The_Record_Directory()
{
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    for file in ["build.rs", "src/registration/mod.rs"]
    {
        let path = crate_root.join(file);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

        for (ordinal, after) in text.split("read_dir(").enumerate().skip(1)
        {
            let arguments = after.split(')').next().unwrap_or(after);

            assert!(
                !arguments.contains("docs/records") && !arguments.contains("RECORD_DIRECTORY"),
                "{file}: directory enumeration number {ordinal} is applied to \
                 `{arguments}`, which names the record directory. Deriving the governing \
                 list from docs/records makes \
                 Test_Every_Canonical_Record_On_Disk_Should_Be_Governing compare that \
                 directory against itself and pass having checked nothing — see \
                 OD-LEDGER-007 and OD-SPEC-007."
            );
        }
    }
}

/// The negative control. Without it the assertion above would pass on a store that
/// contains everything for some other reason.
#[test]
fn Test_An_Unseeded_Store_Should_Hold_None_Of_Them()
{
    let store = SpecificationStore::In_Memory().expect("opens");

    for id in GOVERNING_RECORD_IDS
    {
        assert!(
            store.Node_Uid(id).expect("queries").is_none(),
            "{id} appeared in a store nobody seeded"
        );
    }
}
