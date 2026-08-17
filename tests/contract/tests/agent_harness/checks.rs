use nomos_contract_tests::Workspace;
use nomos_ledger::LedgerDocument;
use std::collections::BTreeSet;
use std::path::Path;
use crate::readers::{
    Board, Declared_Skill_Name, Gate_Run_Commands, Harness_Files, Imports, Ledger_Verb_Lines,
    Missing_Paths, Named_Items, Named_Paths, Read_Harness_File, Read_Repo_File,
    Restated_Gate_Commands, Restated_Ledger_Verb_Lines, Restated_Rows, Skill_Directories,
};
use crate::{
    ADAPTER, ADAPTER_LINE_BUDGET, CONTRACT, CONTRACT_LINE_BUDGET, GATE_WORKFLOW,
    LEDGER_VERB_REFERENCE, ROUTED_AUTHORITIES, TEMPORARY_HAZARDS,
};

/// The front door has to be at the door.
///
/// Claude Code and Codex both look for a file by name before they look at anything else,
/// and neither existed here. Empty counts as absent: a file present and saying nothing
/// routes an agent nowhere while making the absence undetectable.
#[test]
fn Test_The_Repository_Should_Have_An_Agent_Front_Door()
{
    for name in [CONTRACT, ADAPTER]
    {
        let text = Read_Harness_File(name);

        assert!(
            text.lines().filter(|line| return !line.trim().is_empty()).count() > 5,
            "{name} exists and says almost nothing, which is indistinguishable from its \
             absence to the agent reading it"
        );
    }
}

/// The adapter imports the contract; it does not grow a second one.
///
/// Two independently maintained instruction files is the drift `OD-AGENT-001` refuses,
/// and it is worse than one stale file because the two disagree without either being
/// wrong on its face.
#[test]
fn Test_The_Adapter_Should_Import_The_Contract_Rather_Than_Repeat_It()
{
    let adapter = Read_Harness_File(ADAPTER);

    assert!(
        Imports(&adapter, CONTRACT),
        "{ADAPTER} does not import {CONTRACT}. An adapter that restates the contract is a \
         second authority, and OD-AGENT-001 admits only vendor mechanics here."
    );

    let lines = adapter.lines().count();
    assert!(
        lines <= ADAPTER_LINE_BUDGET,
        "{ADAPTER} is {lines} lines against a budget of {ADAPTER_LINE_BUDGET}. Everything \
         it might restate is one import away, so length here is the tell."
    );
}

/// Routing does not need the room, so the budget is what keeps it routing.
#[test]
fn Test_The_Contract_Should_Stay_Inside_Its_Line_Budget()
{
    let lines = Read_Harness_File(CONTRACT).lines().count();

    assert!(
        lines <= CONTRACT_LINE_BUDGET,
        "{CONTRACT} is {lines} lines against a budget of {CONTRACT_LINE_BUDGET}. The limit \
         is the mechanism OD-AGENT-001 relies on: a contract that grows past it has started \
         restating something, one reasonable paragraph at a time."
    );
}

/// A route to a file somebody moved is a route to nothing.
///
/// This is the half of correctness a machine can check. It runs over the skills too, so a
/// path named inside a procedure is held to the same standard as one in the contract.
#[test]
fn Test_Every_Path_The_Harness_Names_Should_Exist()
{
    let root = Workspace::Workspace_Root();
    let mut broken = Vec::new();

    for (name, text) in Harness_Files()
    {
        for missing in Missing_Paths(&text, &root)
        {
            broken.push(format!("{name} names {missing}, which is not in the tree"));
        }
    }

    assert!(
        broken.is_empty(),
        "the harness routes to paths that do not exist: {broken:#?}.\n\
         A path that has moved is the failure this check exists for; a path that is real \
         but uncommitted is the other one, and CI is where it shows."
    );
}

/// The band table is checked where it lives, and nowhere else may hold a copy.
///
/// `tests/contract/tests/boundaries.rs` compares the README's rows against the workspace
/// in both directions. A second copy in a file nobody reviews would be unchecked, and
/// would go stale exactly the way the README already had when `OD-PROJECT-001` found it:
/// twenty-two members described by eleven rows.
#[test]
fn Test_The_Harness_Should_Not_Restate_The_Band_Table()
{
    let workspace = Workspace::Load();
    let members: BTreeSet<String> = workspace
        .Members()
        .into_iter()
        .map(|member| return member.name.clone())
        .collect();

    assert!(
        !members.is_empty(),
        "no workspace members were resolved, so this check would pass over any file at all"
    );

    let restated = Restated_Anywhere(&members);

    assert!(
        restated.is_empty(),
        "the harness carries table rows naming workspace crates: {restated:#?}.\n\
         OD-AGENT-001 admits a link to README.md and refuses a copy of it."
    );
}

/// Every band row any harness file carries, labelled with the file that carries it.
pub(crate) fn Restated_Anywhere(members: &BTreeSet<String>) -> Vec<String>
{
    let mut restated = Vec::new();
    for (name, text) in Harness_Files()
    {
        for row in Restated_Rows(&text, members)
        {
            restated.push(format!("{name}: {row}"));
        }
    }

    return restated;
}

/// The gate's command list is checked where it runs, and nowhere else may hold a copy.
///
/// `.github/workflows/gate.yml` is what actually executes the gate, and `OD-AGENT-002`
/// names it as one of the three shapes a handoff — or any other harness file — must not
/// restate. Derived from the workflow's own `run:` lines rather than retyped here, the same
/// way `Restated_Anywhere` derives the crate list from the real workspace instead of a
/// hand-kept set.
///
/// Two or more, not one: a single command named as a routing example — `AGENTS.md` already
/// does this for `nomos work validate` below — is a reference, not a copy of the list. Two
/// together is the shape a paste takes.
#[test]
fn Test_The_Harness_Should_Not_Restate_The_Gate_Command_List()
{
    let commands = Gate_Run_Commands(&Read_Repo_File(GATE_WORKFLOW));

    assert!(
        !commands.is_empty(),
        "no `run:` line was found in {GATE_WORKFLOW}, so this check would pass over any \
         file at all"
    );

    let mut restated = Vec::new();
    for (name, text) in Harness_Files()
    {
        let found = Restated_Gate_Commands(&text, &commands);

        if found.len() >= 2
        {
            restated.push(format!("{name}: {found:#?}"));
        }
    }

    assert!(
        restated.is_empty(),
        "the harness carries two or more of the gate's own commands: {restated:#?}.\n\
         OD-AGENT-002 admits naming one command in passing and refuses a copy of the list \
         `.github/workflows/gate.yml` already runs."
    );
}

/// The ledger verb reference is checked where `README.md` already documents it, and nowhere
/// else may hold a copy.
///
/// `OD-AGENT-002` names this as the third shape a handoff must not restate, alongside the
/// crate table and the gate command list. Matched whole line to whole line, because a
/// sentence mentioning one verb in a code span is a routing reference and only a pasted
/// block of the usage lines is the reference itself.
#[test]
fn Test_The_Harness_Should_Not_Restate_The_Ledger_Verb_Reference()
{
    let verbs = Ledger_Verb_Lines(&Read_Repo_File(LEDGER_VERB_REFERENCE));

    assert!(
        !verbs.is_empty(),
        "no `nomos work <verb>` usage line was found in {LEDGER_VERB_REFERENCE}, so this \
         check would pass over any file at all"
    );

    let mut restated = Vec::new();
    for (name, text) in Harness_Files()
    {
        let found = Restated_Ledger_Verb_Lines(&text, &verbs);

        if found.len() >= 2
        {
            restated.push(format!("{name}: {found:#?}"));
        }
    }

    assert!(
        restated.is_empty(),
        "the harness carries two or more lines from the ledger verb reference: \
         {restated:#?}.\n\
         OD-AGENT-002 admits naming one verb in passing and refuses a copy of the block \
         README.md already documents."
    );
}

/// Routing is the contract's whole job, so every authority it routes to is required.
///
/// Named as paths rather than as prose, because a path is what an agent can open. None of
/// the four is inferable from the tree by an arriving session: the ledger is not
/// discoverable from the source, a record explains a decision the code only shows the
/// result of, and the pair that says what the workspace is and what checks it are the two
/// this file spent its whole existence refusing to copy — so the route to them has to hold.
#[test]
fn Test_The_Contract_Should_Name_Every_Authority_It_Routes_To()
{
    let named: BTreeSet<String> = Named_Paths(&Read_Harness_File(CONTRACT)).into_iter().collect();

    for authority in ROUTED_AUTHORITIES
    {
        assert!(
            named.iter().any(|path| return path.starts_with(authority)),
            "{CONTRACT} never names {authority}. An agent that does not find the board \
             invents one, an agent that does not find the records repeats a decision \
             somebody already made, and an agent that does not find the workspace \
             description infers the architecture from whatever code is nearest."
        );
    }
}

/// A hazard that outlives its defect is a step every session spends on nothing.
///
/// Both directions, because each catches the failure the other cannot see. The sentence
/// must still be there while the item is open, so a warning cannot be deleted early. The
/// item must still be open, so a warning cannot be left behind — and the day
/// `P10-STALE-WRITER` is finished, this test is what says so and where.
#[test]
fn Test_Every_Temporary_Hazard_Should_Name_An_Item_That_Is_Still_Open()
{
    let board = Board();

    for (file, item) in TEMPORARY_HAZARDS
    {
        Assert_The_Hazard_Is_Still_Live(&board, file, item);
    }
}

/// The sentence must still be there while the item is open, and the item must still be open
/// while the sentence is there.
pub(crate) fn Assert_The_Hazard_Is_Still_Live(board: &LedgerDocument, file: &str, item: &str)
{
    let text = Read_Harness_File(file);

    assert!(
        Named_Items(&text).iter().any(|named| return named == item),
        "{file} is declared to carry the {item} hazard and no longer mentions it. \
         Either the warning was removed while the defect is still open, or this \
         declaration should have gone with it."
    );

    let Some(entry) = board.items.iter().find(|entry| return entry.id.As_Str() == item)
    else
    {
        // With no entry there is nothing for the finished-state assertion below to run
        // against. Returning quietly instead would let a hazard declaration cite an id that
        // is on no board and still pass, which is the one arrangement nobody can retire.
        panic!("{file} names {item}, which is on no board. A hazard pointing at an \
                item nobody can look up cannot be retired by anybody.");
    };
    assert!(
        !entry.state.Is_Finished(),
        "{item} is finished, so the hazard {file} carries for it is over. Remove the \
         warning and its entry here — OD-AGENT-001 admits a temporary hazard on \
         exactly this condition."
    );
}

/// A hazard cannot be added quietly, which is what makes the declaration above worth having.
///
/// The derived direction. Without it the table is a list somebody remembers to update, and
/// `OD-COMPLETENESS-001` is this workspace's record of what that is worth.
#[test]
fn Test_Every_Item_The_Harness_Names_Should_Be_Declared_As_A_Temporary_Hazard()
{
    let undeclared = Items_Named_Without_A_Declaration();

    assert!(
        undeclared.is_empty(),
        "the harness names ledger items that are not declared as temporary hazards: \
         {undeclared:#?}.\n\
         An item id in an instruction file is a promise that the sentence around it \
         expires; declaring it is how the expiry is noticed."
    );
}

/// Every item a harness file names that the table does not declare for that file.
pub(crate) fn Items_Named_Without_A_Declaration() -> Vec<String>
{
    let mut undeclared = Vec::new();
    for (file, text) in Harness_Files()
    {
        for item in Named_Items(&text)
        {
            if !Declared_Hazard(&file, &item)
            {
                undeclared.push(format!("{file} names {item}"));
            }
        }
    }

    return undeclared;
}

/// Whether the table declares this file as carrying this item's hazard.
pub(crate) fn Declared_Hazard(file: &str, item: &str) -> bool
{
    return TEMPORARY_HAZARDS
        .iter()
        .any(|(declared_file, declared_item)| {
            return *declared_file == file && *declared_item == item;
        });
}

/// A skill is addressed by name, and the name it declares must be the one it is found at.
///
/// A mismatch is not cosmetic: the directory is how the tool finds it and the front matter
/// is how the tool describes it, so the two disagreeing produces a skill that is invoked
/// under one name and reports itself under another.
#[test]
fn Test_Every_Committed_Skill_Should_Declare_A_Name_Matching_Its_Directory()
{
    for directory in Skill_Directories()
    {
        Assert_The_Skill_Is_Named_For_Its_Directory(&directory);
    }
}

/// The directory is how the tool finds a skill and the front matter is how the tool describes
/// it, so the two disagreeing produces a skill invoked under one name and reported under
/// another.
pub(crate) fn Assert_The_Skill_Is_Named_For_Its_Directory(directory: &Path)
{
    let label = directory.display().to_string().replace('\\', "/");
    let manifest = directory.join("SKILL.md");

    assert!(
        manifest.is_file(),
        "{label} is a skill directory with no SKILL.md, so it is a directory the tool \
         will not load and a reader will assume works"
    );
    let text = std::fs::read_to_string(&manifest)
        // The assertion above already established this manifest is a file, so a read that
        // fails here is a SKILL.md the tool would fail to load too. There is no name left to
        // compare and no weaker comparison to fall back to.
        .unwrap_or_else(|error| panic!("cannot read {label}/SKILL.md: {error}"));
    let declared = Declared_Skill_Name(&text)
        // Front matter with no name is worse than front matter with the wrong one: the tool
        // then has nothing to report the skill under at all. Reading the absence as "nothing
        // to compare" would pass it through the check that exists to pin the name down.
        .unwrap_or_else(|| panic!("{label}/SKILL.md declares no name in its front matter"));
    let expected = directory
        .file_name()
        .map(|name| return name.to_string_lossy().into_owned())
        .unwrap_or_default();

    assert_eq!(
        declared, expected,
        "{label}/SKILL.md calls itself {declared} while living at {expected}"
    );
}
