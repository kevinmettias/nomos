//! Turning one file's reading into a fact the analysis kernel can store.

use crate::{ConditionalReading, Declared_Guarantee, DefinitionSet, FactContext, Materialization, PROVIDER, Read_Conditionals};
use nomos_analysis::{FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_cap_csharp_semantics::{BuildSelection, Capability, ConditionalPayload, CONTRACT_VERSION, Encode_Payload, Payload_Schema};
use nomos_contracts::{EvidenceClass, ProviderId, SubjectId};
use nomos_model::Digest_Of_Parts;

/// Produces the conditional-compilation fact for one file under one build.
///
/// `subject` is the file's own subject. The fact is filed under [`Build_Subject`] of it and the
/// build, never under the file's subject itself: one file under two builds is two answers, and a
/// store holding both needs them apart whatever subject the caller passed -- so the separation is
/// made here, where the key is, and not left to every caller to remember.
///
/// The semantic inputs are empty, which is what every provider in this workspace whose answer
/// turns on a subprocess's own analysis files: the text is one input and the definition set
/// `MSBuild` evaluated is the other, and no reader holds that set independently to rebuild a key
/// from. A key a reader cannot rebuild is a fact no rule can ask for. A change to either input
/// that changes the answer changes the payload, which is what a caller's currency check compares,
/// and one that does not leaves the fact as current as it was. The evidence is
/// `Verified`: each state follows from the definition set and the directives by the
/// specification's rules, with no inference between.
#[must_use]
pub fn Materialize_Conditional_Fact(subject: SubjectId, source: &str, definitions: &DefinitionSet, context: FactContext) -> Materialization
{
    let (file_definitions, regions) = match Read_Conditionals(source, &definitions.symbols)
    {
        ConditionalReading::Read { definitions, regions } => (definitions, regions),
        ConditionalReading::Refused(failure) => return Materialization::Refused(failure),
    };

    let payload = Encode_Payload(&ConditionalPayload {
        selection: definitions.selection.clone(),
        symbols: definitions.symbols.iter().cloned().collect(),
        definitions: file_definitions,
        regions,
    });
    let guarantee = Declared_Guarantee();
    let key = FactKey {
        contract: Capability(),
        contract_version: CONTRACT_VERSION,
        subject: Build_Subject(subject, &definitions.selection),
        semantic_inputs: InputDigest::Of(&[]),
        provider: ProviderId::New(PROVIDER),
        provider_version: CONTRACT_VERSION,
        guarantee: GuaranteeDigest::Of(&guarantee),
        variant: context.variant,
        configuration: context.configuration,
    };

    return Materialization::Materialized(Box::new(MaterializedFact {
        identity: key.At(context.generation),
        snapshot: context.snapshot,
        evidence: EvidenceClass::Verified,
        guarantee,
        payload: FactPayload::New(Payload_Schema(), payload),
    }));
}

/// The subject one file's answer under one build is filed under: a digest of the file's own
/// subject and the build's project, configuration and target framework, so two builds never share
/// one.
///
/// The build's symbols are not part of it. They are an input to the answer, not a part of what the
/// answer is about: a build whose `DefineConstants` moved is the same build answering differently,
/// and its new fact replaces the old one under the same subject rather than standing beside it.
#[must_use]
pub fn Build_Subject(file: SubjectId, selection: &BuildSelection) -> SubjectId
{
    let file = file.Digest();
    let parts: [&[u8]; 4] = [file.Bytes(), selection.project.as_bytes(), selection.configuration.as_bytes(), selection.target_framework.as_bytes()];
    return SubjectId::From_Digest(Digest_Of_Parts(&parts));
}
