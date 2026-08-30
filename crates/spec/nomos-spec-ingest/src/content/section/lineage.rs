//! Where a section came from.

use crate::RecordedSection;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct Lineage
{
    pub sections: Vec<RecordedSection>,
}
