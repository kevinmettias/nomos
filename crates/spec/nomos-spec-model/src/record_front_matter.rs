//! The declared identity of a record.

use serde::Deserialize;

use crate::record_relation::RecordRelation;

/// The declared identity of a record.
///
/// The field names are the v14 corpus's, so the same reader serves this repository's own
/// records and the records restored from the archives. Two readers for one format is how
/// the two disagree.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct RecordFrontMatter
{
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub title: String,
    pub status: String,
    pub authority: String,
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub relations: Vec<RecordRelation>,
}
