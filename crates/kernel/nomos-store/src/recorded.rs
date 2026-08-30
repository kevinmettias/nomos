//! One document as a commit hands it over: what it is, and its bytes.

use nomos_contracts::SchemaId;
use serde::{Deserialize, Serialize};

use crate::Document;
use crate::DocumentKind;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recorded
{
    pub kind: DocumentKind,
    pub schema: SchemaId,
    pub bytes: Vec<u8>,
}

impl Recorded
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
    pub fn Document(&self) -> Document
    {
        return Document::New(self.kind, self.schema.clone(), self.bytes.clone());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Arbitrary_Recorded() -> Recorded
    {
        return Recorded::New(DocumentKind::Fact, SchemaId::New("nomos.syntax.v1"), b"fn main() {}".to_vec());
    }

    #[test]
    fn Test_New_Should_Build_A_Recorded_From_Its_Kind_Schema_And_Bytes()
    {
        let recorded = Arbitrary_Recorded();

        assert_eq!(recorded.kind, DocumentKind::Fact);
        assert_eq!(recorded.bytes, b"fn main() {}".to_vec());
    }

    #[test]
    fn Test_Document_Should_Convert_A_Recorded_Into_A_Document_Of_The_Same_Kind()
    {
        let document = Arbitrary_Recorded().Document();

        assert_eq!(document.kind, DocumentKind::Fact);
        assert_eq!(document.bytes, b"fn main() {}".to_vec());
    }
}
