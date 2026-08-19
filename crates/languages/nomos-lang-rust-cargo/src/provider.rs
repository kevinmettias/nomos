//! Turning a discovered workspace into the facts this capability answers.

use crate::guarantee::{Declared_Guarantee, PROVIDER};
use crate::metadata::{Discover_Workspace, MetadataError};
use nomos_cap_dependency::{Capability, Encode_Payload, Payload_Schema, CONTRACT_VERSION};
use nomos_analysis::{FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_contracts::{
    BuildVariantId, ConfigurationId, EvidenceClass, GenerationId, Guarantee, ProviderId, SnapshotId,
    SubjectId,
};
use std::path::Path;

/// Where in the workspace's history a fact is being produced.
///
/// The same shape `nomos_lang_rust::FactContext` carries, for the same reason: these four
/// always travel together, and a call site that transposed two of them would compile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactContext
{
    pub snapshot: SnapshotId,
    pub variant: BuildVariantId,
    pub configuration: ConfigurationId,
    pub generation: GenerationId,
}

/// One package's fact, together with the subject it was filed under — a caller needs the
/// subject to key its own read of the same fact back out of a store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageFact
{
    pub subject: SubjectId,
    pub fact: MaterializedFact,
}

/// Runs `cargo metadata` over `root` and produces one fact per workspace member, each a
/// leaf: nothing here reads another fact this or any other provider produced, so every
/// call to `store.Materialize` a caller makes from this function's output should pass no
/// dependency edges, the same shape `nomos_lang_rust::Materialize`'s own doc comment
/// states for the identical reason.
///
/// Deliberately not itself a `nomos_analysis::MemoryFactStore` writer: this crate is not
/// yet composed into `nomos-check-orchestration`'s `Run`, which is the only place today
/// that owns writing to a store, and a function here that took `&mut MemoryFactStore`
/// would be inventing a second composition root's worth of responsibility this crate does
/// not have a caller for yet.
///
/// # Errors
///
/// Whatever [`Discover_Workspace`] returns.
pub fn Materialize_Workspace(root: &Path, context: FactContext) -> Result<Vec<PackageFact>, MetadataError>
{
    let discovered = Discover_Workspace(root)?;

    return Ok(discovered
        .into_iter()
        .map(|package| {
            let subject = nomos_model::Subject_Of_Path(&package.manifest_relative_root);
            let guarantee = Declared_Guarantee();
            let payload_bytes = Encode_Payload(&package.payload);
            let key = Keyed(subject, &payload_bytes, guarantee, context);
            let fact = MaterializedFact {
                identity: key.At(context.generation),
                snapshot: context.snapshot,
                evidence: EvidenceClass::Verified,
                guarantee,
                payload: FactPayload::New(Payload_Schema(), payload_bytes),
            };

            return PackageFact { subject, fact };
        })
        .collect());
}

fn Keyed(subject: SubjectId, payload_bytes: &[u8], guarantee: Guarantee, context: FactContext) -> FactKey
{
    return FactKey {
        contract: Capability(),
        contract_version: CONTRACT_VERSION,
        subject,
        semantic_inputs: InputDigest::Of(&[payload_bytes]),
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

    fn Context() -> FactContext
    {
        return FactContext {
            snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; 16])),
            variant: BuildVariantId::From_Digest(Digest128::From_Bytes([2; 16])),
            configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([3; 16])),
            generation: GenerationId::INITIAL,
        };
    }

    #[test]
    fn Test_Every_Workspace_Member_Should_Produce_One_Fact()
    {
        let facts = Materialize_Workspace(&Repository_Root(), Context()).expect("a real workspace");

        assert!(
            facts.iter().any(|fact| fact.fact.payload.bytes.starts_with(b"package\tnomos-rules\n")),
            "expected a fact for nomos-rules among {} facts",
            facts.len()
        );
    }

    #[test]
    fn Test_A_Facts_Subject_Should_Match_Subject_Of_Its_Own_Path()
    {
        let facts = Materialize_Workspace(&Repository_Root(), Context()).expect("a real workspace");

        let rules = facts
            .iter()
            .find(|fact| fact.fact.payload.bytes.starts_with(b"package\tnomos-rules\n"))
            .expect("nomos-rules is a workspace member");

        assert_eq!(
            rules.subject,
            nomos_model::Subject_Of_Path("crates/rules/nomos-rules")
        );
    }

    #[test]
    fn Test_A_Fact_Should_Carry_The_Declared_Guarantee()
    {
        let facts = Materialize_Workspace(&Repository_Root(), Context()).expect("a real workspace");

        for fact in &facts
        {
            assert_eq!(fact.fact.guarantee, Declared_Guarantee());
            assert_eq!(fact.fact.Key().guarantee, GuaranteeDigest::Of(&Declared_Guarantee()));
            assert_eq!(fact.fact.payload.schema, Payload_Schema());
        }
    }

    #[test]
    fn Test_Two_Runs_Over_The_Same_Tree_Should_Reach_The_Same_Semantic_Inputs()
    {
        let first = Materialize_Workspace(&Repository_Root(), Context()).expect("a real workspace");
        let second = Materialize_Workspace(&Repository_Root(), Context()).expect("a real workspace");

        assert_eq!(first.len(), second.len());
        for (left, right) in first.iter().zip(second.iter())
        {
            assert_eq!(left.fact.Key().semantic_inputs, right.fact.Key().semantic_inputs);
            assert_eq!(left.fact.Key().Digest(), right.fact.Key().Digest());
        }
    }
}
