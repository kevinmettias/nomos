use crate::Manifest;
use crate::Recorded;
use crate::Reference;
use crate::StoreError;
use nomos_contracts::{
    BuildVariantId, ConfigurationId, GenerationId, SnapshotId,
};
use serde::{Deserialize, Serialize};

pub const COMMIT_SCHEMA: &str = "nomos.commit.v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commit
{
    pub snapshot: SnapshotId,
    pub variant: BuildVariantId,
    pub configuration: ConfigurationId,
    pub generation: GenerationId,
    pub records: Vec<Recorded>,
}

/// One record as the manifest names it: what it is, and the identity of its bytes.
///
/// The document identity is computed here rather than carried, because it is a function of
/// the bytes and a manifest that stated a stale one would name a document nobody holds.
fn Referenced(record: &Recorded) -> Reference
{
    return Reference {
        kind: record.kind,
        schema: record.schema.clone(),
        document: record.Document().Id(),
    };
}

impl Commit
{
    #[must_use]
    pub fn Under(
        snapshot: SnapshotId,
        variant: BuildVariantId,
        configuration: ConfigurationId,
        generation: GenerationId,
    ) -> Self
    {
        return Self {
            snapshot,
            variant,
            configuration,
            generation,
            records: Vec::new(),
        };
    }

    #[must_use]
    pub fn Recording(mut self, record: Recorded) -> Self
    {
        self.records.push(record);

        return self;
    }

    /// # Errors
    ///
    /// Returns [`StoreError::Malformed`] if the manifest fails to serialize.
    pub fn Encode(&self) -> Result<Vec<u8>, StoreError>
    {
        let manifest = Manifest {
            schema: COMMIT_SCHEMA.to_owned(),
            snapshot: self.snapshot,
            variant: self.variant,
            configuration: self.configuration,
            generation: self.generation,
            records: self.records.iter().map(Referenced).collect(),
        };

        let mut encoded = serde_json::to_vec(&manifest)
            .map_err(|error| return StoreError::Malformed(error.to_string()))?;
        encoded.push(b'\n');

        return Ok(encoded);
    }

    /// # Errors
    ///
    /// Returns [`StoreError::Malformed`] if `bytes` does not parse as a manifest, or parses as
    /// one whose schema is not [`COMMIT_SCHEMA`].
    pub fn Decode(bytes: &[u8]) -> Result<Manifest, StoreError>
    {
        let manifest: Manifest = serde_json::from_slice(bytes)
            .map_err(|error| return StoreError::Malformed(error.to_string()))?;

        if manifest.schema != COMMIT_SCHEMA
        {
            return Err(StoreError::Malformed(format!(
                "{} is not {COMMIT_SCHEMA}",
                manifest.schema
            )));
        }

        return Ok(manifest);
    }
}
