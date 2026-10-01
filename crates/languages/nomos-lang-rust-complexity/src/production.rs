//! Turning one file's reading into a fact the analysis kernel can store.

use crate::{ComplexityReading, Declared_Guarantee, FactContext, Materialization, PROVIDER, Read_Complexity};
use nomos_analysis::{FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_cap_complexity::{Capability, ComplexityPayload, Complexity_Descriptor, CONTRACT_VERSION, Encode_Payload, Payload_Schema};
use nomos_contracts::{EvidenceClass, ProviderId, SubjectId};

/// Produces the complexity fact for one file: `subject` names it and `source` is all of it.
///
/// The fact's semantic input is the text and only the text, the claim every per-file provider
/// here makes: two copies of one file are one computation, and a modification time would make an
/// untouched file look changed after a checkout. The evidence is `Verified` -- a parser either
/// found a branch in the token stream or it did not.
#[must_use]
pub fn Materialize_Complexity_Fact(subject: SubjectId, source: &str, context: FactContext) -> Materialization
{
    let functions = match Read_Complexity(source)
    {
        ComplexityReading::Parsed(functions) => functions,
        ComplexityReading::Unparseable(failure) => return Materialization::Unparseable(failure),
    };

    let payload = Encode_Payload(&ComplexityPayload { descriptor: Complexity_Descriptor(), functions });
    let guarantee = Declared_Guarantee();
    let key = FactKey {
        contract: Capability(),
        contract_version: CONTRACT_VERSION,
        subject,
        semantic_inputs: InputDigest::Of(&[source.as_bytes()]),
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
