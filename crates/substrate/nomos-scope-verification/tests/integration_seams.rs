//! The real crate-to-crate seams `nomos-scope-verification` reaches across, driven only
//! through this crate's own public API — the same view a real consumer has.
//!
//! `nomos-scope-verification` depends on two other crates in its source: `nomos_contracts`
//! and `nomos_model`. Neither had a suite under this crate's own `tests/` before this file
//! — `src/territory/tests.rs` and the literal `local_tests` module beside it both prove
//! real properties, but both compile *inside* this crate's own `src/`, not under `tests/`,
//! so neither proves the PUBLIC contract on its own.

use nomos_contracts::SubjectId;
use nomos_model::{Content_Digest, Intersection};
use nomos_scope_verification::Territory;

// -----------------------------------------------------------------------------------------
// nomos_contracts — the identity `Territory::As_Subject_Set` states its members as.
// -----------------------------------------------------------------------------------------

/// `nomos_contracts`: the identity a territory names for an ordinary (non-record) path is
/// exactly `nomos_contracts::SubjectId::From_Digest` over `nomos_model::Content_Digest` of
/// the normalized path — the same construction a caller comparing a territory's members
/// against an independently computed subject would perform.
#[test]
fn Test_A_Territorys_Subject_Should_Match_An_Independently_Constructed_Subject_Id()
{
    let territory = Territory::Of_Files(["crates/a/src/lib.rs"]);

    let expected = SubjectId::From_Digest(Content_Digest(b"crates/a/src/lib.rs"));
    let subjects = territory.As_Subject_Set();
    let members: Vec<&SubjectId> = subjects.Members().collect();

    assert_eq!(members, vec![&expected]);
}

/// `nomos_contracts`: two spellings of one file fold onto one `SubjectId`, the identity
/// property `Territory::Intersect` relies on to catch two agents naming the same file two
/// different ways.
#[test]
fn Test_Two_Spellings_Of_One_File_Should_Share_One_Subject_Id()
{
    let canonical = Territory::Of_Files(["crates/a/src/lib.rs"]).As_Subject_Set();
    let shouted = Territory::Of_Files(["./Crates\\A\\Src\\LIB.rs"]).As_Subject_Set();

    let canonical_members: Vec<&SubjectId> = canonical.Members().collect();
    let shouted_members: Vec<&SubjectId> = shouted.Members().collect();

    assert_eq!(canonical_members, shouted_members);
}

// -----------------------------------------------------------------------------------------
// nomos_model — the comparison `Territory::Intersect` states its answer in.
// -----------------------------------------------------------------------------------------

/// `nomos_model`: two territories over disjoint paths compare `nomos_model::Intersection::
/// Disjoint`, and a disjoint answer permits concurrency — the property the work ledger
/// relies on to hand out two claims at once.
#[test]
fn Test_Disjoint_Territories_Should_Compare_Disjoint_And_Permit_Concurrency()
{
    let left = Territory::Of_Files(["crates/a/src/lib.rs"]);
    let right = Territory::Of_Files(["crates/b/src/lib.rs"]);

    let answer = left.Intersect(&right);

    assert_eq!(answer, Intersection::Disjoint);
    assert!(answer.Permits_Concurrency());
}

/// `nomos_model`: two territories sharing a path compare `nomos_model::Intersection::
/// Overlaps`, and an overlap must not permit concurrency — the refusal the work ledger
/// relies on to keep two agents from claiming the same file.
#[test]
fn Test_Overlapping_Territories_Should_Compare_Overlaps_And_Refuse_Concurrency()
{
    let left = Territory::Of_Files(["crates/a/src/lib.rs"]);
    let right = Territory::Of_Files(["crates/a/src/lib.rs", "crates/b/src/lib.rs"]);

    let answer = left.Intersect(&right);

    assert!(matches!(answer, Intersection::Overlaps(_)));
    assert!(!answer.Permits_Concurrency());
}
