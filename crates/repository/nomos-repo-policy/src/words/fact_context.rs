//! Turning `standards.json`'s own declared vocabulary additions into the one fact this
//! capability answers.

use super::guarantee::{Declared_Guarantee, PROVIDER};
use super::reading::{Discover_Workspace, WordsPolicyError};
use crate::scaffolding;
use nomos_cap_words_policy::{Capability, Encode_Payload, Payload_Schema, CONTRACT_VERSION};
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
pub fn Materialize_Workspace<Fs: FileSystem>(root: &Path, context: FactContext, filesystem: &Fs) -> Result<PolicyFact, WordsPolicyError>
{
    let payload = Discover_Workspace(root, filesystem)?;

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
    fn Test_Materialize_Workspace_Should_Materialize_This_Repositorys_Own_Declared_Additions()
    {
        let PolicyFact { subject, fact } = Materialize_Workspace(&scaffolding::test_support::Repository_Root(), Context(), &StdFileSystem)
            .expect("this repository's own standards.json is real and well-formed");

        let decoded = nomos_cap_words_policy::Parse_Payload(&fact.payload.bytes).expect("this crate's own encoding");
        assert!(
            decoded.approved_additions.contains(&"kwb".to_owned()),
            "this workspace's own standards.json declares words.approved_abbreviations to include kwb: {decoded:?}"
        );
        assert!(
            decoded.approved_additions.contains(&"vs".to_owned()),
            "OD-RULES-017 added vs to words.approved_abbreviations as missing from the ported \
             vocabulary rather than banned by it: {decoded:?}"
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
