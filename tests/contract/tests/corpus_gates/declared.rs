use crate::scan::{Files_Naming_A_Corpus, Variables_Read_By};
use crate::{Gate, GATED_TOTAL, GATES, TESTS_IN_GATED_FILES, VARIABLES};
use nomos_contract_tests::Workspace;
use std::collections::BTreeSet;
use std::path::Path;

/// The table accounts for every file that reaches a corpus.
///
/// The direction that matters. A test file added tomorrow that reads one of these variables
/// is invisible in a green suite — it will skip, print `ok`, and never appear in any count —
/// unless something goes looking for it. This goes looking.
#[test]
fn Test_Every_File_That_Reaches_A_Corpus_Should_Be_Declared()
{
    let root = Workspace::Workspace_Root();
    let found = Files_Naming_A_Corpus(&root);

    assert!(
        !found.is_empty(),
        "no test file in this workspace names any of {VARIABLES:?}.\n\
         Every assertion in this file is about that set, so an empty one passes having \
         checked nothing — and this workspace has corpus-gated tests."
    );
    Assert_The_Table_And_The_Tree_Agree(&found);
}

/// Both directions: a file that reaches a corpus and is in no row, and a row naming a file
/// that no longer reaches one.
pub(crate) fn Assert_The_Table_And_The_Tree_Agree(found: &BTreeSet<String>)
{
    let declared: BTreeSet<&str> = GATES.iter().map(|gate| return gate.path).collect();
    let actual: BTreeSet<&str> = found.iter().map(String::as_str).collect();
    let undeclared: Vec<&&str> = actual.difference(&declared).collect();
    let vanished: Vec<&&str> = declared.difference(&actual).collect();

    assert!(
        undeclared.is_empty(),
        "these test files reach a corpus and are not in GATES: {undeclared:#?}.\n\
         A gated test that nothing counts is a test that reports ok on every machine \
         without a corpus, which is every CI machine. Add it to the table with what it \
         gates."
    );
    assert!(
        vanished.is_empty(),
        "GATES names these files and they no longer reach a corpus: {vanished:#?}.\n\
         A table that over-reports is as useless as one that under-reports: it inflates \
         the size of the hole and nobody trusts the number."
    );
}

/// Each declared file holds the number of tests the table says, and reads the variables it
/// says.
///
/// This is what makes a *new test inside an already-declared file* visible. Without it the
/// table would be satisfied forever by listing the fourteen files once, and a fifteenth
/// gated assertion could be added to any of them and counted by nobody.
#[test]
fn Test_Every_Declared_File_Should_Match_What_It_Declares()
{
    let root = Workspace::Workspace_Root();
    let mut wrong = Vec::new();
    for gate in GATES
    {
        let disagreements = Disagreements_With(&root, gate);

        Assert_The_Row_Is_Internally_Consistent(gate);
        wrong.extend(disagreements);
    }

    assert!(
        wrong.is_empty(),
        "the table disagrees with the source: {wrong:#?}.\n\
         Update GATES deliberately. A test added to a gated file inherits that file's \
         silence, and whether it should is a decision rather than a consequence."
    );
}

/// A row cannot gate more tests than its own file holds.
pub(crate) fn Assert_The_Row_Is_Internally_Consistent(gate: &Gate)
{
    assert!(
        gate.gated <= gate.tests,
        "{} declares {} gated of {} tests",
        gate.path,
        gate.gated,
        gate.tests
    );
}

/// What one row says about its file, against what the file holds.
pub(crate) fn Disagreements_With(root: &Path, gate: &Gate) -> Vec<String>
{
    let text = Text_Of(root, gate.path);
    let tests = text
        .lines()
        .filter(|line| return line.trim() == "#[test]")
        .count();
    let named = Variables_Read_By(&text);
    let mut wrong = Vec::new();
    if tests != gate.tests
    {
        wrong.push(format!("{}: declares {} tests, has {tests}", gate.path, gate.tests));
    }
    if named != gate.variables
    {
        wrong.push(format!("{}: declares {:?}, reads {named:?}", gate.path, gate.variables));
    }

    return wrong;
}

/// One of this repository's own files, which must be readable.
pub(crate) fn Text_Of(root: &Path, relative: &str) -> String
{
    let path = root.join(relative);

    return std::fs::read_to_string(&path)
        // The path comes from the GATES table, so a file that will not open means the table
        // names a gate file that is no longer there. Read as empty text it would surface as
        // "declares N tests, has 0" — a count mismatch reported where the path is the defect.
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
}

/// The headline is the table's own sum.
#[test]
fn Test_The_Declared_Total_Should_Be_The_Sum_Of_The_Table()
{
    let gated = GATES
        .iter()
        .fold(0_usize, |running, gate| return running.saturating_add(gate.gated));
    let tests = GATES
        .iter()
        .fold(0_usize, |running, gate| return running.saturating_add(gate.tests));

    assert_eq!(
        gated, GATED_TOTAL,
        "GATED_TOTAL is cited as the number of assertions a corpus-less run does not make, \
         and the table sums to {gated}"
    );
    assert_eq!(tests, TESTS_IN_GATED_FILES);
    assert!(
        gated < tests,
        "every test in every gated file is gated, which would mean the files hold nothing \
         that runs without a corpus — check the table rather than believing it"
    );
}
