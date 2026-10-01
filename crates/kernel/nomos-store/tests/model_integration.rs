//! The seam this crate depends on but never crossed from its own test corpus: every document
//! identity `nomos-store` hands out is computed by `nomos-model`'s content addressing, not by
//! anything this crate invents on its own. `check-integration-coverage` flagged the pairing as an
//! integration surface with no suite -- `nomos_store` reaches `nomos_model` in `document.rs` and
//! `index.rs`, and nothing in `tests/` had ever said so by importing it directly. This file drives
//! `nomos_model` through `nomos_store`'s public API, so the contract between the two is checked
//! from the same side a real consumer sees it from: through `Document`, not through the hash
//! function it delegates to.

use nomos_contracts::SchemaId;
use nomos_model::Digest_Of_Parts;
use nomos_store::{Document, DocumentKind};

/// The happy path: a document's id is not an internal detail `nomos-store` is free to change --
/// it is exactly `nomos_model::Digest_Of_Parts` applied to the document's labeled kind, its schema
/// and its bytes. If this ever drifts, every stored identity in every snapshot silently means
/// something else.
#[test]
fn Test_A_Documents_Id_Should_Be_Nomos_Models_Digest_Of_Its_Labeled_Parts()
{
    let document = Document::New(DocumentKind::Fact, SchemaId::New("nomos.syntax.v1"), b"fn main() {}".to_vec());

    let expected = Digest_Of_Parts(&[
        DocumentKind::Fact.Label().as_bytes(),
        "nomos.syntax.v1".as_bytes(),
        b"fn main() {}",
    ]);

    assert_eq!(
        document.Id().Digest(),
        expected,
        "a document's id must be exactly nomos_model's digest of its parts, or the two crates have \
         quietly disagreed about what identity means"
    );
}

/// The lifecycle nomos-store depends on: two documents whose kind, schema and bytes agree must
/// collide on identity, and any one of the three changing must not. This is `Digest_Of_Parts`'s
/// own framing property (see its unit tests in `nomos-model`), observed here through the crate
/// that actually leans on it for deduplication.
#[test]
fn Test_Document_Identity_Should_Follow_Digest_Of_Parts_Across_The_Boundary()
{
    let one = Document::New(DocumentKind::Fact, SchemaId::New("nomos.syntax.v1"), b"fn main() {}".to_vec());
    let same_parts = Document::New(DocumentKind::Fact, SchemaId::New("nomos.syntax.v1"), b"fn main() {}".to_vec());
    let different_bytes =
        Document::New(DocumentKind::Fact, SchemaId::New("nomos.syntax.v1"), b"struct Other;".to_vec());
    let different_kind =
        Document::New(DocumentKind::Finding, SchemaId::New("nomos.syntax.v1"), b"fn main() {}".to_vec());

    assert_eq!(
        one.Id(),
        same_parts.Id(),
        "identical parts must digest identically, or the store can never recognize a re-commit"
    );
    assert_ne!(one.Id(), different_bytes.Id(), "different bytes must not collide");
    assert_ne!(
        one.Id(),
        different_kind.Id(),
        "a fact and a finding built from the same bytes must not collide, or the store could \
         confuse what kind of thing it is holding"
    );
}
