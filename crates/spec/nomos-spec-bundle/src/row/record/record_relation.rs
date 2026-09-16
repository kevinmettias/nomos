//! One relation a record declared, in the position it declared it.
//!
//! Named `RecordRelation` rather than `Relation` because this crate also declares
//! `row::relation::Relation`, the typed edge between two nodes, and that one publishes the
//! bare name at the crate root. The two rows are different tables answering different
//! questions, so the declared one carries the word that says which table it came from.

use serde::{Deserialize, Serialize};

use crate::DocumentRef;

/// One relation a record declared, in the position it declared it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordRelation
{
    pub document: DocumentRef,
    pub ordinal: i64,
    pub target: String,
    pub relation: String,
}
