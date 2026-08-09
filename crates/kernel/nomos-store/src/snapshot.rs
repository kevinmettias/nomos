use crate::document::{Document, DocumentId, DocumentKind};
use crate::StoreError;
use nomos_contracts::{
    BuildVariantId, ConfigurationId, GenerationId, SchemaId, SnapshotId,
};
use serde::{Deserialize, Serialize};

pub const SNAPSHOT_SCHEMA: &str = "nomos.snapshot.v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recorded
{
    pub kind: DocumentKind,
    pub schema: SchemaId,
    pub bytes: Vec<u8>,
}

impl Recorded
{
    #[must_use]
    pub fn New(kind: DocumentKind, schema: SchemaId, bytes: Vec<u8>) -> Self
    {
        return Self {
            kind,
            schema,
            bytes,
        };
    }

    #[must_use]
    pub fn Document(&self) -> Document
    {
        return Document::New(self.kind, self.schema.clone(), self.bytes.clone());
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot
{
    pub snapshot: SnapshotId,
    pub variant: BuildVariantId,
    pub configuration: ConfigurationId,
    pub generation: GenerationId,
    pub records: Vec<Recorded>,
}

impl Snapshot
{
    #[must_use]
    pub fn Of(
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

    pub fn Encode(&self) -> Result<Vec<u8>, StoreError>
    {
        let manifest = Manifest {
            schema: SNAPSHOT_SCHEMA.to_owned(),
            snapshot: self.snapshot,
            variant: self.variant,
            configuration: self.configuration,
            generation: self.generation,
            records: self
                .records
                .iter()
                .map(|record| {
                    return Reference {
                        kind: record.kind,
                        schema: record.schema.clone(),
                        document: record.Document().Id(),
                    };
                })
                .collect(),
        };

        let mut encoded = serde_json::to_vec(&manifest)
            .map_err(|error| return StoreError::Malformed(error.to_string()))?;
        encoded.push(b'\n');

        return Ok(encoded);
    }

    pub fn Decode(bytes: &[u8]) -> Result<Manifest, StoreError>
    {
        let manifest: Manifest = serde_json::from_slice(bytes)
            .map_err(|error| return StoreError::Malformed(error.to_string()))?;

        if manifest.schema != SNAPSHOT_SCHEMA
        {
            return Err(StoreError::Malformed(format!(
                "{} is not {SNAPSHOT_SCHEMA}",
                manifest.schema
            )));
        }

        return Ok(manifest);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reference
{
    pub kind: DocumentKind,
    pub schema: SchemaId,
    pub document: DocumentId,
}

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
