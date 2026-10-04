//! A waiver in `suppressions.json` whose path is gone is noticed here, because nothing else
//! reads the file.
//!
//! `suppressions.json` is code-standards' waiver file, adopted under `OD-GATE-018`. Nothing in
//! this workspace reads it: `nomos check` reports a declared entry of its own that matches
//! nothing (`OD-GATE-032`), and the gate never runs code-standards. So a waiver whose file was
//! deleted or renamed stayed until somebody happened to read it.
//! `P209-THE-REST-OF-P114S-PLATFORM-RENAMES-LEFT-LIVE-OLD-NAMES-AND-WAIVERS-THAT-MATCH-NOTHING`
//! removed three such waivers by reading, and found a fourth that
//! `P210-THREE-PLATFORM-COUNTS-ARE-STALE-AND-NOTHING-NOTICES-A-WAIVER-WHOSE-PATH-IS-GONE`
//! removed. This test is that reading, done on every run, over the real file.
//!
//! code-standards matches a waiver's path as written: it covers the file it names, or everything
//! under it when it names a directory. A path that exists as neither waives nothing. Its matcher
//! also accepts a doublestar glob. No waiver here uses one, and one that did would be reported
//! below as a path that does not exist, which is the prompt to teach this test to expand it.
//!
//! # What this cannot see
//!
//! A waiver whose file survives but whose symbol was renamed inside it matches nothing either,
//! and this test passes it: the path is there, and whether any finding still carries the
//! waiver's fingerprint is a question only code-standards can answer. `P209`'s commit,
//! `61e0ebd6`, records the method that answers it. Run the waiver's own check with `--json
//! --include-tests` over a copy of the file, in a scratch directory holding this repository's
//! `standards.json` and no `suppressions.json`, so nothing is waived out of the output. Run it
//! first over the file as it stood before the rename, where it must report the waiver's
//! fingerprint, and then over today's file, where a waiver that matches nothing finds none.

use std::path::Path;

/// The waiver file, relative to the repository root.
const WAIVER_FILE: &str = "suppressions.json";

/// A directory this repository has. A waiver may name one, and it covers everything under it.
const A_DIRECTORY_THAT_EXISTS: &str = "crates";

/// A path this repository does not have, as either a file or a directory.
const A_PATH_THAT_IS_GONE: &str = "crates/a-crate-this-repository-does-not-have/src/lib.rs";

/// Every waiver path in `text` that names neither a file nor a directory under `root`, in the
/// order the file lists them.
///
/// # Panics
///
/// If `text` is not JSON, carries no `waivers` array, or holds a waiver without a string
/// `path`. A reader that answered nothing for a file it could not read would pass in its own
/// worst case, so a file this cannot read fails the test instead.
fn Paths_That_Are_Gone(root: &Path, text: &str) -> Vec<String>
{
    let document: serde_json::Value = serde_json::from_str(text).expect("the waiver file parses as JSON");
    let waivers = document
        .get("waivers")
        .and_then(serde_json::Value::as_array)
        .expect("the waiver file carries a `waivers` array");

    return waivers
        .iter()
        .map(Path_Of)
        .filter(|path| return !root.join(path).exists())
        .map(str::to_owned)
        .collect();
}

/// The path `waiver` names, which code-standards matches against a finding's file.
///
/// # Panics
///
/// If the waiver carries no string `path`, for the reason [`Paths_That_Are_Gone`] gives.
fn Path_Of(waiver: &serde_json::Value) -> &str
{
    return waiver
        .get("path")
        .and_then(serde_json::Value::as_str)
        .expect("every waiver names its path as a string");
}

/// A waiver file holding one waiver for each path, in that order.
fn Waiver_File_Naming(paths: &[&str]) -> String
{
    let waivers: Vec<serde_json::Value> = paths
        .iter()
        .map(|path| return serde_json::json!({ "path": path }))
        .collect();

    return serde_json::json!({ "waivers": waivers }).to_string();
}

/// Every waiver in the real `suppressions.json` names a file or a directory this repository has.
/// A failure names each waiver path that is gone, so its entry can be removed, or pointed at the
/// file it moved to once that file is shown to carry the finding.
#[test]
fn Test_Every_Waiver_Should_Name_A_Path_That_Exists()
{
    let root = nomos_contract_tests::Repository_Root();
    let text = std::fs::read_to_string(root.join(WAIVER_FILE)).expect("the repository root holds suppressions.json");

    let gone = Paths_That_Are_Gone(&root, &text);

    assert!(
        gone.is_empty(),
        "{WAIVER_FILE} holds a waiver whose path exists neither as a file nor as a directory, so it \
         waives nothing: {gone:?}"
    );
}

/// The reader reports a path that is gone and passes a file or a directory that exists, so the
/// test above is not green because the reader never reports anything.
#[test]
fn Test_A_Waiver_Naming_A_Path_That_Is_Gone_Should_Be_Reported_And_Only_That_One()
{
    let root = nomos_contract_tests::Repository_Root();
    let text = Waiver_File_Naming(&[WAIVER_FILE, A_PATH_THAT_IS_GONE, A_DIRECTORY_THAT_EXISTS]);

    let gone = Paths_That_Are_Gone(&root, &text);

    assert_eq!(gone, vec![A_PATH_THAT_IS_GONE.to_owned()]);
}
