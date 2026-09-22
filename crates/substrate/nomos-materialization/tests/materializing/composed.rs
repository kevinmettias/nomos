//! `Composed`: only the owned region is written, and the free region is never risked.
//!
//! `OD-PACKAGE-004`: "Regeneration is scoped to the region tied to a declared source of
//! truth, and never touches the region that is not. Where a renderer exists, it writes only
//! the owned region and leaves the rest byte-for-byte untouched." Its controls table names
//! the failure directly — "treat `Composed`'s free region as `GeneratedOwned`" produces
//! "prose a person is expected to edit gets silently overwritten the next time anything
//! regenerates the file" — so every test here asserts what survived, not only what changed.
//!
//! The fixtures are this test's own, deliberately. A `Composed` target's owned region is
//! whatever its placer declared, and constructing the case where the free region *would* be
//! lost means editing the file until the markers are wrong — which is not something to do to
//! a document this repository actually ships.

use crate::fake::FakeTree;
use crate::fixtures::{In_Region, Intent, Roots, Source, Target};
use nomos_materialization::{Materialize, MaterializationError, OwnedRegion, OwnershipClass, PlacementOutcome, Refusal};

/// The prose above the owned region.
const PREAMBLE: &str = "# A document a person wrote\n\nSome prose that is nobody's to regenerate.\n\n";
const OPENING: &str = "<!-- generated:begin -->";
const CLOSING: &str = "<!-- generated:end -->";
/// The prose below it.
const EPILOGUE: &str = "\n\nMore prose, also nobody's to regenerate.\n";

fn Region() -> OwnedRegion
{
    return OwnedRegion::New(OPENING, CLOSING);
}

/// A target carrying free prose either side of one owned region.
fn Composed_Target(owned: &str) -> String
{
    return format!("{PREAMBLE}{OPENING}{owned}{CLOSING}{EPILOGUE}");
}

fn Composed_Intent() -> nomos_materialization::MaterializationIntent
{
    return In_Region(Intent("table.md", "composed.md", OwnershipClass::Composed), Region());
}

#[test]
fn Test_Only_The_Owned_Region_Should_Be_Written()
{
    let tree = FakeTree::New()
        .With(&Source("table.md"), "\n| a | b |\n")
        .With(&Target("composed.md"), &Composed_Target("\n| stale | table |\n"));

    let report = Materialize(&[Composed_Intent()], &Roots(), &tree).expect("both markers, in order, once each");

    assert_eq!(tree.Contents(&Target("composed.md")), Some(Composed_Target("\n| a | b |\n")));
    assert_eq!(
        report.placements.first().map(|placement| return placement.outcome),
        Some(PlacementOutcome::OwnedRegionReplaced)
    );
}

/// The assertion the record's own controls table is about: the free region is still there,
/// byte for byte, and so are the markers that bound it.
#[test]
fn Test_The_Free_Region_Should_Survive_A_Write_Byte_For_Byte()
{
    let tree = FakeTree::New()
        .With(&Source("table.md"), "replacement")
        .With(&Target("composed.md"), &Composed_Target("original"));

    Materialize(&[Composed_Intent()], &Roots(), &tree).expect("a well-formed region");

    let written = tree.Contents(&Target("composed.md")).expect("the target was written");
    assert!(written.starts_with(PREAMBLE), "the prose above the region was rewritten: {written:?}");
    assert!(written.ends_with(EPILOGUE), "the prose below the region was rewritten: {written:?}");
    assert!(!written.contains("original"), "the owned region was not replaced: {written:?}");
}

/// Every way the region cannot be located, and the one assertion that matters for all of
/// them: the target is exactly what it was.
#[test]
fn Test_A_Region_That_Cannot_Be_Located_Should_Refuse_And_Leave_The_Target_Alone()
{
    let cases = [
        (format!("{PREAMBLE}{CLOSING}{EPILOGUE}"), Refusal::AbsentRegionMarker { marker: OPENING.to_owned() }),
        (format!("{PREAMBLE}{OPENING}{EPILOGUE}"), Refusal::AbsentRegionMarker { marker: CLOSING.to_owned() }),
        (
            format!("{PREAMBLE}{OPENING}a{CLOSING}between{OPENING}b{CLOSING}{EPILOGUE}"),
            Refusal::RepeatedRegionMarker { marker: OPENING.to_owned() },
        ),
        (format!("{PREAMBLE}{CLOSING}between{OPENING}{EPILOGUE}"), Refusal::InvertedRegionMarkers),
    ];

    for (target, expected) in cases
    {
        let tree = FakeTree::New().With(&Source("table.md"), "replacement").With(&Target("composed.md"), &target);

        let error = Materialize(&[Composed_Intent()], &Roots(), &tree).expect_err("the region cannot be located");

        assert_eq!(error, MaterializationError::Refused { target: "composed.md".to_owned(), refusal: expected });
        assert_eq!(tree.Contents(&Target("composed.md")).as_deref(), Some(target.as_str()));
        assert_eq!(tree.Attempted_Writes(), 0);
    }
}

/// There is no free region to preserve and no markers to write between, so creating the file
/// would make every byte of it the mechanism's — which is the one thing `Composed` is not.
#[test]
fn Test_A_Composed_Target_That_Is_Not_There_Should_Refuse()
{
    let tree = FakeTree::New().With(&Source("table.md"), "replacement");

    let error = Materialize(&[Composed_Intent()], &Roots(), &tree).expect_err("there is no target to compose into");

    assert_eq!(
        error,
        MaterializationError::Refused { target: "composed.md".to_owned(), refusal: Refusal::AbsentComposedTarget }
    );
    assert_eq!(tree.Contents(&Target("composed.md")), None);
}

#[test]
fn Test_A_Composed_Target_Declaring_No_Region_Should_Refuse()
{
    let tree = FakeTree::New()
        .With(&Source("table.md"), "replacement")
        .With(&Target("composed.md"), &Composed_Target("original"));

    let error = Materialize(&[Intent("table.md", "composed.md", OwnershipClass::Composed)], &Roots(), &tree)
        .expect_err("nothing says which bytes the source owns");

    assert_eq!(
        error,
        MaterializationError::Refused { target: "composed.md".to_owned(), refusal: Refusal::UndeclaredOwnedRegion }
    );
    assert_eq!(tree.Contents(&Target("composed.md")), Some(Composed_Target("original")));
}

/// The other direction, and the more dangerous one: a region declared beside a class that
/// overwrites everything. Reading the class and ignoring the region would replace a whole
/// file somebody meant to protect half of.
#[test]
fn Test_A_Region_Declared_On_A_Generated_Owned_Target_Should_Refuse()
{
    let tree = FakeTree::New()
        .With(&Source("table.md"), "replacement")
        .With(&Target("composed.md"), &Composed_Target("original"));
    let intent = In_Region(Intent("table.md", "composed.md", OwnershipClass::GeneratedOwned), Region());

    let error = Materialize(&[intent], &Roots(), &tree).expect_err("two declarations that disagree");

    assert_eq!(
        error,
        MaterializationError::Refused { target: "composed.md".to_owned(), refusal: Refusal::OwnedRegionOnAnUncomposedTarget }
    );
    assert_eq!(tree.Contents(&Target("composed.md")), Some(Composed_Target("original")));
}
