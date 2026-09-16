//! Turning `standards.json`'s own declared rows into the one fact this capability answers.

use super::guarantee::{Declared_Guarantee, PROVIDER};
use super::reading::{Discover_Workspace, NamingPolicyError};
use crate::scaffolding;
use nomos_cap_naming_policy::{Capability, Encode_Payload, NamingPolicyPayload, Payload_Schema, CONTRACT_VERSION};
use nomos_platform::FileSystem;
use std::path::Path;

pub use crate::scaffolding::{FactContext, PolicyFact};

/// Reads `root`'s own `standards.json` and produces the one fact this capability answers
/// for the workspace as a whole — a leaf: nothing here reads another fact this or any
/// other provider produced.
///
/// Deliberately not itself a `nomos_analysis::MemoryFactStore` writer, the same division
/// `nomos_lang_rust_deny::Materialize_Workspace` draws for the identical reason: a
/// composition root calls this and writes the returned [`PolicyFact`] into the store it
/// owns.
///
/// # Errors
///
/// Whatever [`Discover_Workspace`] returns.
pub fn Materialize_Workspace<Fs: FileSystem>(root: &Path, context: FactContext, filesystem: &Fs) -> Result<PolicyFact, NamingPolicyError>
{
    let payload = NamingPolicyPayload { rows: Discover_Workspace(root, filesystem)? };

    return Ok(scaffolding::Materialize_Whole_Workspace(scaffolding::WholeWorkspaceFiling {
        subject: scaffolding::Workspace_Subject(),
        guarantee: Declared_Guarantee(),
        identity: scaffolding::Identity(Capability(), CONTRACT_VERSION, PROVIDER, CONTRACT_VERSION),
        context,
        schema: Payload_Schema(),
        bytes: Encode_Payload(&payload),
    }));
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_platform_std::StdFileSystem;

    #[test]
    fn Test_Materialize_Workspace_Should_Materialize_This_Repositorys_Own_Declared_Convention()
    {
        let PolicyFact { subject, fact } = Materialize_Workspace(&scaffolding::test_support::Repository_Root(), Context(), &StdFileSystem)
            .expect("this repository's own standards.json is real and well-formed");

        let decoded = nomos_cap_naming_policy::Parse_Payload(&fact.payload.bytes).expect("this crate's own encoding");
        assert!(
            decoded.rows.iter().any(|row| return row.symbol == "function"),
            "this workspace's own standards.json declares naming.function: {decoded:?}"
        );

        scaffolding::test_support::Assert_Carries_Its_Declaration(&subject, &fact, Declared_Guarantee(), &Payload_Schema());
    }

    #[test]
    fn Test_A_Fact_Key_Should_Depend_On_The_Guarantee()
    {
        let identity = scaffolding::Identity(Capability(), CONTRACT_VERSION, PROVIDER, CONTRACT_VERSION);

        scaffolding::test_support::Assert_Keys_File_Apart_By_Guarantee(&identity, Declared_Guarantee());
    }

    /// This test's own fixture, shared with its five siblings -- see
    /// `crate::scaffolding::test_support::Sample_Context`.
    fn Context() -> FactContext
    {
        return scaffolding::test_support::Sample_Context();
    }
}
