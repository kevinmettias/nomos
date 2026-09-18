use crate::readers::{
    Board, Crossing_Counts, Declared_Skill_Name, Gate_Run_Commands, Imports, Ledger_Verb_Lines,
    Missing_Paths, Named_Items, Restated_Gate_Commands, Restated_Ledger_Verb_Lines,
    Restated_Rows,
};

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
    Assert_The_Gate_Command_Check_Reports_A_Pasted_List();
    Assert_The_Ledger_Verb_Check_Reports_A_Pasted_Reference();
    Assert_The_Crossing_Count_Check_Reports_A_Restated_Count();
}

/// Shown reporting a link to nowhere, and shown not reporting a command or a real path.
pub(crate) fn Assert_The_Route_Check_Reports_A_Broken_Link()
{
    use nomos_contract_tests::Workspace;

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
    use std::collections::BTreeSet;

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
    use crate::CONTRACT;

    assert!(
        !Imports("# Claude Code\n\nThe bands are as follows.", CONTRACT),
        "an adapter with no import was accepted, so it could carry its own contract"
    );
}

/// Shown reporting two pasted gate commands together, and shown not reporting one named in
/// passing.
pub(crate) fn Assert_The_Gate_Command_Check_Reports_A_Pasted_List()
{
    let commands = Gate_Run_Commands(
        "      - name: Lint\n        run: cargo clippy --workspace --all-targets -- -D warnings\n\
         \x20     - name: Test\n        run: cargo test --workspace\n",
    );

    assert_eq!(
        commands.len(),
        2,
        "the fixture workflow has two `run:` lines, and the reader found {}",
        commands.len()
    );
    assert!(
        Restated_Gate_Commands(
            "Run the lint and the test step yourself: `cargo clippy --workspace --all-targets \
             -- -D warnings` and then `cargo test --workspace`.",
            &commands
        )
        .len()
            >= 2,
        "a paragraph carrying both commands was not reported, so a pasted gate command list \
         could hide inside prose"
    );
    assert!(
        Restated_Gate_Commands("never run `cargo fmt` by hand", &commands).is_empty(),
        "a command that is not one of the gate's own was reported as if it were"
    );
}

/// Shown reporting two pasted ledger verb lines together, and shown not reporting one named
/// in a sentence.
pub(crate) fn Assert_The_Ledger_Verb_Check_Reports_A_Pasted_Reference()
{
    let readme = "nomos work list [--state ready|claimed|blocked|done|declined]\n\
                   nomos work claim   --item <id> --holder <name> [--lease 2h]\n\
                   nomos work validate\n";
    let verbs = Ledger_Verb_Lines(readme);

    assert_eq!(
        verbs.len(),
        3,
        "the fixture README has three verb lines, and the reader found {}",
        verbs.len()
    );
    assert!(
        Restated_Ledger_Verb_Lines(
            "```\n\
             nomos work list [--state ready|claimed|blocked|done|declined]\n\
             nomos work claim   --item <id> --holder <name> [--lease 2h]\n\
             ```\n",
            &verbs
        )
        .len()
            >= 2,
        "a pasted block carrying two verb lines was not reported, so the reference could be \
         copied wholesale"
    );
    assert!(
        Restated_Ledger_Verb_Lines(
            "`nomos work validate` prints the file's schema beside the build's.",
            &verbs
        )
        .is_empty(),
        "a verb named in a sentence, not on a line of its own, was reported as a copy of the \
         reference"
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

/// Shown reporting the count restated about the crossing, and shown leaving prose about that
/// crossing alone when it states no number.
///
/// The first fixture is the retired sentence itself, verbatim, because the check that reads
/// for it was written after it had already gone stale: a fixture invented here would prove
/// the reader works on a shape that never occurred.
pub(crate) fn Assert_The_Crossing_Count_Check_Reports_A_Restated_Count()
{
    assert!(
        !Crossing_Counts(
            "**This workspace does not build without the `xvpe` checkout beside it.** Six \
             crates name an `xvpe-*` dependency by a relative path that climbs out of this \
             repository into a sibling directory named `xvpe`, and ten reach one \
             transitively (measured 2026-09-11)."
        )
        .is_empty(),
        "the contract's own retired sentence was accepted, so the check would let a \
         crossing count be restated in the contract again"
    );
    assert!(
        Crossing_Counts(
            "The workspace resolves XVPE from a pinned revision. A local override is opt-in, \
             is requested by naming it, and is never the governing form."
        )
        .is_empty(),
        "prose about the crossing that states no count was reported as one, so the check \
         would forbid the contract from saying anything about XVPE at all"
    );
}
