//! A test name written down in a governing record resolves to a test that exists.
//!
//! `AGENTS.md` sends every agent to `docs/records/` for why something was decided, and those
//! records name tests constantly. Until this test, nothing resolved any of them, so a record
//! could assert coverage that had been renamed away or deleted and the only way to find out
//! was to go looking.
//!
//! # Why this is checked at all
//!
//! The identical claim in a code doc comment is already resolved. `checks/mirror.rs` reads a
//! declared mirror and resolves it against real parsed test functions, and `D-134` ranks the
//! failure deliberately: a name resolving to nothing is `Blocking` while a universe admitting
//! it has no mirror is only `Advisory`, because a false claim of coverage is worse than a hole
//! somebody wrote down. `OD-SPEC-017` decided a record's test citation is the same claim, one
//! indirection further from the reader and republished by `spec/domain-specification.md`.
//!
//! That record also decided the other half, and this test deliberately does not implement it:
//! **a path citation is dated history and a dead one is not a defect.** 80 of 253 cited paths
//! do not exist, every measured cause is a legitimate change to the tree, and holding them
//! live would make every module reorganization a record-editing exercise. Nothing here reads
//! a path.
//!
//! # The three exclusions, and why none of them is a hole
//!
//! `OD-SPEC-017` names three, each measured rather than assumed:
//!
//! 1. **A trailing underscore is a line wrap, not a citation.** Nine of the 28 unresolved
//!    names in that record's census end in `_`, every one a prefix of a real test whose name
//!    the record broke across a line. A guard reporting them is wrong nine times before it is
//!    right once.
//! 2. **A record may name a test that deliberately does not exist** — a placeholder standing
//!    for any check, a fixture.
//! 3. **A record may name a test it is retiring**, which it has to be able to do: a record
//!    saying "`X` becomes `Y`" cannot say it without writing `X`.
//!
//! Two and three are one thing to this test, because the distinguishing evidence is
//! grammatical — present tense claims coverage, past tense reports history — and no test reads
//! tense. So the record declares it, in the one form `OD-SPEC-017` states: the backticked name
//! followed by `does not resolve`. [`Declared_Unresolvable`] is that check, and it compares
//! with whitespace normalized because a declaration wraps like any other prose.
//!
//! # Why there is a debt table as well
//!
//! Measured 2026-09-13, 34 record-and-name pairs across 18 records do not resolve once
//! truncations are set aside. `OD-SPEC-017` declares its own twelve. The rest are listed in
//! [`NOT_YET_DECLARED`] with the reason class that record's census found for each, and that
//! list is checked in both directions: an entry that starts resolving must be dropped, so the
//! table cannot quietly outlive the debt it describes.
//!
//! The table is the migration, not the mechanism. A record adopting the prose form loses its
//! rows; when the last row goes, so does the table.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Every unresolved citation this repository has not yet declared in its records, and the
/// reason class `OD-SPEC-017`'s census found for it.
///
/// Rows are `(record file stem, cited name, why it does not resolve)`. Per record and name
/// rather than per name: each record is answerable for what it writes down, and one record
/// declaring a name says nothing about another record citing the same one.
const NOT_YET_DECLARED: &[(&str, &str, &str)] = &[
    // Renames with an obvious surviving successor. Each is a real coverage claim naming a
    // name that moved, and repairing one means naming the successor in an amendment -- work
    // per record, not one sweep, because the assertion has to be checked to still hold.
    (
        "OD-CAPABILITY-010",
        "Test_A_Declared_Mirror_Should_Be_Read_Off_The_Doc_Comment",
        "renamed to Test_A_Declared_Mirror_Should_Be_Read_Off_The_Documentation_Comment by OD-RULES-017's own abbreviation rename",
    ),
    (
        "OD-GATE-017",
        "Test_A_Deselected_Rules_Finding_Should_Not_Block",
        "renamed to Test_A_Deselected_Rules_Finding_Should_Not_Exist",
    ),
    (
        "OD-RULES-018",
        "Test_Adr_Doc_001_Should_Carry_An_Explicit_Supersession_Edge",
        "renamed to Test_The_Superseded_Record_Should_Carry_An_Explicit_Supersession_Edge",
    ),
    (
        "OD-GATE-020",
        "Test_Registered_Should_Offer_All_Eight_Shipped_Rules",
        "renamed to Test_Registered_Should_Offer_Every_Composed_Rule, since the registry stopped offering eight",
    ),
    (
        "OD-COMPLETENESS-002",
        "Test_The_Registry_Should_Match_The_Manifesto",
        "renamed to Test_The_Registry_Should_Match_The_Manifest",
    ),
    (
        "OD-LEDGER-007",
        "Test_Two_Items_Writing_Different_Records_Should_Be_Claimable_At_Once",
        "the one rename whose successor changed subject as well as name",
    ),
    // Retired, and the code says so where the test used to be -- the convention OD-SPEC-017
    // names as the cheap half of the answer. The record still needs the prose declaration,
    // because a reader of the record does not see the tombstone.
    (
        "OD-HOST-007",
        "Test_Only_The_Platform_Adapter_May_Name_The_Sibling_Workspace",
        "retired 2026-09-10 by the owner's decision; boundaries/graph.rs carries the tombstone",
    ),
    (
        "OD-PLATFORM-003",
        "Test_Only_The_Platform_Adapter_May_Name_The_Sibling_Workspace",
        "the same test, cited by a second record",
    ),
    (
        "OD-DETERMINISM-001",
        "Test_The_Declared_Domains_Should_Be_The_Ones_This_Item_Covered",
        "renamed; determinism/declarations.rs carries a Why this test was renamed section naming both",
    ),
    (
        "OD-DETERMINISM-002",
        "Test_The_Declared_Domains_Should_Be_The_Ones_This_Item_Covered",
        "the same test, cited by a second record",
    ),
    // Placeholders: a name standing for any test, in prose that says so where it is written.
    // These need the declaration and nothing else -- there is no successor to name.
    (
        "OD-RULES-002",
        "Test_A_Check_That_Does_Not_Exist_Anywhere_In_This_Tree",
        "a placeholder, and its own prose says so",
    ),
    ("OD-RULES-002", "Test_X_And_More", "a placeholder for however a report elides a list"),
    ("OD-RULES-001", "Test_Anything", "a placeholder standing for any check"),
    ("OD-CAPABILITY-010", "Test_Name", "a placeholder standing for any test's name"),
    ("OD-RULES-025", "Test_Name", "the same placeholder, in a second record"),
    (
        "OD-GATE-004",
        "Test_This_Check_Does_Not_Exist_Anywhere",
        "a placeholder, and its own prose says so",
    ),
    // A record noting its own history: the two OD-SPEC-017 measured, plus the three renames
    // repaired under P95-FIVE-RECORDS-CITE-A-TEST-THAT-NEVER-EXISTED, whose amendments quote
    // the old name in order to say what it became.
    (
        "OD-LEDGER-012",
        "Test_A_Lapsed_Item_Should_Refuse_A_New_Holder_And_Keep_The_Old_One_Visible",
        "named under What Is Renamed Rather Than Reversed, saying what it becomes",
    ),
    (
        "OD-GATE-004",
        "Test_A_Phantom_Mirror_Should_Fail_The_Command",
        "named in the past tense, in a section arguing the old assertion was too weak",
    ),
    (
        "OD-LEDGER-013",
        "Test_A_Held_Pattern_Should_Refuse_Every_Other_Claim_On_The_Board",
        "quoted by that record's own amendment, saying what it became",
    ),
    (
        "OD-LEDGER-013",
        "Test_A_Pattern_Item_Should_Be_Unclaimable_Once_Anything_Is_Held",
        "quoted by that record's own amendment, saying what it became",
    ),
    (
        "OD-ANALYSIS-006",
        "Test_Combining_Should_Never_Exceed_The_Weaker_Input",
        "renamed to Test_Weaker_Of_Should_Never_Exceed_The_Weaker_Input",
    ),
    (
        "OD-ANALYSIS-006",
        "Test_No_Strength_Should_Refuse_A_Trace_Claim",
        "quoted by that record's own amendment, saying what it became",
    ),
];

/// A test name a record writes down either exists, is declared unresolvable by that record,
/// or is a row of [`NOT_YET_DECLARED`] — and nothing else.
///
/// # What a failure here means
///
/// A record claims coverage that is not there. Either the test was renamed, in which case the
/// record should name the successor in an amendment, or it is gone, in which case the record
/// should say so — `OD-SPEC-017` decides which, and neither remedy is to delete the sentence.
#[test]
fn Test_Every_Test_A_Record_Names_Should_Resolve_Or_Be_Declared()
{
    let root = Repository_Root();
    let real = Real_Test_Functions(&root);
    let excused: BTreeSet<(&str, &str)> =
        NOT_YET_DECLARED.iter().map(|(record, name, _)| return (*record, *name)).collect();

    let mut unexplained: Vec<String> = Vec::new();
    for record in Record_Files(&root)
    {
        let text = std::fs::read_to_string(&record).expect("a record listed by the walk must read");
        let stem = Identifier_Of(&record);

        for name in Cited_Names(&text)
        {
            if real.contains(&name) || Is_A_Line_Wrap(&name)
            {
                continue;
            }
            if Declared_Unresolvable(&text, &name) || excused.contains(&(stem.as_str(), name.as_str()))
            {
                continue;
            }
            unexplained.push(format!("{stem} cites {name}"));
        }
    }

    assert!(
        unexplained.is_empty(),
        "these records name a test that does not exist, and neither declare it nor appear in \
         NOT_YET_DECLARED. Name the successor in an amendment, or write \"`<name>` does not \
         resolve\" beside the citation and say why: {unexplained:#?}"
    );
}

/// Every row of [`NOT_YET_DECLARED`] is still a real debt.
///
/// Without this the table outlives what it describes: a record adopting the prose form, or a
/// test coming back under its old name, would leave a row asserting a gap that is closed —
/// which is the same shape of stale claim this whole file exists to catch, in the file that
/// catches it.
#[test]
fn Test_Every_Excused_Citation_Should_Still_Be_Unresolved()
{
    let root = Repository_Root();
    let real = Real_Test_Functions(&root);

    let mut stale: Vec<String> = Vec::new();
    for (record, name, _) in NOT_YET_DECLARED
    {
        if real.contains(*name)
        {
            stale.push(format!("{record} cites {name}, which now exists"));
            continue;
        }

        let Some(path) = Record_Files(&root).into_iter().find(|file| return Identifier_Of(file) == *record)
        else
        {
            stale.push(format!("{record} is named by NOT_YET_DECLARED and is not a record"));
            continue;
        };
        let text = std::fs::read_to_string(&path).expect("a record listed by the walk must read");
        if Declared_Unresolvable(&text, name)
        {
            stale.push(format!("{record} now declares {name} itself"));
        }
        else if !Cited_Names(&text).contains(*name)
        {
            stale.push(format!("{record} no longer cites {name}"));
        }
    }

    assert!(
        stale.is_empty(),
        "NOT_YET_DECLARED describes a debt that is paid -- drop the row: {stale:#?}"
    );
}

/// The one prose form `OD-SPEC-017` states: the backticked name followed by `does not
/// resolve`.
///
/// Whitespace is normalized before comparing because a declaration wraps like any other
/// prose, and three of `OD-SPEC-017`'s own do. A form the longest names could not fit on one
/// line would be a form the longest names could never use — which is the truncation defect
/// that record measured, arriving again by a different door.
fn Declared_Unresolvable(text: &str, name: &str) -> bool
{
    let flattened: Vec<&str> = text.split_whitespace().collect();

    return flattened.join(" ").contains(&format!("`{name}` does not resolve"));
}

/// Whether `name` is a fragment a line wrap left behind rather than a citation.
///
/// Every one of the nine `OD-SPEC-017` measured ends in an underscore, and every one is a
/// prefix of a real test. A name a Rust author actually wrote never ends in `_`.
fn Is_A_Line_Wrap(name: &str) -> bool
{
    return name.ends_with('_');
}

/// Every `Test_*` name `text` writes down.
///
/// `Test_` has to begin a token, or this finds one inside a longer identifier and reports a
/// name nobody wrote. Measured before this guard was believed: without the check it reports
/// `Test_Does_Not_Retry_Until_Green` out of the rule function `Check_A_Test_Does_Not_Retry_
/// Until_Green`, and two more of the same shape.
fn Cited_Names(text: &str) -> BTreeSet<String>
{
    let mut found = BTreeSet::new();
    let mut rest = text;
    let mut preceding: Option<char> = None;

    while let Some(offset) = rest.find("Test_")
    {
        let from = rest.get(offset..).unwrap_or_default();
        let before = rest.get(..offset).and_then(|head| return head.chars().next_back()).or(preceding);
        preceding = None;
        if before.is_some_and(|character| return character.is_ascii_alphanumeric() || character == '_')
        {
            rest = from.get(5..).unwrap_or_default();
            preceding = Some('_');
            continue;
        }
        let length = from
            .find(|character: char| return !(character.is_ascii_alphanumeric() || character == '_'))
            .unwrap_or(from.len());
        if let Some(name) = from.get(..length)
        {
            found.insert(name.to_owned());
        }
        rest = from.get(length.max(1)..).unwrap_or_default();
    }

    return found;
}

/// Every `fn Test_*` this workspace really declares.
fn Real_Test_Functions(root: &Path) -> BTreeSet<String>
{
    let mut found = BTreeSet::new();
    for file in Rust_Files(root)
    {
        let Ok(text) = std::fs::read_to_string(&file)
        else
        {
            continue;
        };
        let mut rest = text.as_str();
        while let Some(offset) = rest.find("fn Test_")
        {
            let from = rest.get(offset.saturating_add(3)..).unwrap_or_default();
            let length = from
                .find(|character: char| return !(character.is_ascii_alphanumeric() || character == '_'))
                .unwrap_or(from.len());
            if let Some(name) = from.get(..length)
            {
                found.insert(name.to_owned());
            }
            rest = from.get(length.max(1)..).unwrap_or_default();
        }
    }

    return found;
}

/// The identifier a record file's name begins with, which is how a row of
/// [`NOT_YET_DECLARED`] names it.
fn Identifier_Of(path: &Path) -> String
{
    let stem = path.file_stem().map(|name| return name.to_string_lossy().into_owned()).unwrap_or_default();

    // Segments while they are still identifier-shaped, rather than a fixed count: this
    // repository has both `OD-SPEC-017-...` and `D-134-...`, and a fixed three would read the
    // second as `D-134-a`.
    let identifier: Vec<&str> = stem
        .split('-')
        .take_while(|part| {
            return !part.is_empty()
                && part.chars().all(|character| return character.is_ascii_uppercase() || character.is_ascii_digit());
        })
        .collect();

    if identifier.is_empty()
    {
        return stem;
    }

    return identifier.join("-");
}

/// Every governing record's own file.
fn Record_Files(root: &Path) -> Vec<PathBuf>
{
    let mut found: Vec<PathBuf> = std::fs::read_dir(root.join("docs/records"))
        .expect("docs/records must be readable")
        .filter_map(|entry| return entry.ok().map(|found| return found.path()))
        .filter(|path| return path.extension().is_some_and(|extension| return extension == "md"))
        .collect();
    found.sort();

    return found;
}

/// Every `.rs` file in the workspace, build output excluded.
fn Rust_Files(root: &Path) -> Vec<PathBuf>
{
    let mut found = Vec::new();
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
            if path.extension().is_some_and(|extension| return extension == "rs")
            {
                found.push(path);
            }
            else if Is_Worth_Descending(&entry)
            {
                pending.push(path);
            }
        }
    }

    return found;
}

/// Whether the walk should go into `entry`.
///
/// Build output is skipped because it holds generated copies of the same sources, and a
/// dot-directory because `.git` alone is larger than the tree being read.
///
/// Its own function rather than two more conditions inside the walk: this workspace's own
/// `nesting-depth` rule refuses four levels, and it refused the first draft of this walk for
/// exactly that reason.
fn Is_Worth_Descending(entry: &std::fs::DirEntry) -> bool
{
    if !entry.path().is_dir()
    {
        return false;
    }

    let name = entry.file_name().to_string_lossy().into_owned();

    return name != "target" && !name.starts_with('.');
}

/// This repository's own root, from the test binary's own manifest directory.
fn Repository_Root() -> PathBuf
{
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    return manifest
        .ancestors()
        .find(|candidate| return candidate.join("docs/records").is_dir())
        .map(Path::to_path_buf)
        .unwrap_or(manifest);
}
