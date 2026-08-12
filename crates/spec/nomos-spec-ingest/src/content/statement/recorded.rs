//! One normative statement as the store holds it.

use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct RecordedStatement
{
    pub id: String,
    pub kind: String,
    pub canonical_text: String,
    pub canonical_hash: String,
    #[serde(default)]
    pub source_document: String,
}
