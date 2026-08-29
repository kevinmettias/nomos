//! One heading of a source document, at its depth.

use serde::{Deserialize, Serialize};

use crate::DocumentRef;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceHeading
{
    pub document: DocumentRef,
    pub ordinal: i64,
    pub depth: i64,
    pub title: String,
}
