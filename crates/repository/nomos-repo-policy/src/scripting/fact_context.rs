//! Turning `standards.json`'s own declared scripting policy into the one fact this
//! capability answers.

use super::guarantee::{Declared_Guarantee, PROVIDER};
use super::reading::{Discover_Workspace, ScriptingPolicyError};
use crate::scaffolding;
use nomos_cap_scripting_policy::{Capability, Encode_Payload, Payload_Schema, CONTRACT_VERSION};
use nomos_platform::FileSystem;
use std::path::Path;

pub use crate::scaffolding::{FactContext, PolicyFact};

/// Reads `root`'s own `standards.json` and produces the one fact this capability answers
/// for the workspace as a whole — a leaf: nothing here reads another fact this or any
/// other provider produced.
///
/// # Errors
///
/// Whatever [`Discover_Workspace`] returns.
pub fn Materialize_Workspace<Fs: FileSystem>(root: &Path, context: FactContext, filesystem: &Fs) -> Result<PolicyFact, ScriptingPolicyError>
{
    let payload = Discover_Workspace(root, filesystem)?;
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
    use super::*;
    use nomos_analysis::GuaranteeDigest;
    use nomos_platform_std::StdFileSystem;

    #[test]
    fn Test_Materialize_Workspace_Should_Materialize_This_Repositorys_Own_Declared_Policy()
    {
        let PolicyFact { subject, fact } = Materialize_Workspace(&Repository_Root(), Context(), &StdFileSystem)
            .expect("this repository's own standards.json is real and well-formed");

        assert_eq!(subject, nomos_model::Subject_Of_Path(""));
        assert_eq!(fact.guarantee, Declared_Guarantee());
        assert_eq!(fact.Key().guarantee, GuaranteeDigest::Of(&Declared_Guarantee()));
        assert_eq!(fact.payload.schema, Payload_Schema());

        let decoded = nomos_cap_scripting_policy::Parse_Payload(&fact.payload.bytes).expect("this crate's own encoding");
        assert_eq!(decoded.tooling_language, Some("rust".to_owned()));
    }

    fn Repository_Root() -> std::path::PathBuf
    {
        let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        return manifest
            .parent()
            .and_then(std::path::Path::parent)
            .and_then(std::path::Path::parent)
            .map(std::path::PathBuf::from)
            .expect("this crate sits three levels below the workspace root");
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
