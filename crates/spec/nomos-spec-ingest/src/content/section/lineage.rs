//! Where a section came from.

use crate::RecordedSection;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct SectionLineage
{
    pub sections: Vec<RecordedSection>,
}
