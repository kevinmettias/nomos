//! Source content deliberately not restored, and the record that decided so.

use serde::{Deserialize, Serialize};

use crate::OrdinalRef;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Omission
{
    pub source_block: Option<OrdinalRef>,
    pub source_heading: Option<OrdinalRef>,
    pub reason: String,
    pub justification: String,
    pub decision_record: String,
}
