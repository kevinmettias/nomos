//! Turning `standards.json`'s own declared goal policy into the one fact this capability
//! answers.

use super::guarantee::{Declared_Guarantee, PROVIDER};
use super::reading::{Discover_Workspace, GoalsPolicyError};
use crate::scaffolding;
use nomos_cap_goals_policy::{Capability, Encode_Payload, Payload_Schema, CONTRACT_VERSION};
use nomos_platform::FileSystem;
use std::path::Path;

pub use crate::scaffolding::{FactContext, PolicyFact};

/// Reads `root`'s own `standards.json` and produces the one fact this capability answers for
/// the workspace as a whole — a leaf: nothing here reads another fact this or any other
/// provider produced.
///
/// A repository declaring no goals still produces a fact, and that is deliberate rather than
/// an oversight: "this repository has not taken goal traceability on" is an answer a rule
/// can act on, and it is a different thing from the capability being unavailable. The empty
/// payload says the former; a missing fact would say the latter.
///
/// # Errors
///
/// Whatever [`Discover_Workspace`] returns.
pub fn Materialize_Workspace<Fs: FileSystem>(root: &Path, context: FactContext, filesystem: &Fs) -> Result<PolicyFact, GoalsPolicyError>
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
    use nomos_platform_std::StdFileSystem;

    #[test]
    fn Test_Materialize_Workspace_Should_Materialize_This_Repositorys_Own_Declared_Policy()
    {
        let PolicyFact { subject, fact } = Materialize_Workspace(&scaffolding::test_support::Repository_Root(), Context(), &StdFileSystem)
            .expect("this repository's own standards.json is real and well-formed");

        let decoded = nomos_cap_goals_policy::Parse_Payload(&fact.payload.bytes).expect("this crate's own encoding");
        assert_eq!(
            decoded,
            nomos_cap_goals_policy::GoalsPolicyPayload::default(),
            "an undeclared repository still produces a fact, and the fact says it declared nothing"
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
