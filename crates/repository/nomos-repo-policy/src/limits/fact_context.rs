//! Turning `standards.json`'s own declared rows into the one fact this capability answers.

use super::guarantee::{Declared_Guarantee, PROVIDER};
use super::reading::{Discover_Workspace, LimitsPolicyError};
use nomos_analysis::{FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_cap_limits_policy::{Capability, Encode_Payload, LimitsPolicyPayload, Payload_Schema, CONTRACT_VERSION};
use nomos_contracts::{
    BuildVariantId, ConfigurationId, EvidenceClass, GenerationId, Guarantee, ProviderId, SnapshotId,
    SubjectId,
};
use nomos_platform::FileSystem;
use std::path::Path;

#[path = "provider/policy_fact.rs"]
mod policy_fact;

pub use policy_fact::PolicyFact;

/// Where in the workspace's history a fact is being produced — the same four-field shape
/// `crate::naming::FactContext` carries, for the identical reason: these four always
/// travel together.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactContext
{
    pub snapshot: SnapshotId,
    pub variant: BuildVariantId,
    pub configuration: ConfigurationId,
    pub generation: GenerationId,
}

/// Reads `root`'s own `standards.json` and produces the one fact this capability answers
/// for the workspace as a whole — a leaf: nothing here reads another fact this or any
/// other provider produced.
///
/// Deliberately not itself a `nomos_analysis::MemoryFactStore` writer, the same division
/// `crate::naming::Materialize_Workspace` draws for the identical reason: a
/// composition root calls this and writes the returned [`PolicyFact`] into the store it
/// owns.
///
/// # Errors
///
/// Whatever [`Discover_Workspace`] returns.
pub fn Materialize_Workspace<Fs: FileSystem>(root: &Path, context: FactContext, filesystem: &Fs) -> Result<PolicyFact, LimitsPolicyError>
{
    let rows = Discover_Workspace(root, filesystem)?;
    let payload = LimitsPolicyPayload { rows };
    let subject = nomos_model::Subject_Of_Path("");
    let guarantee = Declared_Guarantee();
    let payload_bytes = Encode_Payload(&payload);
    let key = Compute_Fact_Key(subject, guarantee, context);
    let fact = MaterializedFact {
        identity: key.At(context.generation),
        snapshot: context.snapshot,
        evidence: EvidenceClass::Verified,
        guarantee,
        payload: FactPayload::New(Payload_Schema(), payload_bytes),
    };

    return Ok(PolicyFact { subject, fact });
}

/// The key this fact is filed under.
///
/// `semantic_inputs` is empty, deliberately, the same choice
/// `crate::naming::fact_context::Compute_Fact_Key` already makes for the identical
/// reason: this provider's real input is `standards.json`'s own current text, which no
/// caller has independently, so a caller building a lookup key has nothing to reconstruct
/// it from.
fn Compute_Fact_Key(subject: SubjectId, guarantee: Guarantee, context: FactContext) -> FactKey
{
    return FactKey {
        contract: Capability(),
        contract_version: CONTRACT_VERSION,
        subject,
        semantic_inputs: InputDigest::Of(&[]),
        provider: ProviderId::New(PROVIDER),
        provider_version: CONTRACT_VERSION,
        guarantee: GuaranteeDigest::Of(&guarantee),
        variant: context.variant,
        configuration: context.configuration,
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::Digest128;
    use nomos_platform::FileSystemError;

    /// The declared threshold this fixture's `standards.json` text carries.
    const SAMPLE_LIMIT: u32 = 1500;

    /// A [`FileSystem`] that hands back fixed text instead of reading a real path, the
    /// boundary [`crate::reading`]'s own module doc names as the one place a caller
    /// substitutes a real filesystem.
    struct FakeFileSystem
    {
        text: String,
    }

    impl FileSystem for FakeFileSystem
    {
        fn Read_To_String(&self, _path: &Path) -> Result<String, FileSystemError>
        {
            return Ok(self.text.clone());
        }

        fn Replace_Atomically(&self, _path: &Path, _contents: &str) -> Result<(), FileSystemError>
        {
            unimplemented!("this reader never writes")
        }

        fn Exists(&self, _path: &Path) -> bool
        {
            return true;
        }
    }

    #[test]
    fn Test_Materialize_Workspace_Should_Materialize_Declared_Thresholds()
    {
        let filesystem = FakeFileSystem {
            text: serde_json::json!({ "limits": { "file-size-hard-lines": SAMPLE_LIMIT } }).to_string(),
        };

        let PolicyFact { subject, fact } = Materialize_Workspace(Path::new("."), Context(), &filesystem).expect("well-formed JSON");

        assert_eq!(subject, nomos_model::Subject_Of_Path(""));
        assert_eq!(fact.guarantee, Declared_Guarantee());
        assert_eq!(fact.Key().guarantee, GuaranteeDigest::Of(&Declared_Guarantee()));
        assert_eq!(fact.payload.schema, Payload_Schema());

        let decoded = nomos_cap_limits_policy::Parse_Payload(&fact.payload.bytes).expect("this crate's own encoding");
        assert!(
            decoded.rows.iter().any(|row| return row.key == "file-size-hard-lines" && row.value == SAMPLE_LIMIT),
            "the fixture declares limits.file-size-hard-lines = {SAMPLE_LIMIT}: {decoded:?}"
        );
    }

    #[test]
    fn Test_A_Fact_Key_Should_Depend_On_The_Guarantee()
    {
        let subject = nomos_model::Subject_Of_Path("");
        let weaker = nomos_contracts::Guarantee::New(
            nomos_contracts::FactVariant::Syntactic,
            nomos_contracts::Assurance::Unsound,
            nomos_contracts::Assurance::Unknown,
            nomos_contracts::IncrementalGranularity::WholeWorkspace,
        );

        let strong_key = Compute_Fact_Key(subject, Declared_Guarantee(), Context());
        let weak_key = Compute_Fact_Key(subject, weaker, Context());

        assert_ne!(strong_key.Digest(), weak_key.Digest(), "two offers of the same subject at different guarantees must file apart");
    }

    /// Fill bytes distinct enough that `Context()`'s three digests differ from one
    /// another; each value carries no meaning beyond "not equal to the others".
    const VARIANT_DIGEST_FILL: u8 = 2;
    const CONFIGURATION_DIGEST_FILL: u8 = 3;

    fn Context() -> FactContext
    {
        return FactContext {
            snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; Digest128::BYTE_LENGTH])),
            variant: BuildVariantId::From_Digest(Digest128::From_Bytes([VARIANT_DIGEST_FILL; Digest128::BYTE_LENGTH])),
            configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([
                CONFIGURATION_DIGEST_FILL;
                Digest128::BYTE_LENGTH
            ])),
            generation: GenerationId::INITIAL,
        };
    }
}
