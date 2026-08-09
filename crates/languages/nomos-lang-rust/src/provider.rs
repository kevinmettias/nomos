//! Turning a reading into a fact the analysis kernel can store.
//!
//! Everything identity-bearing about the result is assembled here: which capability
//! answered, which provider, under which guarantee, over which bytes, in which
//! generation. The kernel derives invalidation from those components, so a fact that got
//! any of them wrong would be invalidated at the wrong time rather than merely labelled
//! badly.

use crate::guarantee::{Declared_Guarantee, PROVIDER};
use crate::syntax::{ParseFailure, Reading, Read_Source, SyntaxFacts};
use nomos_analysis::{FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_cap_syntax::{Capability, Payload_Schema, CONTRACT_VERSION};
use nomos_contracts::{
    BuildVariantId, ConfigurationId, EvidenceClass, GenerationId, ProviderId, SnapshotId,
    SubjectId,
};

/// Where in the workspace's history a fact is being produced.
///
/// Carried as one value rather than five parameters because these five always travel
/// together and a call site that transposed two of them would compile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactContext
{
    pub snapshot: SnapshotId,
    pub variant: BuildVariantId,
    pub configuration: ConfigurationId,
    pub generation: GenerationId,
}

/// The outcome of asking this provider for a fact about one file.
///
/// Mirrors [`Reading`] deliberately. A `Result<MaterializedFact, ParseFailure>` would
/// invite `.ok()`, and a corpus walk that maps failures to `None` and counts the `Some`s
/// is exactly the shape of a run reporting a clean corpus it never read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Materialization
{
    Materialized(Box<MaterializedFact>),
    Unparseable(ParseFailure),
}

/// Produces the syntax-items fact for one file.
///
/// `subject` identifies the file; `source` is its entire contents. Nothing else is read,
/// which is what makes [`nomos_contracts::IncrementalGranularity::File`] true rather than
/// asserted.
#[must_use]
pub fn Materialize(subject: SubjectId, source: &str, context: FactContext) -> Materialization
{
    let facts = match Read_Source(source)
    {
        Reading::Parsed(facts) => facts,
        Reading::Unparseable(failure) => return Materialization::Unparseable(failure),
    };

    let payload = Encode_Payload(&facts);
    let guarantee = Declared_Guarantee();

    // The semantic input is the file text and only the file text. Not its path — two
    // copies of one file are one computation and must reach one fact — and not its
    // modification time, which would make an untouched file look changed after a
    // checkout.
    let key = nomos_analysis::FactKey {
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
        // Provenance, not identity. The tree this file was read from, recorded beside the
        // fact rather than folded into what it is — see OD-ANALYSIS-001.
        snapshot: context.snapshot,
        // A parser either found the item in the token stream or it did not; there is no
        // inference step by which this could report something the text does not contain.
        // Not `Derived`, which is for conclusions drawn from other facts — the source is
        // not a fact, it is the territory.
        evidence: EvidenceClass::Verified,
        guarantee,
        payload: FactPayload::New(Payload_Schema(), payload),
    }));
}

/// The canonical byte encoding of a reading.
///
/// Line-oriented text rather than a serialization format, for two reasons. It is
/// diffable, so two facts that disagree can be compared by a person rather than by a
/// tool. And it is written here in one place with no derive between the data and the
/// bytes, so the digest cannot change because a dependency changed how it orders fields —
/// which would invalidate every fact in the store for no reason anybody could see.
///
/// Fields are tab-separated and records are `\n`-terminated, never `\r\n`. A digest that
/// depends on the line ending of the machine that produced it is not a content address.
#[must_use]
pub fn Encode_Payload(facts: &SyntaxFacts) -> Vec<u8>
{
    let mut encoded = String::new();

    encoded.push_str("unexpanded\t");
    encoded.push_str(&facts.unexpanded.to_string());
    encoded.push('\n');

    for item in &facts.items
    {
        encoded.push_str("item\t");
        encoded.push_str(&item.ordinal.to_string());
        encoded.push('\t');
        encoded.push_str(item.kind.Label());
        encoded.push('\t');
        encoded.push_str(&item.visibility.Label());
        encoded.push('\t');
        encoded.push_str(&item.Qualified_Name());
        encoded.push('\n');
    }

    return encoded.into_bytes();
}

#[cfg(test)]
mod tests
{
    use super::*;
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

    /// The identity claim behind `IncrementalGranularity::File`: the same bytes are the
    /// same computation, so a file that did not change does not recompute.
    #[test]
    fn Test_The_Same_Bytes_Should_Reach_The_Same_Semantic_Inputs()
    {
        let source = "pub struct S;\nimpl S { pub fn new() -> Self { Self } }\n";

        let first = Fact(source);
        let second = Fact(source);

        assert_eq!(first.Key().semantic_inputs, second.Key().semantic_inputs);
        assert_eq!(first.Key().Digest(), second.Key().Digest());
        assert_eq!(first.payload.Digest(), second.payload.Digest());
    }

    /// The negative control for the test above. If the input digest ignored the source,
    /// every file in the corpus would share one fact key and the store would hold one
    /// entry claiming to describe all of them.
    #[test]
    fn Test_Different_Bytes_Should_Reach_Different_Semantic_Inputs()
    {
        let first = Fact("pub fn one() {}\n");
        let second = Fact("pub fn two() {}\n");

        assert_ne!(first.Key().semantic_inputs, second.Key().semantic_inputs);
        assert_ne!(first.payload.Digest(), second.payload.Digest());
    }

    /// Two subjects with identical contents are two facts, because a caller asks about a
    /// file rather than about a digest. They share the input digest — the computation was
    /// the same — and differ in subject, which is what lets a change to one of them
    /// invalidate one of them.
    #[test]
    fn Test_Two_Subjects_With_One_Content_Should_Be_Two_Facts_Over_One_Computation()
    {
        let source = "pub fn shared() {}\n";
        let context = Context();

        let (Materialization::Materialized(left), Materialization::Materialized(right)) = (
            Materialize(Subject("a.rs"), source, context),
            Materialize(Subject("b.rs"), source, context),
        )
        else
        {
            panic!("both subjects parse");
        };

        assert_eq!(left.Key().semantic_inputs, right.Key().semantic_inputs);
        assert_eq!(left.payload.Digest(), right.payload.Digest());
        assert_ne!(left.Key().Digest(), right.Key().Digest());
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

    /// The encoding is the fact's content address, so it must not vary with anything but
    /// the facts — and it must not be empty for a file that has items, or the digest
    /// would be the digest of nothing.
    #[test]
    fn Test_The_Encoding_Should_Be_Stable_And_Not_Empty()
    {
        let fact = Fact("pub fn one() {}\nmod inner { fn two() {} }\n");
        let rendered = String::from_utf8(fact.payload.bytes.clone())
            .expect("the encoding is ASCII tabs around UTF-8 identifiers");

        assert_eq!(
            rendered,
            "unexpanded\t0\n\
             item\t0\tFunction\tPublic\tone\n\
             item\t1\tModule\tPrivate\tinner\n\
             item\t2\tFunction\tPrivate\tinner::two\n"
        );
        assert!(!rendered.contains('\r'), "line endings must not be local");
    }
}
