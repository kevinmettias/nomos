//! Where a section came from.

use crate::content::section::recorded::RecordedSection;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct SectionLineage
{
    pub sections: Vec<RecordedSection>,
}
