//! `GeneratedOwned`: overwritten unconditionally.
//!
//! `OD-PACKAGE-004`: "Overwrites the file unconditionally. The renderer is the sole author
//! of every byte in it, so there is no partial write and nothing to merge." Unconditionally
//! means the target's own contents are never consulted for permission — a hand edit is not a
//! competing claim, and the record says so directly.

use crate::fake::FakeTree;
use crate::fixtures::{Intent, Roots, Source, Target};
use nomos_materialization::{Materialize, MaterializationError, OwnershipClass, PlacementOutcome, Refusal};

#[test]
fn Test_A_Generated_Owned_Target_Should_Be_Overwritten_Whatever_It_Held()
{
    let tree = FakeTree::New()
        .With(&Source("rendered.md"), "the renderer's bytes")
        .With(&Target("rendered.md"), "somebody typed this in by hand");

    let report = Materialize(&[Intent("rendered.md", "rendered.md", OwnershipClass::GeneratedOwned)], &Roots(), &tree)
        .expect("a GeneratedOwned target is overwritten unconditionally");

    assert_eq!(tree.Contents(&Target("rendered.md")).as_deref(), Some("the renderer's bytes"));
    assert_eq!(
        report.placements.first().map(|placement| return placement.outcome),
        Some(PlacementOutcome::Overwritten)
    );
}

#[test]
fn Test_A_Generated_Owned_Target_That_Is_Not_There_Should_Be_Created()
{
    let tree = FakeTree::New().With(&Source("rendered.md"), "the renderer's bytes");

    let report = Materialize(&[Intent("rendered.md", "rendered.md", OwnershipClass::GeneratedOwned)], &Roots(), &tree)
        .expect("an absent GeneratedOwned target is the renderer's to create");

    assert_eq!(tree.Contents(&Target("rendered.md")).as_deref(), Some("the renderer's bytes"));
    assert_eq!(
        report.placements.first().map(|placement| return placement.outcome),
        Some(PlacementOutcome::Created)
    );
}

#[test]
fn Test_Every_Intent_Should_Be_Performed_In_Declaration_Order()
{
    let tree = FakeTree::New()
        .With(&Source("one.md"), "first")
        .With(&Source("two.md"), "second")
        .With(&Source("three.md"), "third");
    let intents = [
        Intent("one.md", "one.md", OwnershipClass::GeneratedOwned),
        Intent("two.md", "two.md", OwnershipClass::GeneratedOwned),
        Intent("three.md", "three.md", OwnershipClass::GeneratedOwned),
    ];

    let report = Materialize(&intents, &Roots(), &tree).expect("three writable targets");

    let targets: Vec<&str> = report.placements.iter().map(|placement| return placement.target.as_str()).collect();
    assert_eq!(targets, vec!["one.md", "two.md", "three.md"]);
    assert_eq!(tree.Contents(&Target("three.md")).as_deref(), Some("third"));
}

/// A source that does not read is a refusal rather than an empty file, because writing an
/// empty file is a write and this mechanism has nothing to write.
#[test]
fn Test_A_Source_That_Does_Not_Read_Should_Refuse_Before_Anything_Is_Written()
{
    let tree = FakeTree::New().With(&Target("rendered.md"), "what was there");

    let error = Materialize(&[Intent("absent.md", "rendered.md", OwnershipClass::GeneratedOwned)], &Roots(), &tree)
        .expect_err("the source is not in the tree");

    assert!(matches!(error, MaterializationError::Refused { refusal: Refusal::UnreadableSource { .. }, .. }), "{error}");
    assert_eq!(tree.Attempted_Writes(), 0);
    assert_eq!(tree.Contents(&Target("rendered.md")).as_deref(), Some("what was there"));
}

#[test]
fn Test_A_Target_Outside_The_Tree_Should_Refuse()
{
    let tree = FakeTree::New().With(&Source("rendered.md"), "bytes");

    for (target, expected) in [("/etc/hosts", Refusal::AbsoluteTarget), ("../sibling/file.md", Refusal::EscapingTarget)]
    {
        let error = Materialize(&[Intent("rendered.md", target, OwnershipClass::GeneratedOwned)], &Roots(), &tree)
            .expect_err("a target outside the tree is nowhere this mechanism may write");

        assert_eq!(error, MaterializationError::Refused { target: target.to_owned(), refusal: expected });
    }
    assert_eq!(tree.Attempted_Writes(), 0);
}

#[test]
fn Test_A_Source_Outside_The_Tree_Should_Refuse()
{
    let tree = FakeTree::New();

    for (source, expected) in [("C:\\secrets", Refusal::AbsoluteSource), ("../sibling/file.md", Refusal::EscapingSource)]
    {
        let error = Materialize(&[Intent(source, "rendered.md", OwnershipClass::GeneratedOwned)], &Roots(), &tree)
            .expect_err("a source outside the tree is nowhere this mechanism may read");

        assert_eq!(error, MaterializationError::Refused { target: "rendered.md".to_owned(), refusal: expected });
    }
    assert_eq!(tree.Attempted_Writes(), 0);
}
