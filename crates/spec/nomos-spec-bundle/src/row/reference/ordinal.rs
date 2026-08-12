//! A block or a heading, addressed by its position inside a document.

use serde::{Deserialize, Serialize};

use crate::row::reference::document::DocumentRef;

/// A block or a heading, addressed by its position inside a document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrdinalRef
{
    pub document: DocumentRef,
    pub ordinal: i64,
}
