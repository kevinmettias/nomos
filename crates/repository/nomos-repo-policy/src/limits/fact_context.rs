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
    let payload = LimitsPolicyPayload { rows: Discover_Workspace(root, filesystem)? };

    return Ok(scaffolding::Materialize_Whole_Workspace(scaffolding::WholeWorkspaceFiling {
        subject: scaffolding::Workspace_Subject(),
        guarantee: Declared_Guarantee(),
        identity: scaffolding::Declared_Identity(Capability(), CONTRACT_VERSION, PROVIDER, CONTRACT_VERSION),
        context,
        schema: Payload_Schema(),
        bytes: Encode_Payload(&payload),
    }));
}

#[cfg(test)]
mod tests
{
    use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
    use super::*;
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

        let decoded = nomos_cap_limits_policy::Parse_Payload(&fact.payload.bytes).expect("this crate's own encoding");
        assert!(
            decoded.rows.iter().any(|row| return row.key == "file-size-hard-lines" && row.value == SAMPLE_LIMIT),
            "the fixture declares limits.file-size-hard-lines = {SAMPLE_LIMIT}: {decoded:?}"
        );

        scaffolding::test_support::Assert_Carries_Its_Declaration(&subject, &fact, Declared_Guarantee(), &Payload_Schema());
    }

    #[test]
    fn Test_A_Fact_Key_Should_Depend_On_The_Guarantee()
    {
        let identity = scaffolding::Declared_Identity(Capability(), CONTRACT_VERSION, PROVIDER, CONTRACT_VERSION);

        scaffolding::test_support::Assert_Keys_File_Apart_By_Guarantee(&identity, Declared_Guarantee());
    }

    /// This test's own fixture, shared with its five siblings -- see
    /// `crate::scaffolding::test_support::Sample_Context`.
    fn Context() -> FactContext
    {
        return scaffolding::test_support::Sample_Context();
    }
}
