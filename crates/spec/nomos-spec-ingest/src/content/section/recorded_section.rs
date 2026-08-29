//! One section as the store holds it.

use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct RecordedSection
{
    pub source_document: String,
    pub source_heading: String,
    #[serde(default)]
    pub heading_level: i64,
    pub disposition: String,
    #[serde(default)]
    pub stable_ids: Vec<String>,
}
