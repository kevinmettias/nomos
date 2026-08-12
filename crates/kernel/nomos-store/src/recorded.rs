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
