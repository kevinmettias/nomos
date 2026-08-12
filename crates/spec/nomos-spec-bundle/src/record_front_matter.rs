//! The front matter one record declared, as it declared it.

use serde::{Deserialize, Serialize};

use crate::document_ref::DocumentRef;

/// The front matter one record declared, as it declared it.
///
/// Carried rather than derived from [`Node`] and [`Relation`], for the reason the schema
/// gives: the graph is inverse-completed and undirected about `relates-to`, so it cannot say
/// which end of an edge wrote it down. A bundle that dropped this would rebuild a store that
/// can preserve every record and render none of them back.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordFrontMatter
{
    pub document: DocumentRef,
    pub node_id: String,
    pub status: String,
    pub version: i64,
    pub tags: Vec<String>,
}
