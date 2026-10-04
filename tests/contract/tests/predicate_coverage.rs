//! The predicate-coverage declaration names paths this repository has and packages that are its
//! members, so a rename cannot leave its rule silently matching nothing.
//!
//! `nomos-predicate-coverage.json` declares that an item whose territory reaches certain paths
//! must carry a predicate naming the packages that enumerate the composed set, and `work add`
//! and `work widen` refuse one that does not. `OD-GATE-036` decided both the rule and what keeps
//! it honest: "existence, not completeness". Both its lists are kept by hand. A declared path
//! renamed away reaches nothing, so the rule stops firing with nothing said; a declared package
//! renamed away is an argument every predicate is told to carry and none can, since cargo
//! refuses a package it does not have. This test reads the committed file through the ledger's
//! own reader, so a file that reader refuses fails here too, and holds each list against the
//! real tree.
//!
//! Whether the lists are complete is not tested and cannot be. A dependent red measured outside
//! them is what amends them, as `OD-GATE-036`'s "What Would Reopen It" says.
//!
//! The ledger compares argument tokens and interprets no build tool. This test is where the
//! interpretation lives instead: in this repository every argument a rule requires is the name
//! of a package, so each is held against `cargo metadata`'s member list. An argument that
//! satisfies a rule alone, such as `--workspace`, names no package and is not held to one.

use std::path::Path;

use nomos_contract_tests::{Repository_Root, Workspace};
use nomos_ledger::{CoverageRule, PREDICATE_COVERAGE, PredicateCoverage};

/// Where this repository's decision records live, relative to its root.
const RECORD_DIRECTORY: &str = "docs/records";

/// The rules `text` declares, read by the ledger's own reader.
///
/// # Panics
///
/// If the reader refuses the text, or the text declares no rule. A declaration of nothing
/// would make every assertion below pass over an empty list, which is this test passing in its
/// own worst case.
fn Rules_In(text: &str) -> Vec<CoverageRule>
{
    let declared = PredicateCoverage::From_Json(text)
        .unwrap_or_else(|cause| panic!("{PREDICATE_COVERAGE} is refused by the reader `work add` uses: {cause}"));
    let PredicateCoverage::Declared(rules) = declared
    else
    {
        panic!("From_Json answers declared rules or an error, and answered neither: {declared:?}");
    };
    assert!(!rules.is_empty(), "{PREDICATE_COVERAGE} declares no rule, so nothing here would be checked");

    return rules;
}

/// The committed declaration's rules.
fn Committed_Rules(root: &Path) -> Vec<CoverageRule>
{
    let text = std::fs::read_to_string(root.join(PREDICATE_COVERAGE))
        .unwrap_or_else(|error| panic!("the repository root holds {PREDICATE_COVERAGE}: {error}"));

    return Rules_In(&text);
}

/// Every declared path that names neither a file nor a directory under `root`, in declaration
/// order.
fn Paths_That_Are_Gone(root: &Path, rules: &[CoverageRule]) -> Vec<String>
{
    return rules
        .iter()
        .flat_map(|rule| return rule.paths.iter())
        .filter(|path| return !root.join(path).exists())
        .cloned()
        .collect();
}

/// Every required argument that is not the name of a member of the workspace, in declaration
/// order.
fn Arguments_That_Name_No_Member(rules: &[CoverageRule], members: &[String]) -> Vec<String>
{
    return rules
        .iter()
        .flat_map(|rule| return rule.requires.iter())
        .filter(|argument| return !members.contains(argument))
        .cloned()
        .collect();
}

/// Every record a rule names that the record directory carries no file for.
///
/// A record file is named for its identifier and then a slug, so a rule naming `OD-GATE-036` is
/// satisfied by `OD-GATE-036-<slug>.md` and by nothing that merely begins with the same
/// characters, such as `OD-GATE-0361-...`.
fn Records_That_Are_Unpublished(root: &Path, rules: &[CoverageRule]) -> Vec<String>
{
    let published: Vec<String> = std::fs::read_dir(root.join(RECORD_DIRECTORY))
        .unwrap_or_else(|error| panic!("the repository root holds {RECORD_DIRECTORY}: {error}"))
        .filter_map(|entry| return entry.ok()?.file_name().into_string().ok())
        .collect();

    return rules
        .iter()
        .map(|rule| return rule.record.clone())
        .filter(|record| {
            return !published.iter().any(|file| return file.starts_with(&format!("{record}-")));
        })
        .collect();
}

/// The names of every member of this workspace, as `cargo metadata` reports them.
fn Member_Names() -> Vec<String>
{
    return Workspace::Load().Members().iter().map(|package| return package.name.clone()).collect();
}

/// Every path the committed declaration names is a file or a directory this repository has. A
/// failure names each that is gone, so the rule can be pointed at where it moved.
#[test]
fn Test_Every_Declared_Path_Should_Exist()
{
    let root = Repository_Root();

    let gone = Paths_That_Are_Gone(&root, &Committed_Rules(&root));

    assert!(gone.is_empty(), "{PREDICATE_COVERAGE} declares a path this repository does not have, so the rule reaches nothing there: {gone:?}");
}

/// Every argument the committed declaration requires names a member of this workspace. A
/// failure names each that does not, which every predicate would be told to carry and none
/// could.
#[test]
fn Test_Every_Declared_Package_Should_Be_A_Workspace_Member()
{
    let root = Repository_Root();

    let strangers = Arguments_That_Name_No_Member(&Committed_Rules(&root), &Member_Names());

    assert!(strangers.is_empty(), "{PREDICATE_COVERAGE} requires a package this workspace does not have: {strangers:?}");
}

/// Every rule names a record this repository has published, since the refusal sends an author
/// to read it.
#[test]
fn Test_Every_Rule_Should_Name_A_Published_Record()
{
    let root = Repository_Root();

    let unpublished = Records_That_Are_Unpublished(&root, &Committed_Rules(&root));

    assert!(unpublished.is_empty(), "{PREDICATE_COVERAGE} cites a record {RECORD_DIRECTORY} does not carry: {unpublished:?}");
}

/// The three readers report what is gone and pass what is there, so the tests above are not
/// green because a reader never reports anything.
#[test]
fn Test_Each_Reader_Should_Report_Only_What_Is_Not_There()
{
    let root = Repository_Root();
    let rules = Rules_In(
        r#"{ "rules": [ {
            "record": "OD-GATE-999999",
            "paths": ["crates", "crates/a-directory-this-repository-does-not-have", "README.md"],
            "requires": ["nomos-ledger", "nomos-a-package-this-workspace-does-not-have"]
        } ] }"#,
    );

    assert_eq!(Paths_That_Are_Gone(&root, &rules), vec!["crates/a-directory-this-repository-does-not-have".to_owned()]);
    assert_eq!(
        Arguments_That_Name_No_Member(&rules, &Member_Names()),
        vec!["nomos-a-package-this-workspace-does-not-have".to_owned()]
    );
    assert_eq!(Records_That_Are_Unpublished(&root, &rules), vec!["OD-GATE-999999".to_owned()]);
}
