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
fn Reference_For(record: &Recorded) -> Reference
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
            records: self.records.iter().map(Reference_For).collect(),
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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::DocumentKind;
    use nomos_contracts::{Digest128, SchemaId};

    /// The seeds `Empty_Commit` derives its build variant and configuration from. Named so
    /// a reader can see the two digests are deliberately different.
    const BUILD_VARIANT_SEED: u8 = 2;
    const CONFIGURATION_SEED: u8 = 3;

    #[test]
    fn Test_Under_Should_Start_A_Commit_With_No_Records()
    {
        let commit = Empty_Commit();

        assert!(commit.records.is_empty());
        assert_eq!(commit.snapshot, SnapshotId::From_Digest(Seeded_Digest(1)));
    }

    #[test]
    fn Test_Recording_Should_Append_A_Document_To_The_Commit()
    {
        let commit = Empty_Commit().Recording(Fact_Record("fn main() {}"));

        assert_eq!(commit.records.len(), 1);
    }

    #[test]
    fn Test_Encode_Should_Produce_Bytes_That_Reconstruct_The_Same_Manifest()
    {
        let commit = Empty_Commit().Recording(Fact_Record("fn main() {}"));

        let encoded = commit.Encode().expect("Manifest holds only schema strings and digest fields, which serde_json encodes");
        let manifest = Commit::Decode(&encoded).expect("reconstructs");

        assert_eq!(manifest.schema, COMMIT_SCHEMA);
        assert_eq!(manifest.snapshot, commit.snapshot);
        assert_eq!(manifest.records.len(), commit.records.len());
    }

    #[test]
    fn Test_Decode_Should_Refuse_A_Manifest_With_A_Different_Schema()
    {
        let refusal = Commit::Decode(br#"{"schema":"nomos.other.v1"}"#).expect_err("must refuse");

        assert!(matches!(refusal, StoreError::Malformed(_)));
    }

    fn Seeded_Digest(seed: u8) -> Digest128
    {
        return Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);
    }

    fn Fact_Record(payload: &str) -> Recorded
    {
        return Recorded::New(DocumentKind::Fact, SchemaId::New("nomos.syntax.v1"), payload.as_bytes().to_vec());
    }

    fn Empty_Commit() -> Commit
    {
        return Commit::Under(
            SnapshotId::From_Digest(Seeded_Digest(1)),
            BuildVariantId::From_Digest(Seeded_Digest(BUILD_VARIANT_SEED)),
            ConfigurationId::From_Digest(Seeded_Digest(CONFIGURATION_SEED)),
            GenerationId::INITIAL,
        );
    }
}
