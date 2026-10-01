//! A failed run leaves the tree as it found it.
//!
//! The claim is not about one file — `Replace_Atomically` already promises that, and a test
//! of it would be a test of `nomos-platform`. It is about a multi-target intent whose third
//! write fails after the first two succeeded, which is the state a per-file primitive cannot
//! reach on its own and the one a caller has no way to repair by rerunning.
//!
//! So every test here forces a real failure partway through a real run, by handing the
//! materializer a tree told to refuse one particular write, and asserts the *whole* tree
//! afterwards rather than the targets it expects to have been restored. A snapshot
//! comparison catches a rollback that put back four of five files; an assertion naming the
//! files would only catch the ones it happened to name.

use crate::fake::FakeTree;
use crate::fixtures::{Intent, Roots, Source, Target};
use nomos_materialization::{Materialize, MaterializationError, OwnershipClass};

/// Three placements, all writable, so nothing but the injected failure can refuse.
fn Three_Intents() -> [nomos_materialization::MaterializationIntent; 3]
{
    return [
        Intent("one.md", "one.md", OwnershipClass::GeneratedOwned),
        Intent("two.md", "two.md", OwnershipClass::GeneratedOwned),
        Intent("three.md", "three.md", OwnershipClass::GeneratedOwned),
    ];
}

/// A tree holding all three sources and the first two targets.
fn Tree_With_Sources() -> FakeTree
{
    return FakeTree::New()
        .With(&Source("one.md"), "new one")
        .With(&Source("two.md"), "new two")
        .With(&Source("three.md"), "new three")
        .With(&Target("one.md"), "old one")
        .With(&Target("two.md"), "old two")
        .With(&Target("three.md"), "old three");
}

/// The midway failure: the second of three writes is refused after the first has already
/// landed, and the whole tree is compared against the snapshot taken before the run.
#[test]
fn Test_A_Write_That_Fails_Midway_Should_Leave_Every_Target_Unchanged()
{
    let tree = Tree_With_Sources().Failing(&[2]);
    let before = tree.Snapshot();

    let error = Materialize(&Three_Intents(), &Roots(), &tree).expect_err("the second write was refused");

    assert!(
        matches!(&error, MaterializationError::RolledBack { target, .. } if target == "two.md"),
        "{error}"
    );
    assert_eq!(tree.Snapshot(), before, "a failed run must leave the tree exactly as it found it");
    assert!(tree.Attempted_Writes() >= 3, "the first write landed and was then put back, so three writes were attempted");
}

/// The same failure one step later, so two writes have to be undone rather than one — and in
/// reverse order, which a single-undo test cannot distinguish from no ordering at all.
#[test]
fn Test_A_Failure_On_The_Last_Write_Should_Undo_Both_Earlier_Ones()
{
    let tree = Tree_With_Sources().Failing(&[3]);
    let before = tree.Snapshot();

    let error = Materialize(&Three_Intents(), &Roots(), &tree).expect_err("the third write was refused");

    assert!(
        matches!(&error, MaterializationError::RolledBack { target, .. } if target == "three.md"),
        "{error}"
    );
    assert_eq!(tree.Snapshot(), before);
}

/// Undoing a *creation* is a removal, not a restoration, and the two are different
/// operations on the port. A target that was not there before the run must not be there
/// after a failed one either.
#[test]
fn Test_A_Target_Created_Before_The_Failure_Should_Be_Removed_Again()
{
    let tree = FakeTree::New()
        .With(&Source("one.md"), "new one")
        .With(&Source("two.md"), "new two")
        .With(&Target("two.md"), "old two")
        .Failing(&[2]);
    let before = tree.Snapshot();
    let intents = [
        Intent("one.md", "one.md", OwnershipClass::GeneratedOwned),
        Intent("two.md", "two.md", OwnershipClass::GeneratedOwned),
    ];

    let error = Materialize(&intents, &Roots(), &tree).expect_err("the second write was refused");

    assert!(matches!(&error, MaterializationError::RolledBack { .. }), "{error}");
    assert_eq!(tree.Contents(&Target("one.md")), None, "a target created by a failed run must not survive it");
    assert_eq!(tree.Snapshot(), before);
}

/// The outcome the run cannot repair, reported as itself. Folding it into `RolledBack` would
/// claim a restoration that did not happen, and the caller would stop looking.
#[test]
fn Test_A_Rollback_That_Fails_Should_Say_So_Rather_Than_Report_A_Clean_Undo()
{
    let tree = Tree_With_Sources().Failing(&[2, 3]);

    let error = Materialize(&Three_Intents(), &Roots(), &tree).expect_err("the second write and the undo both failed");

    let MaterializationError::RollbackFailed { target, unrestored_target, .. } = &error
    else
    {
        panic!("a failed undo must not report as a clean rollback: {error}");
    };
    assert_eq!(target, "two.md");
    assert_eq!(unrestored_target, "one.md");
    assert_ne!(
        tree.Contents(&Target("one.md")).as_deref(),
        Some("old one"),
        "the point of this outcome is that the earlier write really is still there"
    );
}
