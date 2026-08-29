//! Turning a reading into a fact the analysis kernel can store.
//!
//! Everything identity-bearing about the result is assembled here — see `nomos-lang-rust`'s
//! own `provider.rs` for the full reasoning; this module makes the identical choices for a
//! Go file in place of a Rust one.

use crate::Materialization;
use crate::SyntaxFacts;
use crate::SyntaxItem;
use crate::{Declared_Guarantee, PROVIDER};
use nomos_analysis::{FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_cap_syntax::{CONTRACT_VERSION, Capability, Payload_Schema};
use nomos_contracts::{
    BuildVariantId, ConfigurationId, EvidenceClass, GenerationId, Guarantee, ProviderId, SnapshotId, SubjectId,
};

/// Where in the workspace's history a fact is being produced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactContext
{
    pub snapshot: SnapshotId,
    pub variant: BuildVariantId,
    pub configuration: ConfigurationId,
    pub generation: GenerationId,
}

/// What this provider computes a file's fact from — the file text and only the file text.
#[must_use]
pub(crate) fn Syntax_Inputs(source: &str) -> InputDigest
{
    return InputDigest::Of(&[source.as_bytes()]);
}

/// Produces the syntax-items fact for one file.
#[must_use]
pub fn Materialize(subject: SubjectId, source: &str, context: FactContext) -> Materialization
{
    use crate::Read_Source;
    use crate::Reading;

    let facts = match Read_Source(source)
    {
        Reading::Parsed(facts) => facts,
        Reading::Unparseable(failure) => return Materialization::Unparseable(failure),
    };

    let payload = Encode_Payload(&facts);
    let guarantee = Declared_Guarantee();
    let key = Keyed(subject, source, guarantee, context);

    let fact = Fact(key, guarantee, payload, context);

    return Materialization::Materialized(Box::new(fact));
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
        semantic_inputs: Syntax_Inputs(source),
        provider: ProviderId::New(PROVIDER),
        provider_version: CONTRACT_VERSION,
        guarantee: GuaranteeDigest::Of(&guarantee),
        variant: context.variant,
        configuration: context.configuration,
    };
}

/// The fact itself, once its identity is settled.
///
/// The evidence is `Verified`, not `Derived`: a parser either found the item in the tree or
/// it did not, and this provider's own [`Assurance::Sound`](nomos_contracts::Assurance::Sound)
/// claim on both axes is exactly the claim that there is no inference step in between.
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

/// The canonical byte encoding of a reading — hand-written here rather than through
/// `nomos_cap_syntax::Render_Payload`, deliberately: see that crate's own module doc for why
/// the writers stay with their providers while the reader stays shared.
#[must_use]
pub fn Encode_Payload(facts: &SyntaxFacts) -> Vec<u8>
{
    let mut encoded = String::new();

    encoded.push_str("unexpanded\t");
    encoded.push_str(&facts.unexpanded.to_string());
    encoded.push('\n');

    for item in &facts.items
    {
        Encode_Item(&mut encoded, item);
    }

    return encoded.into_bytes();
}

/// One item as a record: ordinal, kind, visibility, name, documentation and shape.
fn Encode_Item(encoded: &mut String, item: &SyntaxItem)
{
    encoded.push_str("item\t");
    encoded.push_str(&item.ordinal.to_string());
    encoded.push('\t');
    encoded.push_str(item.kind.Label());
    encoded.push('\t');
    encoded.push_str(item.visibility.Label());
    encoded.push('\t');
    encoded.push_str(&item.Qualified_Name());
    encoded.push('\t');
    encoded.push_str(&Observed(item.documentation.as_deref()));
    encoded.push('\t');
    encoded.push_str(&Observed(item.shape.as_deref()));
    encoded.push('\n');
}

/// One observed field, in the schema's spelling. `NotObserved` — `-` — is unreachable from
/// here for the same reason it is unreachable from `nomos-lang-rust`'s encoder: this provider
/// parses, so every field it does not write is an absence it looked for.
fn Observed(value: Option<&str>) -> String
{
    return match value
    {
        Some(seen) => format!("+{}", nomos_cap_syntax::Escape(seen)),
        None => ".".to_owned(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Materialization;
    use nomos_contracts::Digest128;
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
        return match Materialize(Subject("a.go"), source, Context())
        {
            Materialization::Materialized(fact) => *fact,
            Materialization::Unparseable(failure) => panic!("expected a fact: {failure}"),
        };
    }

    #[test]
    fn Test_A_Fact_Should_Carry_The_Declared_Guarantee()
    {
        let fact = Fact("package main\n\nfunc One() {}\n");

        assert_eq!(fact.guarantee, Declared_Guarantee());
        assert_eq!(fact.Key().guarantee, GuaranteeDigest::Of(&Declared_Guarantee()));
        assert_eq!(fact.payload.schema, Payload_Schema());
    }

    #[test]
    fn Test_The_Same_Bytes_Should_Reach_The_Same_Semantic_Inputs()
    {
        let source = "package main\n\ntype S struct{}\n\nfunc (s S) New() S { return s }\n";

        let first = Fact(source);
        let second = Fact(source);

        assert_eq!(first.Key().semantic_inputs, second.Key().semantic_inputs);
        assert_eq!(first.Key().Digest(), second.Key().Digest());
        assert_eq!(first.payload.Digest(), second.payload.Digest());
    }

    #[test]
    fn Test_Different_Bytes_Should_Reach_Different_Semantic_Inputs()
    {
        let first = Fact("package main\n\nfunc One() {}\n");
        let second = Fact("package main\n\nfunc Two() {}\n");

        assert_ne!(first.Key().semantic_inputs, second.Key().semantic_inputs);
        assert_ne!(first.payload.Digest(), second.payload.Digest());
    }

    #[test]
    fn Test_An_Unparseable_File_Should_Produce_No_Fact()
    {
        let outcome = Materialize(Subject("broken.go"), "func unclosed( {", Context());

        assert!(matches!(outcome, Materialization::Unparseable(_)), "got {outcome:?}");
    }

    /// The encoding is the fact's content address, so it must not vary with anything but the
    /// facts, and it must not be empty for a file that has items.
    #[test]
    fn Test_The_Encoding_Should_Be_Stable_And_Not_Empty()
    {
        let fact = Fact(
            "package main\n\n\
             func One() {}\n\n\
             type Inner struct{}\n\n\
             func (i Inner) Two() {}\n",
        );
        let rendered = String::from_utf8(fact.payload.bytes.clone()).expect("ASCII tabs around UTF-8 identifiers");

        assert_eq!(
            rendered,
            "unexpanded\t0\n\
             item\t0\tFunction\tPublic\tOne\t.\t+fn/0\n\
             item\t1\tStruct\tPublic\tInner\t.\t.\n\
             item\t2\tFunction\tPublic\tInner::Two\t.\t+fn/1\n"
        );
        assert!(!rendered.contains('\r'), "line endings must not be local");
    }

    /// What this provider writes for the two `Observation` fields, as bytes.
    #[test]
    fn Test_The_Encoding_Should_Carry_What_This_Provider_Observed()
    {
        let fact = Fact(
            "package main\n\n\
             // Tables lists every table.\n\
             // Mirrored by Test_Every_Row.\n\
             var Tables []string\n\n\
             const Limit = 2\n",
        );
        let rendered = String::from_utf8(fact.payload.bytes.clone()).expect("ASCII and tabs");

        assert_eq!(
            rendered,
            "unexpanded\t0\n\
             item\t0\tVariable\tPublic\tTables\t+Tables lists every table.\\nMirrored by Test_Every_Row.\t+slice\n\
             item\t1\tConstant\tPublic\tLimit\t.\t.\n"
        );
    }

    /// What this provider writes is what the schema says a payload is — decoded here through
    /// the shared reader rather than compared against `nomos-lang-rust`'s own bytes, for the
    /// reason `nomos_cap_syntax`'s module doc gives.
    #[test]
    fn Test_This_Providers_Payload_Should_Decode_Under_The_Schemas_Own_Reader()
    {
        let fact = Fact("package main\n\nfunc One() {}\n\ntype Inner struct{}\n\nfunc (i Inner) Two() {}\n");

        let payload = nomos_cap_syntax::Parse_Payload(&fact.payload.bytes).expect("this provider writes nomos.syntax.items.v2");

        assert_eq!(payload.unexpanded, 0);
        assert_eq!(payload.items.len(), 3);

        let third = payload.items.get(2).expect("three items");
        assert_eq!(third.ordinal, 2);
        assert_eq!(third.kind, nomos_cap_syntax::FUNCTION);
        assert_eq!(third.qualified_name, "Inner::Two");
        assert_eq!(third.Own_Name(), "Two");
        assert!(third.Is_Public());
    }

    /// The mark the schema reserves is the one this provider puts on the blank identifier —
    /// a different real-world case than `nomos-lang-rust`'s trait member, and the reason
    /// [`crate::Visibility::NotApplicable`] exists at all for this provider.
    #[test]
    fn Test_The_Blank_Identifier_Should_Carry_The_Mark_The_Schema_Reserves()
    {
        let fact = Fact("package main\n\ntype Writer interface{}\n\nvar _ Writer = nil\n\nvar Free int\n");

        let payload = nomos_cap_syntax::Parse_Payload(&fact.payload.bytes).expect("well formed");

        let blank = payload
            .items
            .iter()
            .find(|item| return item.Own_Name() == "_")
            .expect("the blank identifier is an item");
        assert!(blank.Declares_No_Visibility(), "{blank:?}");

        let free = payload
            .items
            .iter()
            .find(|item| return item.Own_Name() == "Free")
            .expect("the free variable is an item");
        assert!(free.Is_Public(), "{free:?}");
    }
}
