//! One relation a record declared, in the position it declared it.

use serde::{Deserialize, Serialize};

use crate::DocumentRef;

/// One relation a record declared, in the position it declared it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relation
{
    pub document: DocumentRef,
    pub ordinal: i64,
    pub target: String,
    pub relation: String,
}
