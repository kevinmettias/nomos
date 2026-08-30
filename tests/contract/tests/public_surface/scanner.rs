//! The negative controls for the reader itself.
//!
//! Everything in [`crate::snapshots`] is satisfied by a scanner that returns nothing for
//! every crate: the snapshots would be empty, they would match, and the suite would be green
//! having checked that nothing equals nothing. These assert the scanner finds real exports,
//! stops where the language stops, and reports the one kind of name it cannot follow.

use crate::reading::{Says, Surface_Named};
use nomos_contract_tests::Surface;

/// Every restricted-visibility spelling the scanner must draw the line at.
const RESTRICTED_VISIBILITY_MARKERS: [&str; 3] = ["pub(crate)", "pub(super)", "pub(in "];

/// The negative control.
///
/// This asserts the scanner finds real exports, and that it draws the line where the
/// language draws it.
#[test]
fn Test_The_Scanner_Should_Find_Real_Exports_And_Stop_At_Restricted_Ones()
{
    let surface = Surface_Named("nomos-spec-project");

    // Declared in a private module and re-exported by the crate root. If the resolver
    // stopped at `pub mod`, this crate's entire API would be invisible.
    assert!(Says(&surface, "Build("), "the re-exported Build function is not in the surface");
    assert!(Says(&surface, "pub struct"), "no struct reached the surface");
    assert!(Says(&surface, "pub enum"), "no enum reached the surface");
    // `Without_Test_Modules` blanks unit-test bodies before the scan. A helper written
    // inside one is not an export, and counting it would make every crate's surface grow
    // with its test suite.
    assert!(
        !Says(&surface, "::tests::"),
        "a unit test module reached the public surface: {:?}",
        surface.declarations
    );
    for restricted in RESTRICTED_VISIBILITY_MARKERS
    {
        assert!(
            !Says(&surface, restricted),
            "{restricted} reached the surface, which is not public"
        );
    }
}

/// Restricted visibility elsewhere in the workspace is genuinely excluded.
///
/// `pub(crate)` helpers declared in this very crate. Neither is an export, and a scanner
/// that treated any `pub` prefix as public would list all of them — so this is the case
/// that would catch it.
const CRATE_VISIBLE_HELPERS: [&str; 3] = ["Source_Files", "Without_Test_Modules", "Matching_Brace"];

/// `gates.rs` in this very crate declares `pub(crate) fn Source_Files` and
/// `pub(crate) fn Without_Test_Modules`. Neither is an export, and a scanner that treated
/// any `pub` prefix as public would list both — so this is the case that would catch it.
#[test]
fn Test_A_Crate_Visible_Helper_Should_Not_Be_An_Export()
{
    let surface = Surface_Named("nomos-contract-tests");

    for hidden in CRATE_VISIBLE_HELPERS
    {
        assert!(
            !Says(&surface, hidden),
            "{hidden} is pub(crate) and appears in the exported surface"
        );
    }

    assert!(
        Says(&surface, "Corpus_Gates"),
        "the crate's real exports are missing, so the assertions above proved nothing"
    );
}

/// A re-exported name this reader cannot follow is reported, and its siblings still resolve.
///
/// The case `OD-GATE-002` version 2 records, and the reason it is asserted against a real
/// crate rather than a fixture: the defect was never that the resolver followed a name
/// wrongly. It followed them correctly and then asked the wrong question about the answer.
/// `Emit_Re_Export` set one `resolved_any` flag for a whole `pub use` declaration, so a name
/// that resolved to nothing was reported only when *every* name beside it also did.
///
/// `tests/integration/src/lib.rs` writes `pub use corpus::{Corpus, SourceFile,
/// Subject_Of_Path, Walk}`. Three of those are declared in that crate. `Subject_Of_Path` is
/// re-exported by `corpus.rs` from `nomos-model`, which this per-crate reader cannot see
/// into — so the three that landed made the fourth look handled, and it left the snapshot
/// while the crate went on exporting it and `slice.rs` went on calling it. `a6f0e9c` is the
/// commit that blessed it away.
///
/// The siblings declared in `nomos-integration-tests`'s own `pub use` list, alongside the
/// one name (`Subject_Of_Path`) the reader cannot follow.
const RESOLVABLE_REEXPORT_SIBLINGS: [&str; 3] = ["Corpus", "SourceFile", "Walk"];

/// Both halves are in one list on purpose. The names that must resolve and the name that
/// must not are siblings, so a reader that had simply started reporting everything as
/// unresolved would satisfy the first assertion and fail the second in the same breath.
#[test]
fn Test_An_Unfollowable_Re_Export_Should_Be_Reported_While_Its_Siblings_Resolve()
{
    let surface = Surface_Named("nomos-integration-tests");

    Assert_The_Unfollowable_Name_Is_Reported(&surface);

    // The negative control, and the whole reason the grain is per name rather than per
    // declaration: its three siblings are declared in this crate and must still be found.
    for resolved in RESOLVABLE_REEXPORT_SIBLINGS
    {
        Assert_The_Sibling_Resolves(&surface, resolved);
    }
}

/// Both halves of the surface, for the one name this per-crate reader cannot follow.
fn Assert_The_Unfollowable_Name_Is_Reported(surface: &Surface)
{
    assert!(
        surface
            .unresolved
            .iter()
            .any(|line| return line.contains("Subject_Of_Path")),
        "a name re-exported from another crate is missing from both halves of the surface. \
         It is not in `unresolved` here, and the assertion below says it is not a \
         declaration either, so the snapshot claims the crate does not export it: {:?}",
        surface.unresolved
    );
    assert!(
        !Says(surface, "Subject_Of_Path"),
        "the name resolved to a declaration in this crate, which would make the assertion \
         above a claim about the wrong mechanism — `Subject_Of_Path` is declared in \
         nomos-model and this reader is recorded as reading one crate"
    );
}

/// A sibling in the same `pub use` list is declared here, so it must still resolve.
fn Assert_The_Sibling_Resolves(surface: &Surface, resolved: &str)
{
    let owned = format!("nomos_integration_tests::{resolved}");

    assert!(
        Says(surface, &owned),
        "{resolved} shares a `pub use` list with the unfollowable name and is declared \
         in this crate, so it must still resolve. A reader reporting every re-exported \
         name as unresolved would pass the assertion above and export nothing."
    );
    assert!(
        !surface.unresolved.iter().any(|line| return line.contains(resolved)),
        "{resolved} is declared in this crate and was reported as unfollowable"
    );
}
