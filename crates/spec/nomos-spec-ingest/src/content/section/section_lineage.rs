//! Where a section came from.
//!
//! Named `SectionLineage` rather than `Lineage` because this crate publishes its whole
//! vocabulary at one flat root, where the section's lineage and a block's would otherwise
//! share a name.

use crate::RecordedSection;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct SectionLineage
{
    pub sections: Vec<RecordedSection>,
}
