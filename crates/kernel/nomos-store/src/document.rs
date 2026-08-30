use nomos_contracts::SchemaId;
use serde::{Deserialize, Serialize};

// A document's identity and its kind are parts of a document, not peers of one.
mod id;
mod kind;

pub use id::Id as DocumentId;
pub use kind::Kind as DocumentKind;

use crate::Authority;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document
{
    pub kind: DocumentKind,
    pub schema: SchemaId,
    pub bytes: Vec<u8>,
}

impl Document
{
    #[must_use]
    pub fn New(kind: DocumentKind, schema: SchemaId, bytes: Vec<u8>) -> Self
    {
        return Self {
            kind,
            schema,
            bytes,
        };
    }

    #[must_use]
    pub fn Id(&self) -> DocumentId
    {
        use nomos_model::Digest_Of_Parts;

        return DocumentId::From_Digest(Digest_Of_Parts(&[
            self.kind.Label().as_bytes(),
            self.schema.As_Str().as_bytes(),
            &self.bytes,
        ]));
    }

    #[must_use]
    pub fn Authority(&self) -> Authority
    {
        return self.kind.Authority();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Arbitrary_Document() -> Document
    {
        return Document::New(DocumentKind::Fact, SchemaId::New("nomos.syntax.v1"), b"fn main() {}".to_vec());
    }

    #[test]
    fn Test_New_Should_Build_A_Document_From_Its_Kind_Schema_And_Bytes()
    {
        let document = Arbitrary_Document();

        assert_eq!(document.kind, DocumentKind::Fact);
        assert_eq!(document.bytes, b"fn main() {}".to_vec());
    }

    #[test]
    fn Test_Id_Should_Be_Stable_For_The_Same_Kind_Schema_And_Bytes()
    {
        assert_eq!(Arbitrary_Document().Id(), Arbitrary_Document().Id());
    }

    #[test]
    fn Test_Authority_Should_Delegate_To_The_Documents_Kind()
    {
        assert_eq!(Arbitrary_Document().Authority(), DocumentKind::Fact.Authority());
    }
}
