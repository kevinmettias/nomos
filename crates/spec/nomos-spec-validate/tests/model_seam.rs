//! The seam between this crate and `nomos_spec_model`.
//!
//! Every [`ValidationRun`](nomos_spec_validate::ValidationRun) carries a `ContentHash`
//! identifying the ruleset that produced it, but this crate does not define what a
//! `ContentHash` is, how it is computed, or how two of them compare — `nomos_spec_model`
//! does. Nothing in this crate's own tests calls into `nomos_spec_model` directly; they only
//! compare opaque hashes this crate already produced. This suite drives `nomos_spec_model`
//! through this crate's public API instead: the happy path (the hash this crate reports is
//! the hash `nomos_spec_model` would independently compute for the same ruleset), the
//! lifecycle either side imposes on the other (a narrowed ruleset must change the hash), and
//! round-tripping this crate's hash back through `nomos_spec_model`'s own parser.

use nomos_spec_model::ContentHash;
use nomos_spec_store::SpecificationStore;
use nomos_spec_validate::{DECLARED_RULES, Registered, Validate_Rules};

fn Store() -> SpecificationStore
{
    return SpecificationStore::In_Memory()
        .expect("an in-memory store opens no file, so only the shipped schema can fail here");
}

/// The happy path across the boundary: the hash a run reports must be the hash
/// `nomos_spec_model::ContentHash::Of` independently computes over the same declared
/// ruleset, sorted the same way. If this crate ever hashed something other than what it
/// claims to, this is the seam that would notice — both sides are individually correct in
/// their own tests, and only this proves they agree on what a ruleset hash is.
#[test]
fn Test_Validate_Rules_Should_Report_The_Hash_Nomos_Spec_Model_Computes_For_The_Ruleset()
{
    let run = Validate_Rules(&Store(), &Registered());

    let mut sorted: Vec<&str> = DECLARED_RULES.to_vec();
    sorted.sort_unstable();
    let expected = ContentHash::Of(&sorted.join("\n"));

    assert_eq!(run.ruleset_hash, expected);
}

/// The lifecycle this crate imposes on `nomos_spec_model`: a different ruleset must produce
/// a different `ContentHash`. This is a property of `nomos_spec_model`'s hashing that this
/// crate's own rule depends on to make a narrowed ruleset detectable at all.
#[test]
fn Test_A_Narrowed_Ruleset_Should_Change_The_Hash_Nomos_Spec_Model_Computed_For_It()
{
    let full = Validate_Rules(&Store(), &Registered());

    let mut narrowed = Registered();
    narrowed.pop();
    let narrowed_run = Validate_Rules(&Store(), &narrowed);

    assert_ne!(
        full.ruleset_hash, narrowed_run.ruleset_hash,
        "removing a rule must change the hash nomos_spec_model computed for it"
    );
}

/// A hash this crate produced must itself be a value `nomos_spec_model` recognizes as one of
/// its own: round-tripping it through `ContentHash::Parse` and `As_String_Slice` must return
/// exactly what this crate reported, proving this crate is producing a real `ContentHash`
/// and not merely a string that happens to look like one.
#[test]
fn Test_The_Ruleset_Hash_Should_Round_Trip_Through_Content_Hash_Parse()
{
    let run = Validate_Rules(&Store(), &Registered());

    let text = run.ruleset_hash.As_String_Slice();
    let parsed = ContentHash::Parse(text).expect("a hash this crate produced must itself parse");

    assert_eq!(parsed, run.ruleset_hash);
}
