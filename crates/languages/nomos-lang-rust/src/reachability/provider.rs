//! Turning a reachability reading into a fact the analysis kernel can store.
//!
//! The identical assembly [`crate::provider`] performs for the first capability, over the
//! second. [`crate::FactContext`] is reused rather than a second, structurally identical
//! type declared here — the four fields travel together for the same reason regardless of
//! which capability they are stamping.

use super::guarantee::Declared_Guarantee;
use super::{ReachabilityReading, Read_Reachability};
use crate::{FactContext, Materialization, PROVIDER};
use nomos_analysis::{FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_cap_controlflow::{Capability, CONTRACT_VERSION, Encode_Payload, Payload_Schema, ReachabilityPayload};
use nomos_contracts::{EvidenceClass, Guarantee, ProviderId, SubjectId};

/// What this offer computes a file's fact from — the file text and only the file text, the
/// identical claim `crate::provider::Syntax_Inputs` makes for the same reason: two copies
/// of one file reach one fact, and neither a path nor a modification time factors in.
#[must_use]
pub(crate) fn Reachability_Inputs(source: &str) -> InputDigest
{
    return InputDigest::Of(&[source.as_bytes()]);
}

/// Produces the reachability fact for one file.
#[must_use]
pub fn Materialize(subject: SubjectId, source: &str, context: FactContext) -> Materialization
{
    let sites = match Read_Reachability(source)
    {
        ReachabilityReading::Parsed(sites) => sites,
        ReachabilityReading::Unparseable(failure) => return Materialization::Unparseable(failure),
    };

    let payload = Encode_Payload(&ReachabilityPayload { sites });
    let guarantee = Declared_Guarantee();
    let key = Keyed(subject, source, guarantee, context);

    let fact = Fact(key, guarantee, payload, context);

    return Materialization::Materialized(Box::new(fact));
}

/// The fact itself, once its identity is settled — evidence `Verified`, the identical
/// argument `crate::provider::Fact` gives: a pattern either matched the token stream or it
/// did not, with no inference step between.
fn Fact(
    key: nomos_analysis::FactKey,
    guarantee: Guarantee,
    payload: Vec<u8>,
    context: FactContext,
) -> MaterializedFact
{
    return MaterializedFact {
        identity: key.At(context.generation),
        snapshot: context.snapshot,
        evidence: EvidenceClass::Verified,
        guarantee,
        payload: FactPayload::New(Payload_Schema(), payload),
    };
}

fn Keyed(
    subject: SubjectId,
    source: &str,
    guarantee: Guarantee,
    context: FactContext,
) -> nomos_analysis::FactKey
{
    return nomos_analysis::FactKey {
        contract: Capability(),
        contract_version: CONTRACT_VERSION,
        subject,
        semantic_inputs: Reachability_Inputs(source),
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
    use crate::Materialization;
    use nomos_contracts::{BuildVariantId, ConfigurationId, Digest128, GenerationId, SnapshotId};
    use nomos_model::Content_Digest;

    fn Subject(path: &str) -> SubjectId
    {
        return SubjectId::From_Digest(Content_Digest(path.as_bytes()));
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

    fn Fact(source: &str) -> MaterializedFact
    {
        return match Materialize(Subject("a.rs"), source, Context())
        {
            Materialization::Materialized(fact) => *fact,
            Materialization::Unparseable(failure) => panic!("expected a fact: {failure}"),
        };
    }

    #[test]
    fn Test_A_Fact_Should_Carry_The_Declared_Guarantee()
    {
        let fact = Fact("pub fn one() {}\n");

        assert_eq!(fact.guarantee, Declared_Guarantee());
        assert_eq!(fact.Key().guarantee, GuaranteeDigest::Of(&Declared_Guarantee()));
        assert_eq!(fact.payload.schema, Payload_Schema());
    }

    #[test]
    fn Test_An_Unparseable_File_Should_Produce_No_Fact()
    {
        let outcome = Materialize(Subject("broken.rs"), "fn unclosed( {", Context());

        assert!(
            matches!(outcome, Materialization::Unparseable(_)),
            "got {outcome:?}"
        );
    }

    /// This item's own real precedent, transcribed. A regression here is the exact defect
    /// `OD-RULES-008` names.
    #[test]
    fn Test_A_Real_Instance_Shaped_Arm_Should_Not_Be_Flagged()
    {
        let source = "fn Payload_Of() -> Result<u32, u32> {\n\
                       match facts.Require() {\n\
                       Ok(fact) => Ok(fact),\n\
                       Err(applicability) => return Err(Unread(source, applicability, \"why\")),\n\
                       }\n\
                       }\n";

        let fact = Fact(source);
        let rendered = String::from_utf8(fact.payload.bytes.clone()).expect("ASCII and tabs");

        assert_eq!(rendered, "", "a real, correctly written arm must flag nothing");
    }

    #[test]
    fn Test_An_Empty_Arm_Should_Be_Flagged()
    {
        let source = "fn Payload_Of() -> Result<u32, u32> {\n\
                       match facts.Require() {\n\
                       Ok(fact) => Ok(fact),\n\
                       Err(applicability) => {}\n\
                       }\n\
                       }\n";

        let fact = Fact(source);
        let rendered = String::from_utf8(fact.payload.bytes.clone()).expect("ASCII and tabs");

        assert_eq!(rendered, "site\tPayload_Of\tapplicability\tempty\n");
    }
}
