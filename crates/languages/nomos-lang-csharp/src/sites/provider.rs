//! Turning C#'s stance on each declared kind into a fact the analysis kernel can store.

use super::LABELED_JUMP_DECLINE;
use super::guarantee::Declared_Guarantee;
use crate::{FactContext, Materialization, PROVIDER};
use nomos_analysis::{FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_cap_syntax::{KindDecline, LABELED_JUMP, Render_Sites_Payload, SITES_CONTRACT_VERSION, Sites_Capability, Sites_Payload_Schema, SitesPayload};
use nomos_contracts::{EvidenceClass, ProviderId, SubjectId};

/// Produces the sites fact for one C# file: no kind offered, the labeled jump declined with its
/// reason, and no record.
///
/// Always materialized: the decline is about the language, so it does not wait on a parse.
#[must_use]
pub fn Materialize_Sites_Fact(subject: SubjectId, source: &str, context: FactContext) -> Materialization
{
    let payload = SitesPayload {
        offered: Vec::new(),
        declined: vec![KindDecline { kind: LABELED_JUMP.name.to_owned(), reason: LABELED_JUMP_DECLINE.to_owned() }],
        records: Vec::new(),
    };
    let guarantee = Declared_Guarantee();
    let key = FactKey {
        contract: Sites_Capability(),
        contract_version: SITES_CONTRACT_VERSION,
        subject,
        // The file text, as every fact this provider files is keyed: two copies of one file reach one
        // fact, though this one's payload does not vary with the text.
        semantic_inputs: InputDigest::Of(&[source.as_bytes()]),
        provider: ProviderId::New(PROVIDER),
        provider_version: SITES_CONTRACT_VERSION,
        guarantee: GuaranteeDigest::Of(&guarantee),
        variant: context.variant,
        configuration: context.configuration,
    };

    return Materialization::Materialized(Box::new(MaterializedFact {
        identity: key.At(context.generation),
        snapshot: context.snapshot,
        evidence: EvidenceClass::Verified,
        guarantee,
        payload: FactPayload::New(Sites_Payload_Schema(), Render_Sites_Payload(&payload)),
    }));
}
