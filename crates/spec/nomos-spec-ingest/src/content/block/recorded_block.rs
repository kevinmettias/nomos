//! One block as the store holds it.

use serde::Deserialize;
/// One block as v14 recorded it.
#[derive(Debug, Deserialize)]
pub struct RecordedBlock
{
    pub source_document: String,
    pub block_ordinal: u32,
    pub block_kind: String,
    pub content_hash: String,
    pub normalized_hash: String,
    #[serde(default)]
    pub disposition: String,
}
