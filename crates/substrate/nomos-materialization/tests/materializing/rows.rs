//! Adding a row to a table that sits inside an owned region.
//!
//! `OD-PACKAGE-004` version 3 names this as the corner where the two regions meet: "a
//! mechanism adding a row writes the two owned cells and leaves the `Owns` cell empty for a
//! person, because it has nothing to write there and inventing a description is the failure
//! this record exists to prevent."
//!
//! What this mechanism owes that sentence is narrow and worth stating exactly, because the
//! record's own instance is a granularity a marker pair cannot name. Which cells a row's
//! owned region covers is decided by whoever renders the source — that is the "units the
//! comparison reads" question, and it is answered at the declaration, not here. What is
//! answered here is everything after that decision: the rendered rows land inside the
//! declared span and nowhere else, an empty cell is written as an empty cell rather than
//! filled or dropped, and every byte outside the span survives untouched.
//!
//! The fixture is table-shaped for exactly that reason. A test over an anonymous blob would
//! pass just as happily while normalizing whitespace, trimming a trailing empty cell or
//! reflowing a long line — the three ways a mechanism silently rewrites a person's text
//! while believing it only wrote its own.

use crate::fake::FakeTree;
use crate::fixtures::{In_Region, Intent, Roots, Source, Target};
use nomos_materialization::{Materialize, OwnedRegion, OwnershipClass};

/// Prose above the table: free region, and the mechanism has no business in it.
const PREAMBLE: &str = "# The layout\n\nFour rows below are marked `[repo tooling]`.\n\n";
const OPENING: &str = "<!-- rows:begin -->";
const CLOSING: &str = "<!-- rows:end -->";
/// Prose below it, free for the same reason.
const EPILOGUE: &str = "\nA paragraph somebody wrote, which nothing generates.\n";

/// The table as it stands: a header, a separator and two rows.
const EXISTING_ROWS: &str = concat!(
    "\n| Zone | Crate | Owns |\n",
    "|---|---|---|\n",
    "| Substrate | `one` | What one owns, written by a person. |\n",
    "| Provider | `two` | What two owns, also written by a person. |\n"
);

/// The same table with a third row appended, whose description cell a mechanism has nothing
/// to put in and therefore leaves empty.
const ROWS_WITH_ONE_ADDED: &str = concat!(
    "\n| Zone | Crate | Owns |\n",
    "|---|---|---|\n",
    "| Substrate | `one` | What one owns, written by a person. |\n",
    "| Provider | `two` | What two owns, also written by a person. |\n",
    "| Substrate | `three` |  |\n"
);

fn Target_Text(rows: &str) -> String
{
    return format!("{PREAMBLE}{OPENING}{rows}{CLOSING}{EPILOGUE}");
}

fn Row_Intent() -> nomos_materialization::MaterializationIntent
{
    return In_Region(
        Intent("rows.md", "layout.md", OwnershipClass::Composed),
        OwnedRegion::New(OPENING, CLOSING),
    );
}

/// The whole assertion in one comparison: the file afterwards is the file before with the
/// span replaced and nothing else different.
#[test]
fn Test_Adding_A_Row_Should_Change_Only_The_Span_The_Markers_Name()
{
    let tree = FakeTree::New()
        .With(&Source("rows.md"), ROWS_WITH_ONE_ADDED)
        .With(&Target("layout.md"), &Target_Text(EXISTING_ROWS));

    Materialize(&[Row_Intent()], &Roots(), &tree).expect("a well-formed region");

    assert_eq!(tree.Contents(&Target("layout.md")), Some(Target_Text(ROWS_WITH_ONE_ADDED)));
}

/// An empty cell is a marker for a person, per the record, so it has to still be there and
/// still be empty. A mechanism that trimmed the trailing `|  |` or filled it in would pass a
/// test that only counted rows.
#[test]
fn Test_A_New_Rows_Empty_Cell_Should_Be_Written_As_An_Empty_Cell()
{
    let tree = FakeTree::New()
        .With(&Source("rows.md"), ROWS_WITH_ONE_ADDED)
        .With(&Target("layout.md"), &Target_Text(EXISTING_ROWS));

    Materialize(&[Row_Intent()], &Roots(), &tree).expect("a well-formed region");

    let written = tree.Contents(&Target("layout.md")).expect("the target was written");
    assert!(written.contains("| Substrate | `three` |  |\n"), "the empty cell was not written as spelled: {written:?}");
}

/// The free region either side, asserted separately from the comparison above so that a
/// failure says which half moved.
#[test]
fn Test_The_Prose_Around_The_Table_Should_Survive_Byte_For_Byte()
{
    let tree = FakeTree::New()
        .With(&Source("rows.md"), ROWS_WITH_ONE_ADDED)
        .With(&Target("layout.md"), &Target_Text(EXISTING_ROWS));

    Materialize(&[Row_Intent()], &Roots(), &tree).expect("a well-formed region");

    let written = tree.Contents(&Target("layout.md")).expect("the target was written");
    assert!(written.starts_with(PREAMBLE), "the prose above the table was rewritten: {written:?}");
    assert!(written.ends_with(EPILOGUE), "the prose below the table was rewritten: {written:?}");
    assert!(
        written.contains("Four rows below are marked `[repo tooling]`."),
        "a mark inside the free region did not survive: {written:?}"
    );
}

/// The negative control for all three above. With the markers gone the span cannot be
/// located, and the mechanism refuses rather than replacing the file with a table — which is
/// what "never write the free region" has to mean when the region cannot be found.
#[test]
fn Test_A_Table_With_No_Markers_Should_Refuse_Rather_Than_Replace_The_File()
{
    let unmarked = format!("{PREAMBLE}{EXISTING_ROWS}{EPILOGUE}");
    let tree = FakeTree::New()
        .With(&Source("rows.md"), ROWS_WITH_ONE_ADDED)
        .With(&Target("layout.md"), &unmarked);

    Materialize(&[Row_Intent()], &Roots(), &tree).expect_err("there is no span to write into");

    assert_eq!(tree.Contents(&Target("layout.md")).as_deref(), Some(unmarked.as_str()));
    assert_eq!(tree.Attempted_Writes(), 0);
}
