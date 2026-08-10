//! What a commit says about itself once it has been written down.

use nomos_contracts::{
    BuildVariantId, ConfigurationId, GenerationId, SnapshotId,
};
use serde::{Deserialize, Serialize};

use crate::reference::Reference;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest
{
    pub schema: String,
    pub snapshot: SnapshotId,
    pub variant: BuildVariantId,
    pub configuration: ConfigurationId,
    pub generation: GenerationId,
    pub records: Vec<Reference>,
}
