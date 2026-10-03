//! Turning a sites reading into a fact the analysis kernel can store.
//!
//! The assembly `crate::provider` performs for the items fact, over the sites family.
//! [`crate::FactContext`] is reused for the reason `crate::reachability` reuses it.

use super::guarantee::Declared_Guarantee;
use super::{Read_Sites, SitesReading};
use crate::{FactContext, Materialization, PROVIDER};
use nomos_analysis::{FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_cap_syntax::{LABELED_JUMP, LabeledJump, Render_Sites_Payload, SITES_CONTRACT_VERSION, Sites_Capability, Sites_Payload_Schema, SitesPayload};
use nomos_contracts::{EvidenceClass, ProviderId, SubjectId};

/// Produces the sites fact for one file: the kinds this provider offers, and every record of them.
#[must_use]
pub fn Materialize_Sites_Fact(subject: SubjectId, source: &str, context: FactContext) -> Materialization
{
    let jumps = match Read_Sites(source)
    {
        SitesReading::Parsed(jumps) => jumps,
        SitesReading::Unparseable(failure) => return Materialization::Unparseable(failure),
    };

    let payload = SitesPayload {
        offered: vec![LABELED_JUMP.name.to_owned()],
        declined: Vec::new(),
        records: jumps.iter().map(LabeledJump::Record).collect(),
    };
    let guarantee = Declared_Guarantee();
    let key = FactKey {
        contract: Sites_Capability(),
        contract_version: SITES_CONTRACT_VERSION,
        subject,
        // The file text and only the file text, the claim `crate::provider` makes for the items
        // fact: two copies of one file reach one fact.
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
        // `Verified`, the argument `crate::provider` gives: a jump either names a label written
        // around it in the token stream or it does not, with no inference between.
        evidence: EvidenceClass::Verified,
        guarantee,
        payload: FactPayload::New(Sites_Payload_Schema(), Render_Sites_Payload(&payload)),
    }));
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_cap_syntax::{KindStance, Parse_Sites_Payload};
    use nomos_contracts::{BuildVariantId, ConfigurationId, Digest128, GenerationId, SnapshotId};
    use nomos_model::Content_Digest;

    /// The byte this fixture's snapshot identity seeds, unique among the three.
    const SNAPSHOT_SEED: u8 = 1;
    /// The byte this fixture's build variant identity seeds.
    const BUILD_VARIANT_SEED: u8 = 2;
    /// The byte this fixture's configuration identity seeds.
    const CONFIGURATION_SEED: u8 = 3;

    fn Context() -> FactContext
    {
        return FactContext {
            snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([SNAPSHOT_SEED; Digest128::BYTE_LENGTH])),
            variant: BuildVariantId::From_Digest(Digest128::From_Bytes([BUILD_VARIANT_SEED; Digest128::BYTE_LENGTH])),
            configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([CONFIGURATION_SEED; Digest128::BYTE_LENGTH])),
            generation: GenerationId::INITIAL,
        };
    }

    fn Fact_Of(source: &str) -> MaterializedFact
    {
        return match Materialize_Sites_Fact(SubjectId::From_Digest(Content_Digest(b"a.rs")), source, Context())
        {
            Materialization::Materialized(fact) => *fact,
            Materialization::Unparseable(failure) => panic!("the fixture is Rust written to parse: {failure}"),
        };
    }

    /// The provider's own output, decoded through the one reader every consumer uses -- the check
    /// a provider owes the grammar it writes.
    #[test]
    fn Test_Materialize_Sites_Fact_Should_Write_A_Payload_The_Reader_Accepts()
    {
        let fact = Fact_Of("fn f(rows: &[u8]) { 'each: for _ in rows { continue 'each; } }\n");

        let payload = Parse_Sites_Payload(&fact.payload.bytes).expect("this provider writes the family's schema");

        assert_eq!(fact.payload.schema, Sites_Payload_Schema());
        assert_eq!(fact.guarantee, Declared_Guarantee());
        assert_eq!(payload.Stance(LABELED_JUMP.name), KindStance::Offered);
        assert_eq!(LabeledJump::All_In(&payload).map(|jumps| return jumps.len()), Some(1));
    }

    /// A file with no labeled jump still offers the kind, which is what makes its silence a clean
    /// answer rather than a gap.
    #[test]
    fn Test_A_File_With_No_Jumps_Should_Still_Offer_The_Kind()
    {
        let fact = Fact_Of("pub fn One() {}\n");

        let payload = Parse_Sites_Payload(&fact.payload.bytes).expect("this provider writes the family's schema");

        assert_eq!(payload.Stance(LABELED_JUMP.name), KindStance::Offered);
        assert!(payload.records.is_empty());
    }

    #[test]
    fn Test_An_Unparseable_File_Should_Produce_No_Fact()
    {
        let outcome = Materialize_Sites_Fact(SubjectId::From_Digest(Content_Digest(b"broken.rs")), "fn f( {", Context());

        assert!(matches!(outcome, Materialization::Unparseable(_)), "{outcome:?}");
    }
}
