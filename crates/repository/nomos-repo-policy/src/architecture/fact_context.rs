//! Turning `nomos-architecture.json`'s own declared architecture into the one fact this
//! capability answers.

use super::guarantee::{Declared_Guarantee, PROVIDER};
use super::reading::{ArchitectureError, Discover_Workspace};
use crate::scaffolding;
use nomos_cap_architecture::{Capability, Encode_Payload, Payload_Schema, CONTRACT_VERSION};
use nomos_platform::FileSystem;
use std::path::Path;

pub use crate::scaffolding::{FactContext, PolicyFact};

/// Reads `root`'s own `nomos-architecture.json` and produces the one fact this capability answers for
/// the workspace as a whole — a leaf: nothing here reads another fact this or any other
/// provider produced.
///
/// A repository declaring no architecture still produces a fact, and that is deliberate rather
/// than an oversight: "this repository has not declared an architecture" is an answer the rules
/// act on, and it is a different thing from the capability being unavailable. The empty payload
/// says the former; a missing fact would say the latter. `OD-RULES-029` measured what happens
/// when the two cannot be told apart — every member of every other repository read as a gap.
///
/// # Errors
///
/// Whatever [`Discover_Workspace`] returns.
pub fn Materialize_Workspace<Fs: FileSystem>(root: &Path, context: FactContext, filesystem: &Fs) -> Result<PolicyFact, ArchitectureError>
{
    let payload = Discover_Workspace(root, filesystem)?;

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
    use super::*;
    use nomos_platform::{DeterminismStrength, FileSystemError, ReproducibilityScope, Strategy, TraceEquivalence};

    /// A [`FileSystem`] that hands back fixed text instead of reading a real path.
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
    fn Test_Materialize_Workspace_Should_Materialize_A_Declared_Architecture()
    {
        let filesystem = FakeFileSystem {
            text: serde_json::json!({ "components": ["Domain"], "members": { "billing": "Domain" } }).to_string(),
        };

        let PolicyFact { subject, fact } = Materialize_Workspace(Path::new("."), Context(), &filesystem).expect("well-formed JSON");

        let decoded = nomos_cap_architecture::Parse_Payload(&fact.payload.bytes).expect("this crate's own encoding");
        assert_eq!(decoded.Component_Of("billing"), Some("Domain"));

        scaffolding::test_support::Assert_Carries_Its_Declaration(&subject, &fact, Declared_Guarantee(), &Payload_Schema());
    }

    /// A repository declaring nothing still gets a fact, carrying an empty declaration.
    #[test]
    fn Test_Materialize_Workspace_Should_Materialize_An_Empty_Declaration()
    {
        let filesystem = FakeFileSystem { text: serde_json::json!({}).to_string() };

        let PolicyFact { fact, .. } = Materialize_Workspace(Path::new("."), Context(), &filesystem).expect("well-formed JSON");

        let decoded = nomos_cap_architecture::Parse_Payload(&fact.payload.bytes).expect("this crate's own encoding");
        assert!(!decoded.Has_An_Architecture());
    }

    #[test]
    fn Test_A_Fact_Key_Should_Depend_On_The_Guarantee()
    {
        let identity = scaffolding::Declared_Identity(Capability(), CONTRACT_VERSION, PROVIDER, CONTRACT_VERSION);

        scaffolding::test_support::Assert_Keys_File_Apart_By_Guarantee(&identity, Declared_Guarantee());
    }

    /// This test's own fixture, shared with its five siblings — see
    /// `crate::scaffolding::test_support::Sample_Context`.
    fn Context() -> FactContext
    {
        return scaffolding::test_support::Sample_Context();
    }
}
