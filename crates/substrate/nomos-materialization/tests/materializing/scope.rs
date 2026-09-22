//! Publication scope is carried and reported, and gates no write.
//!
//! `OD-PACKAGE-005` makes the scope a property of where an asset may travel — whether it may
//! enter repository-distributed state — and states the guard it owes as a guard on *that*
//! transition. A materializer that refused to place a `Local` target would be enforcing a
//! publication policy at the wrong boundary, and would collapse the two axes that record's
//! own controls table keeps apart: "collapse scope into ownership as combined values"
//! produces names that "describe combinations rather than properties".
//!
//! So the tests here are the pair. One shows all three scopes written; the other shows all
//! three reported.

use crate::fake::FakeTree;
use crate::fixtures::{Intent, Roots, Scoped, Source, Target};
use nomos_materialization::{Materialize, OwnershipClass, PublicationScope};

const EVERY_SCOPE: [PublicationScope; 3] = [PublicationScope::Ephemeral, PublicationScope::Local, PublicationScope::Shared];

/// One `GeneratedOwned` intent per scope, all three otherwise identical.
fn Scoped_Intents() -> Vec<nomos_materialization::MaterializationIntent>
{
    return EVERY_SCOPE
        .iter()
        .map(|scope| {
            let name = format!("{scope}.md");

            return Scoped(Intent(&name, &name, OwnershipClass::GeneratedOwned), *scope);
        })
        .collect();
}

fn Tree_For_Every_Scope() -> FakeTree
{
    let mut tree = FakeTree::New();
    for scope in EVERY_SCOPE
    {
        tree = tree.With(&Source(&format!("{scope}.md")), "bytes");
    }

    return tree;
}

/// The scope decides nothing about whether the bytes land. All three do.
#[test]
fn Test_Every_Publication_Scope_Should_Be_Written()
{
    let tree = Tree_For_Every_Scope();

    Materialize(&Scoped_Intents(), &Roots(), &tree).expect("a scope refuses no write");

    for scope in EVERY_SCOPE
    {
        assert_eq!(
            tree.Contents(&Target(&format!("{scope}.md"))).as_deref(),
            Some("bytes"),
            "{scope} was not written, so the scope gated a write it has no business gating"
        );
    }
}

/// And it is not dropped on the floor either: whoever asks `OD-PACKAGE-005`'s question later
/// reads the answer off the report rather than re-deriving it from a manifest they may not
/// hold.
#[test]
fn Test_The_Report_Should_Carry_Each_Targets_Scope()
{
    let tree = Tree_For_Every_Scope();

    let report = Materialize(&Scoped_Intents(), &Roots(), &tree).expect("three writable targets");

    for scope in EVERY_SCOPE
    {
        let placements = report.Placements_Scoped(scope);
        let targets: Vec<&str> = placements.iter().map(|placement| return placement.target.as_str()).collect();

        assert_eq!(targets, vec![format!("{scope}.md").as_str()], "{scope} was not reported");
    }
}

/// The surface label the intent carried is passed through unread, which is what makes a
/// multi-target report legible without this crate knowing what any of the labels mean.
#[test]
fn Test_The_Report_Should_Pass_The_Surface_Label_Through()
{
    let tree = FakeTree::New().With(&Source("one.md"), "bytes");

    let report = Materialize(&[Intent("one.md", "one.md", OwnershipClass::GeneratedOwned)], &Roots(), &tree)
        .expect("one writable target");

    assert_eq!(
        report.placements.first().map(|placement| return placement.surface.as_str()),
        Some("the one.md surface")
    );
}
