//! `UserOwned`: never written, under any circumstance.
//!
//! `OD-PACKAGE-004`: "Never happens, under any circumstance. No package, no installer, no
//! renderer, no profile writes this path." The tests here are written against the strongest
//! reading of that sentence, because the weaker ones are all satisfiable by a mechanism that
//! is still wrong: skipping the target and writing the rest, or writing the earlier targets
//! and stopping when it reaches this one.

use crate::fake::FakeTree;
use crate::fixtures::{Intent, Roots, Source, Target};
use nomos_materialization::{Materialize, MaterializationError, OwnershipClass, Refusal};

#[test]
fn Test_A_User_Owned_Target_Should_Not_Be_Written()
{
    let tree = FakeTree::New()
        .With(&Source("settings.json"), "what a package would have placed")
        .With(&Target("settings.json"), "what the person wrote");

    let error = Materialize(&[Intent("settings.json", "settings.json", OwnershipClass::UserOwned)], &Roots(), &tree)
        .expect_err("a UserOwned target is never written");

    assert_eq!(error, MaterializationError::Refused { target: "settings.json".to_owned(), refusal: Refusal::UserOwnedTarget });
    assert_eq!(tree.Contents(&Target("settings.json")).as_deref(), Some("what the person wrote"));
}

/// The target is absent, so there is nothing to destroy and every weaker reading of the rule
/// would let the write through. It is still refused: the class says nothing about whether a
/// file is there.
#[test]
fn Test_A_User_Owned_Target_That_Is_Not_There_Should_Still_Not_Be_Written()
{
    let tree = FakeTree::New().With(&Source("settings.json"), "what a package would have placed");

    let error = Materialize(&[Intent("settings.json", "settings.json", OwnershipClass::UserOwned)], &Roots(), &tree)
        .expect_err("creating a UserOwned target is still writing it");

    assert!(matches!(error, MaterializationError::Refused { refusal: Refusal::UserOwnedTarget, .. }), "{error}");
    assert_eq!(tree.Contents(&Target("settings.json")), None);
    assert_eq!(tree.Attempted_Writes(), 0);
}

/// The one that separates "refuses the write" from "refuses the run". Two perfectly writable
/// `GeneratedOwned` targets come first, and neither is written, because a run that placed
/// them and then refused would leave a tree no declaration describes.
#[test]
fn Test_A_User_Owned_Target_Should_Stop_The_Whole_Run_Before_Its_Siblings_Are_Written()
{
    let tree = FakeTree::New()
        .With(&Source("one.md"), "first")
        .With(&Source("two.md"), "second")
        .With(&Source("settings.json"), "what a package would have placed");
    let intents = [
        Intent("one.md", "one.md", OwnershipClass::GeneratedOwned),
        Intent("two.md", "two.md", OwnershipClass::GeneratedOwned),
        Intent("settings.json", "settings.json", OwnershipClass::UserOwned),
    ];
    let before = tree.Snapshot();

    let error = Materialize(&intents, &Roots(), &tree).expect_err("the third intent is UserOwned");

    assert_eq!(error, MaterializationError::Refused { target: "settings.json".to_owned(), refusal: Refusal::UserOwnedTarget });
    assert_eq!(tree.Attempted_Writes(), 0, "a refusal is reached before the first write, not after two of them");
    assert_eq!(tree.Snapshot(), before);
}

/// `OwnershipClass::UNDECLARED` is `UserOwned`, which is what makes an asset nobody
/// classified safe by construction: the refusal is the same refusal.
#[test]
fn Test_An_Undeclared_Class_Should_Be_Refused_As_User_Owned()
{
    let tree = FakeTree::New().With(&Source("unknown.md"), "bytes");

    let error = Materialize(&[Intent("unknown.md", "unknown.md", OwnershipClass::UNDECLARED)], &Roots(), &tree)
        .expect_err("an unrecognized asset defaults to UserOwned and is refused");

    assert!(matches!(error, MaterializationError::Refused { refusal: Refusal::UserOwnedTarget, .. }), "{error}");
}
