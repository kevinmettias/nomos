//! Turning `standards.json`'s own declared rows into the one fact this capability answers.

use super::guarantee::{Declared_Guarantee, PROVIDER};
use super::reading::{Discover_Workspace, LimitsPolicyError};
use crate::scaffolding;
use nomos_cap_limits_policy::{Capability, Encode_Payload, LimitsPolicyPayload, Payload_Schema, CONTRACT_VERSION};
use nomos_platform::FileSystem;
use std::path::Path;

pub use crate::scaffolding::{FactContext, PolicyFact};

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
    let identity = scaffolding::CapabilityIdentity { capability: Capability(), contract_version: CONTRACT_VERSION, provider: PROVIDER, provider_version: CONTRACT_VERSION };
    let key = scaffolding::Compute_Fact_Key(&identity, subject, guarantee, context);
    let payload = scaffolding::EncodedPayload { schema: Payload_Schema(), bytes: payload_bytes };

    return Ok(scaffolding::Materialize_Fact(subject, guarantee, scaffolding::FactFiling { context, key }, payload));
}

#[cfg(test)]
mod tests
{
    use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
    use super::*;
    use nomos_analysis::GuaranteeDigest;
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

    /// Answers from fixed data, so its outputs reproduce byte for byte.
    impl Strategy for FakeFileSystem
    {
        const STRENGTH: DeterminismStrength = DeterminismStrength::State;
        const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
        const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
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

        let identity = scaffolding::CapabilityIdentity { capability: Capability(), contract_version: CONTRACT_VERSION, provider: PROVIDER, provider_version: CONTRACT_VERSION };
        let strong_key = scaffolding::Compute_Fact_Key(&identity, subject, Declared_Guarantee(), Context());
        let weak_key = scaffolding::Compute_Fact_Key(&identity, subject, weaker, Context());

        assert_ne!(strong_key.Digest(), weak_key.Digest(), "two offers of the same subject at different guarantees must file apart");
    }

    /// This test's own fixture, shared with its four siblings -- see
    /// `crate::scaffolding::test_support::Sample_Context`.
    fn Context() -> FactContext
    {
        return scaffolding::test_support::Sample_Context();
    }
}
