//! The seam between `nomos_analysis` and `nomos_model`, driven through `nomos_analysis`'s
//! own public API — the same view a real consumer has.
//!
//! `FactPayload::Digest`, `InputDigest::Of` and `GuaranteeDigest::Of` each delegate to
//! `nomos_model::Content_Digest` or `nomos_model::Digest_Of_Parts` internally, and nothing
//! under this crate's own `tests/` called into `nomos_model` before this file — every
//! digest this crate produced was only ever checked against itself.

use nomos_analysis::{FactPayload, GuaranteeDigest, InputDigest};
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, SchemaId};
use nomos_model::{Content_Digest, Digest_Of_Parts};

/// `FactPayload::Digest` is exactly `nomos_model::Content_Digest` over the payload's own
/// bytes — the construction a caller comparing two payloads by hand would perform.
#[test]
fn Test_A_Fact_Payloads_Digest_Should_Match_Nomos_Models_Own_Content_Digest()
{
    let payload = FactPayload::New(SchemaId::New("nomos.test.payload.v1"), b"hello".to_vec());

    assert_eq!(payload.Digest(), Content_Digest(b"hello"));
}

/// The negative control: two payloads with different bytes must not collide through
/// `nomos_model`'s own digest either, or the equality above would hold for any two
/// payloads.
#[test]
fn Test_Different_Payloads_Should_Not_Share_Nomos_Models_Digest()
{
    let one = FactPayload::New(SchemaId::New("nomos.test.payload.v1"), b"hello".to_vec());
    let other = FactPayload::New(SchemaId::New("nomos.test.payload.v1"), b"world".to_vec());

    assert_ne!(one.Digest(), other.Digest());
    assert_eq!(one.Digest(), Content_Digest(b"hello"));
    assert_eq!(other.Digest(), Content_Digest(b"world"));
}

/// `InputDigest::Of` is exactly `nomos_model::Digest_Of_Parts` over the same parts — the
/// property a caller recomputing a fact's semantic inputs independently relies on.
#[test]
fn Test_An_Input_Digest_Should_Match_Nomos_Models_Own_Digest_Of_Parts()
{
    let parts: &[&[u8]] = &[b"a", b"b"];

    assert_eq!(InputDigest::Of(parts).Digest(), Digest_Of_Parts(parts));
}

/// `GuaranteeDigest::Of` is `nomos_model::Digest_Of_Parts` over the guarantee's own four
/// axes, in the same order this crate's own implementation feeds them — checked here from
/// the outside, through the public field access a real consumer of `nomos_contracts::
/// Guarantee` has.
#[test]
fn Test_A_Guarantee_Digest_Should_Match_Nomos_Models_Own_Digest_Of_Parts()
{
    let guarantee = Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );

    let expected = Digest_Of_Parts(&[
        &[guarantee.variant as u8],
        &[guarantee.soundness as u8],
        &[guarantee.completeness as u8],
        &[guarantee.incremental as u8],
    ]);

    assert_eq!(GuaranteeDigest::Of(&guarantee).Digest(), expected);
}
