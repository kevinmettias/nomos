//! What this module promises, exercised.

use super::*;
use crate::ChangeSource;

/// The byte a fixture configuration digest repeats across its length. Which byte it is
/// carries no meaning; that every fixture fills with the same one is what makes the two
/// configurations in this file comparable.
const FIXTURE_CONFIGURATION_BYTE: u8 = 0x11;

/// How many files the checkout fixture writes in one set, and therefore how many effects
/// that one generation is expected to carry.
const CHECKOUT_FILES: usize = 3;

/// How many changes this file applies before it edits one file back: the generation a
/// workspace stands at once that many have landed.
const CHANGES_BEFORE_THE_EDIT_BACK: u64 = 3;

/// How many absolute spellings of a path the door must refuse: one arm of
/// [`Absolute_Paths`] each.
const ABSOLUTE_PATH_SPELLINGS: usize = 4;

/// How many spellings of reaching outside the workspace the door must refuse: one arm of
/// [`Escaping_Paths`] each.
const ESCAPING_PATH_SPELLINGS: usize = 3;

/// How wide the window is that compares every neighbouring pair of workspace identities.
/// Two, because a pair is the smallest window in which a difference can show.
const NEIGHBOURING_PAIR: usize = 2;

fn Variant() -> BuildVariant
{
    return BuildVariant::New("x86_64-unknown-linux-gnu", "dev", "1.85", ["analysis"]);
}

fn Fresh() -> Workspace
{
    return Workspace::Empty(
        Variant(),
        ConfigurationId::From_Digest(Digest128::From_Bytes([
            FIXTURE_CONFIGURATION_BYTE;
            Digest128::BYTE_LENGTH
        ])),
    );
}

/// A workspace-relative path, typed apart from the content written at it so that a caller
/// cannot transpose the two: a fixture that swapped them would compile.
struct MemberPath(&'static str);

fn Edit(path: MemberPath, content: &str) -> WorkspaceChangeSet
{
    return WorkspaceChangeSet::From(ChangeSource::IdeEdit).Present(path.0, content);
}

fn Removal(path: &str) -> WorkspaceChangeSet
{
    return WorkspaceChangeSet::From(ChangeSource::AgentEdit).Absent(path);
}

#[test]
fn Test_One_Applied_Set_Should_Produce_One_Generation()
{
    let mut workspace = Fresh();
    assert_eq!(workspace.Generation(), GenerationId::INITIAL);

    let checkout = WorkspaceChangeSet::From(ChangeSource::GitCheckout)
        .Present("src/a.rs", "pub fn a() {}")
        .Present("src/b.rs", "pub fn b() {}")
        .Present("src/c.rs", "pub fn c() {}");

    let applied = workspace.Apply(&checkout).expect("a checkout applies");

    assert_eq!(applied.Generation(), GenerationId::From_Raw(1));
    assert_eq!(
        workspace.Generation(),
        GenerationId::From_Raw(1),
        "three files in one set is one event and one generation"
    );
    assert_eq!(applied.Effects().len(), CHECKOUT_FILES);
}

/// A save that changed nothing is not a change. Advancing here would invalidate every
/// fact in the store to arrive back at the answer it already had.
#[test]
fn Test_A_Change_That_Says_What_Is_Already_True_Should_Not_Advance()
{
    let mut workspace = Fresh();
    let first = Edit(MemberPath("src/a.rs"),"pub fn a() {}");
    workspace.Apply(&first).expect("every path in this fixture is workspace-relative and distinct, which the door admits");
    let before = workspace.Generation();
    let identity = workspace.Id();

    let redundant = Edit(MemberPath("src/a.rs"),"pub fn a() {}");
    let applied = workspace
        .Apply(&redundant)
        .expect("a redundant save is not an error");

    assert!(matches!(applied, Applied::Unchanged { .. }), "{applied:?}");
    assert_eq!(workspace.Generation(), before);
    assert_eq!(workspace.Id(), identity);
    assert_eq!(
        applied.Effects(),
        &[Effect {
            path: "src/a.rs".to_owned(),
            kind: EffectKind::Redundant
        }]
    );
}

/// The positive control for the test above. If nothing ever advanced, it would pass
/// over a workspace that cannot change at all.
#[test]
fn Test_A_Change_That_Says_Something_New_Should_Advance()
{
    let mut workspace = Fresh();
    let first = Edit(MemberPath("src/a.rs"),"pub fn a() {}");
    workspace.Apply(&first).expect("every path in this fixture is workspace-relative and distinct, which the door admits");
    let before = workspace.Generation();

    let changed = Edit(MemberPath("src/a.rs"),"pub fn changed() {}");
    let applied = workspace.Apply(&changed).expect("every path in this fixture is workspace-relative and distinct, which the door admits");

    assert!(matches!(applied, Applied::Advanced { .. }), "{applied:?}");
    assert!(workspace.Generation() > before);
    assert_eq!(
        applied.Effects(),
        &[Effect {
            path: "src/a.rs".to_owned(),
            kind: EffectKind::Modified
        }]
    );
}

/// Identity is the state, not the history. This is the property that makes a snapshot
/// worth content-addressing at all.
#[test]
fn Test_Editing_A_File_Back_Should_Return_To_The_Same_Snapshot()
{
    let mut workspace = Fresh();
    let first = Edit(MemberPath("src/a.rs"),"original");
    workspace.Apply(&first).expect("every path in this fixture is workspace-relative and distinct, which the door admits");
    let original = workspace.Id();

    let changed = Edit(MemberPath("src/a.rs"),"changed");
    workspace.Apply(&changed).expect("every path in this fixture is workspace-relative and distinct, which the door admits");
    assert_ne!(workspace.Id(), original);

    let back = Edit(MemberPath("src/a.rs"),"original");
    workspace.Apply(&back).expect("every path in this fixture is workspace-relative and distinct, which the door admits");

    assert_eq!(
        workspace.Id(),
        original,
        "the workspace is what it was, however it got back"
    );
    assert_eq!(
        workspace.Generation(),
        GenerationId::From_Raw(CHANGES_BEFORE_THE_EDIT_BACK),
        "and the generation counts what happened, which is three changes"
    );
}

#[test]
fn Test_A_Removal_Should_Take_The_Member_And_A_Second_Should_Not()
{
    let mut workspace = Fresh();
    let first = Edit(MemberPath("src/a.rs"),"pub fn a() {}");
    workspace.Apply(&first).expect("every path in this fixture is workspace-relative and distinct, which the door admits");
    let gone = Removal("src/a.rs");

    let removed = workspace.Apply(&gone).expect("every path in this fixture is workspace-relative and distinct, which the door admits");

    assert!(matches!(removed, Applied::Advanced { .. }));
    assert_eq!(workspace.Content_Of("src/a.rs"), None);

    let again = workspace.Apply(&gone).expect("removing what is not there is not an error");

    assert!(matches!(again, Applied::Unchanged { .. }));
    assert_eq!(
        again.Effects(),
        &[Effect {
            path: "src/a.rs".to_owned(),
            kind: EffectKind::AlreadyAbsent
        }],
        "worth seeing: the submitter has a different idea of what is here"
    );
}

#[test]
fn Test_An_Empty_Change_Set_Should_Be_Refused()
{
    let mut workspace = Fresh();

    assert_eq!(
        workspace.Apply(&WorkspaceChangeSet::From(ChangeSource::Correction)),
        Err(WorkspaceError::Vacuous)
    );
}

/// The failure portability exists to prevent, caught at the door rather than in the
/// bytes.
/// Every spelling of "this is not workspace-relative" the door has to catch.
fn Absolute_Paths() -> [&'static str; ABSOLUTE_PATH_SPELLINGS]
{
    return ["F:/repos/xvpe/a.rs", "/usr/src/a.rs", "C:\\src\\a.rs", "\\\\?\\F:\\a.rs"];
}

#[test]
fn Test_An_Absolute_Path_Should_Be_Refused()
{
    let mut workspace = Fresh();

    for path in Absolute_Paths()
    {
        assert!(
            workspace.Apply(&Edit(MemberPath(path), "content")).is_err(),
            "`{path}` is not workspace-relative"
        );
    }
}

/// Every spelling of "this reaches outside the workspace" the door has to catch.
fn Escaping_Paths() -> [&'static str; ESCAPING_PATH_SPELLINGS]
{
    return ["../outside.rs", "src/../../outside.rs", ".."];
}

#[test]
fn Test_A_Path_Reaching_Outside_The_Workspace_Should_Be_Refused()
{
    let mut workspace = Fresh();

    for path in Escaping_Paths()
    {
        assert!(workspace.Apply(&Edit(MemberPath(path), "content")).is_err(), "`{path}`");
    }
}

/// Two spellings of one path are one member, so a set naming both is a set with no
/// correct reading.
#[test]
fn Test_One_Path_Changed_Twice_Should_Be_Refused()
{
    let mut workspace = Fresh();

    assert_eq!(
        workspace.Apply(
            &WorkspaceChangeSet::From(ChangeSource::CodeGenerator)
                .Present("src/a.rs", "one")
                .Present("./src/A.rs", "other")
        ),
        Err(WorkspaceError::Conflicting {
            path: "src/a.rs".to_owned()
        })
    );
}

/// A refused set leaves the workspace exactly as it was. A half-applied checkout is
/// not a state anybody should be able to ask questions about.
#[test]
fn Test_A_Refused_Set_Should_Change_Nothing()
{
    let mut workspace = Fresh();
    let first = Edit(MemberPath("src/a.rs"),"original");
    workspace.Apply(&first).expect("every path in this fixture is workspace-relative and distinct, which the door admits");
    let before = (workspace.Generation(), workspace.Id());

    let checkout = WorkspaceChangeSet::From(ChangeSource::GitCheckout)
        .Present("src/b.rs", "new")
        .Present("src/c.rs", "new")
        .Present("/absolute/d.rs", "new");

    let refused = workspace.Apply(&checkout);

    assert!(refused.is_err());
    assert_eq!((workspace.Generation(), workspace.Id()), before);
    assert_eq!(
        workspace.Content_Of("src/b.rs"),
        None,
        "the valid changes in a refused set must not have landed"
    );
}

/// Provenance is a fact about the change, not about the workspace. Two workspaces
/// holding the same files are the same workspace however the files got there.
#[test]
fn Test_The_Source_Should_Not_Reach_The_Workspace_Identity()
{
    let mut identities = Vec::new();

    for source in ChangeSource::All()
    {
        let mut workspace = Fresh();
        let change = WorkspaceChangeSet::From(*source).Present("src/a.rs", "pub fn a() {}");
        workspace.Apply(&change).expect("every path in this fixture is workspace-relative and distinct, which the door admits");
        identities.push(workspace.Id());
    }

    assert!(
        identities.windows(NEIGHBOURING_PAIR).all(|pair| return pair.first() == pair.last()),
        "five sources, one workspace: {identities:?}"
    );
}
