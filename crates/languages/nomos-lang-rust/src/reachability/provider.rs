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
pub fn Materialize_Reachability_Fact(subject: SubjectId, source: &str, context: FactContext) -> Materialization
{
    let sites = match Read_Reachability(source)
    {
        ReachabilityReading::Parsed(sites) => sites,
        ReachabilityReading::Unparseable(failure) => return Materialization::Unparseable(failure),
    };

    let payload = Encode_Payload(&ReachabilityPayload { sites });
    let guarantee = Declared_Guarantee();
    let key = Compute_Fact_Key(subject, source, guarantee, context);

    let fact = Assembled_Fact(key, guarantee, payload, context);

    return Materialization::Materialized(Box::new(fact));
}

fn Compute_Fact_Key(
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

/// The fact itself, once its identity is settled — evidence `Verified`, the identical
/// argument `crate::provider::Assembled_Fact` gives: a pattern either matched the token
/// stream or it did not, with no inference step between.
fn Assembled_Fact(
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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Materialization;
    use nomos_contracts::{BuildVariantId, ConfigurationId, Digest128, GenerationId, SnapshotId};
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Materialize_Reachability_Fact_Should_Carry_The_Declared_Guarantee()
    {
        let fact = Fact_From_Source("pub fn one() {}\n");

        assert_eq!(fact.guarantee, Declared_Guarantee());
        assert_eq!(fact.Key().guarantee, GuaranteeDigest::Of(&Declared_Guarantee()));
        assert_eq!(fact.payload.schema, Payload_Schema());
    }

    #[test]
    fn Test_Read_Reachability_Should_Produce_No_Fact_For_An_Unparseable_File()
    {
        let outcome = Materialize_Reachability_Fact(Subject_Of_Path("broken.rs"), "fn unclosed( {", Context());

        assert!(
            matches!(outcome, Materialization::Unparseable(_)),
            "got {outcome:?}"
        );
    }

    /// This item's own real precedent, transcribed. A regression here is the exact defect
    /// `OD-RULES-008` names.
    #[test]
    fn Test_Reachability_Inputs_Should_Feed_A_Fact_Over_A_Real_Instance_Shaped_Arm()
    {
        let source = "fn Payload_Of() -> Result<u32, u32> {\n\
                       match facts.Require() {\n\
                       Ok(fact) => Ok(fact),\n\
                       Err(applicability) => return Err(Unread(source, applicability, \"why\")),\n\
                       }\n\
                       }\n";

        let fact = Fact_From_Source(source);
        let rendered = String::from_utf8(fact.payload.bytes.clone()).expect("ASCII and tabs");

        assert_eq!(rendered, "", "a real, correctly written arm must flag nothing");
    }

    #[test]
    fn Test_New_Should_Produce_A_Walk_That_Flags_An_Empty_Arm()
    {
        let source = "fn Payload_Of() -> Result<u32, u32> {\n\
                       match facts.Require() {\n\
                       Ok(fact) => Ok(fact),\n\
                       Err(applicability) => {}\n\
                       }\n\
                       }\n";

        let fact = Fact_From_Source(source);
        let rendered = String::from_utf8(fact.payload.bytes.clone()).expect("ASCII and tabs");

        assert_eq!(rendered, "site\tPayload_Of\tapplicability\tempty\n");
    }

    fn Subject_Of_Path(path: &str) -> SubjectId
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

    fn Fact_From_Source(source: &str) -> MaterializedFact
    {
        return match Materialize_Reachability_Fact(Subject_Of_Path("a.rs"), source, Context())
        {
            Materialization::Materialized(fact) => *fact,
            // Every source passed to this helper is Rust its author wrote to be parseable,
            // so a refusal is a broken fixture and not a reading worth handing back to the
            // tests below, which compare a rendered fact against an expected string and
            // would need something to compare in the first place.
            Materialization::Unparseable(failure) => panic!("expected a fact: {failure}"),
        };
    }
}
