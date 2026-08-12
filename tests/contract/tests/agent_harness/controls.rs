use crate::readers::{
    Board, Declared_Skill_Name, Imports, Missing_Paths, Named_Items, Restated_Rows,
};
use crate::CONTRACT;
use nomos_contract_tests::Workspace;
use std::collections::BTreeSet;

/// Every check above passes over a file that says nothing, so each is shown failing.
///
/// Over fixtures rather than over the tree, because the alternative is a test that edits
/// the repository to prove a point and leaves it edited when it fails partway.
#[test]
fn Test_Every_Check_Here_Should_Fail_On_A_Fixture_That_Breaks_It()
{
    Assert_The_Route_Check_Reports_A_Broken_Link();
    Assert_The_Band_Check_Reports_A_Pasted_Row();
    Assert_The_Import_Check_Reports_An_Adapter_That_Imports_Nothing();
    Assert_The_Skill_Name_Reader_Reports_A_Manifest_Without_Front_Matter();
    Assert_The_Item_Reader_Reports_An_Id_And_Nothing_Else();
}

/// Shown reporting a link to nowhere, and shown not reporting a command or a real path.
pub(crate) fn Assert_The_Route_Check_Reports_A_Broken_Link()
{
    let root = Workspace::Workspace_Root();

    assert!(
        !Missing_Paths("routes to `docs/records/there-is-no-such-record.md`", &root).is_empty(),
        "a named path that is not in the tree was not reported, so the routing check would \
         accept a link to anywhere"
    );
    assert!(
        Missing_Paths("run `cargo fmt` and read `README.md`", &root).is_empty(),
        "a command span or a real path was mistaken for a broken route"
    );
}

/// Shown reporting a pasted row, and shown not reporting prose that names a crate.
pub(crate) fn Assert_The_Band_Check_Reports_A_Pasted_Row()
{
    let members: BTreeSet<String> = ["nomos-rules".to_owned()].into_iter().collect();

    assert!(
        !Restated_Rows("| 30 | `nomos-rules` | A rule is a pure function. |", &members).is_empty(),
        "a pasted band row was not reported, so the band table could be copied here"
    );
    assert!(
        Restated_Rows("The rule crate is `nomos-rules`, in README.md.", &members).is_empty(),
        "prose naming a crate was reported as a restated row"
    );
}

/// Shown refusing an adapter that carries its own contract instead of importing one.
pub(crate) fn Assert_The_Import_Check_Reports_An_Adapter_That_Imports_Nothing()
{
    assert!(
        !Imports("# Claude Code\n\nThe bands are as follows.", CONTRACT),
        "an adapter with no import was accepted, so it could carry its own contract"
    );
}

/// Shown reading a declared name, and shown reporting none where there is no front matter.
pub(crate) fn Assert_The_Skill_Name_Reader_Reports_A_Manifest_Without_Front_Matter()
{
    assert_eq!(
        Declared_Skill_Name("---\nname: nomos-task\ndescription: x\n---\n").as_deref(),
        Some("nomos-task"),
        "a declared skill name was not read back out of its own front matter"
    );
    assert_eq!(
        Declared_Skill_Name("# nomos-task\n\nNo front matter here.\n"),
        None,
        "a manifest with no front matter reported a name, so the check would compare \
         nothing against the directory"
    );
}

/// Shown recognising an id, shown refusing three near misses, and shown that the board really
/// does hold a finished item for the expiry check to have something to catch.
pub(crate) fn Assert_The_Item_Reader_Reports_An_Id_And_Nothing_Else()
{
    assert_eq!(
        Named_Items("until `P10-STALE-WRITER` closes, confirm the write survived"),
        vec!["P10-STALE-WRITER".to_owned()],
        "an item id in a code span was not recognised, so a temporary hazard could be \
         added without an expiry"
    );
    assert!(
        Named_Items("read `README.md`, then run P10 and OD-AGENT-001 through work list")
            .is_empty(),
        "a path, a bare phase and a record id were read as ledger items, which would put \
         entries in the hazard table that can never be retired"
    );
    assert!(
        Board().items.iter().any(|entry| return entry.state.Is_Finished()),
        "no finished item was found on the board, so the expiry check has never been \
         shown the state it exists to catch"
    );
}
