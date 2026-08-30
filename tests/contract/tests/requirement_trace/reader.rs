//! The reader refuses rather than skips.
//!
//! A skipped entry is an assessment that quietly stops being one — `OD-SPEC-005`'s defect
//! in this registry's clothes — so the reader has no lenient path at all.

use crate::registry::{Entries, Is_Requirement_Id, Parse};

/// Every refusal, asserted as an `Err` rather than as a short `Ok`.
#[test]
fn Test_The_Reader_Should_Refuse_Every_Malformed_Entry()
{
    let sound = "verdict: Met\nsite: README.md#Nomos\n";
    assert!(
        Parse("CHK-003", sound).is_ok(),
        "the well-formed case must parse, or every refusal below proves only that the \
         reader refuses everything"
    );

    let partial_sound = "verdict: Partial\nsite: README.md#Nomos\ngap: README.md#Nomos\n";
    assert!(
        Parse("CHK-003", partial_sound).is_ok(),
        "a Partial entry naming both a site and a gap must parse, or every Partial \
         refusal below proves only that the reader refuses every Partial"
    );

    for (stem, text, because) in MALFORMED
    {
        assert!(
            Parse(stem, text).is_err(),
            "the reader accepted an entry with {because}"
        );
    }
}

/// Every shape the reader must refuse, and what is wrong with each.
///
/// A table rather than one test apiece. They are one rule — the reader has no lenient path —
/// and fourteen near-identical test functions would state it fourteen times.
const MALFORMED: &[(&str, &str, &str)] = &[
    ("CHK-003", "site: README.md#Nomos\n", "no verdict"),
    ("CHK-003", "verdict: Met\n", "no site"),
    (
        "CHK-003",
        "verdict: Unassessed\nsite: README.md#Nomos\n",
        "Unassessed is held by the absence of an entry and must not be writable",
    ),
    (
        "CHK-003",
        "verdict: Satisfied\nsite: README.md#Nomos\n",
        "a verdict outside the three",
    ),
    (
        "CHK-003",
        "verdict: Met\nverdict: Diverges\nsite: README.md#Nomos\n",
        "two verdicts",
    ),
    (
        "CHK-003",
        "verdict: Met\nrecord: OD-TRACE-001\nrecord: OD-SPEC-007\nsite: README.md#Nomos\n",
        "two records",
    ),
    ("CHK-003", "verdict: Met\nsite: README.md\n", "a site with no symbol"),
    ("CHK-003", "verdict: Met\nsite: #Nomos\n", "a site with no path"),
    (
        "CHK-003",
        "verdict: Met\nsite: /absolute.rs#Nomos\n",
        "a site that is not repo-relative",
    ),
    (
        "CHK-003",
        "verdict: Met\nsite: ../outside.rs#Nomos\n",
        "a site reaching outside the workspace",
    ),
    (
        "CHK-003",
        "verdict: Met\nid: CHK-003\nsite: README.md#Nomos\n",
        "an unknown key — an id: line would be a second place for the identity to be wrong, \
         which is the rule OD-SPEC-007 set for a record registration",
    ),
    (
        "CHK-003",
        "verdict Met\nsite: README.md#Nomos\n",
        "a line that is not a key",
    ),
    (
        "chk-3",
        "verdict: Met\nsite: README.md#Nomos\n",
        "a stem that is not a requirement identifier",
    ),
    (
        "US-CHK-001",
        "verdict: Met\nsite: README.md#Nomos\n",
        "a User Story identifier; OD-TRACE-004 keeps a User Story out of this registry's \
         assessable population",
    ),
    (
        "CHK-003",
        "verdict: Diverges\nsite: README.md#Nomos\n",
        "a divergence with no record, refused at read time as well as compared",
    ),
    (
        "CHK-003",
        "verdict: Partial\nsite: README.md#Nomos\n",
        "a partial with no gap, refused at read time as well as compared",
    ),
    (
        "CHK-003",
        "verdict: Partial\nsite: README.md#Nomos\ngap: README.md\n",
        "a gap with no symbol, refused the same way a site with no symbol is",
    ),
];

/// Well-formed requirement identifiers: a family and a number, nothing more.
const REQUIREMENT_IDS: &[&str] = &["CHK-003", "EVID-001", "CAP-002", "WORK-LEDGER-005"];

/// Near misses, each wrong in one way a lenient stem check would let through.
const NOT_REQUIREMENT_IDS: &[&str] =
    &["CHK-3", "CHK-0003", "chk-003", "003", "CHK-", "-003", "CHK-00A", "US-CHK-001"];

/// The identifier is the stem, and nothing inside the file may restate it.
#[test]
fn Test_A_Requirement_Identifier_Should_Be_A_Family_And_A_Number()
{
    for accepted in REQUIREMENT_IDS
    {
        assert!(Is_Requirement_Id(accepted), "{accepted} is a requirement id");
    }

    for refused in NOT_REQUIREMENT_IDS
    {
        assert!(!Is_Requirement_Id(refused), "{refused} is not a requirement id");
    }
}

/// The reader enumerates the directory it is given, not one it knows about.
///
/// `OD-SPEC-007` made the same check structural for record registrations, for the reason it
/// states: a reader that reaches for a fixed path cannot be handed a constructed input, and
/// a guard whose input cannot be constructed is a guard nobody can write a control for.
#[test]
fn Test_The_Reader_Should_Enumerate_The_Directory_It_Is_Given()
{
    use crate::assessment::REGISTRY;
    use nomos_contract_tests::Workspace;

    let root = Workspace::Workspace_Root();
    // Non-empty rather than at the floor. How many entries there are is
    // `Test_The_Assessed_Set_Should_Not_Shrink`'s fact, and asserting it here as well would
    // give one deletion two red tests — measured, when removing a single entry reddened
    // both. This one owes only that the reader read the directory it was handed.
    let real = Entries(&root.join(REGISTRY)).expect("the committed registry must read");
    assert!(!real.is_empty(), "the committed registry read as empty");

    let empty = Entries(&root.join("tests/contract/surface"))
        .expect("a directory holding no entry is empty rather than an error");
    assert!(
        empty.is_empty(),
        "the reader answered about the registry while being handed another directory, so \
         no control can ever construct an input for it"
    );
}
