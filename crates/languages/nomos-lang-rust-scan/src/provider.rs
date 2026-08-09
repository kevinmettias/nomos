//! Turning a scan into a fact the analysis kernel can store.

use crate::guarantee::{Declared_Guarantee, Payload_Schema, CAPABILITY, CONTRACT_VERSION, PROVIDER};
use crate::scan::{Scan, ScannedFile};
use nomos_analysis::{FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_contracts::{
    BuildVariantId, CapabilityId, ConfigurationId, EvidenceClass, GenerationId, ProviderId,
    SnapshotId, SubjectId,
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

/// Produces the syntax-items fact for one file.
///
/// # Why this returns a fact and not a `Materialization`
///
/// `nomos-lang-rust` has two outcomes because a parse can fail. A scan cannot. Giving this
/// a refusal arm to mirror the other provider's shape would be inventing a case that does
/// not exist, and every caller would carry a branch that never runs — which is how a
/// never-taken branch stays wrong without anybody noticing.
///
/// That asymmetry is the capability's whole point of interest: the two providers do not
/// merely differ in quality, they differ in which inputs they can answer for at all.
#[must_use]
pub fn Materialize(subject: SubjectId, source: &str, context: FactContext) -> MaterializedFact
{
    let scanned = Scan(source);
    let guarantee = Declared_Guarantee();

    let key = nomos_analysis::FactKey {
        contract: CapabilityId::New(CAPABILITY),
        contract_version: CONTRACT_VERSION,
        subject,
        // The file text and only the file text, by the same rule and for the same reason as
        // the parser: two copies of one file are one computation.
        semantic_inputs: InputDigest::Of(&[source.as_bytes()]),
        provider: ProviderId::New(PROVIDER),
        provider_version: CONTRACT_VERSION,
        guarantee: GuaranteeDigest::Of(&guarantee),
        variant: context.variant,
        configuration: context.configuration,
    };

    return MaterializedFact {
        identity: key.At(context.generation),
        snapshot: context.snapshot,
        // Not `Verified`. A parser found the item in a token stream and could not have
        // reported something the text does not contain; this pattern-matched a line, and
        // its own tests enumerate the cases where the line was not a declaration.
        // `Approximate` is the evidence class for a method that trades accuracy for cost,
        // and grading this any higher would launder a heuristic into a measurement.
        evidence: EvidenceClass::Approximate,
        guarantee,
        payload: FactPayload::New(Payload_Schema(), Encode_Payload(&scanned)),
    };
}

/// The canonical byte encoding of a scan.
///
/// The same line format `nomos-lang-rust` writes, hand-written here rather than shared. The
/// duplication is the interface: what makes two providers interchangeable to a consumer is
/// that both produce bytes it can read, and a shared encoder would make that true by
/// construction and prove nothing.
///
/// `unexpanded` is `0` and always will be. A line-reader cannot see a macro invocation
/// inside a function body, so it has no lower bound to offer — and writing the field is not
/// a claim that there were none, it is the format saying the same thing in every payload so
/// a consumer needs no special case.
#[must_use]
pub fn Encode_Payload(scanned: &ScannedFile) -> Vec<u8>
{
    let mut encoded = String::new();

    encoded.push_str("unexpanded\t0\n");

    for item in &scanned.items
    {
        encoded.push_str("item\t");
        encoded.push_str(&item.ordinal.to_string());
        encoded.push('\t');
        encoded.push_str(item.kind.Label());
        encoded.push('\t');
        encoded.push_str(&item.visibility.Label());
        encoded.push('\t');
        encoded.push_str(&item.name);
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

    #[test]
    fn Test_The_Encoding_Should_Be_The_Format_The_Other_Provider_Writes()
    {
        let fact = Materialize(Subject("a.rs"), "pub fn one() {}\nfn two() {}\n", Context());
        let rendered = String::from_utf8(fact.payload.bytes.clone()).expect("ASCII and identifiers");

        assert_eq!(
            rendered,
            "unexpanded\t0\n\
             item\t0\tFunction\tPublic\tone\n\
             item\t1\tFunction\tPrivate\ttwo\n"
        );
        assert!(!rendered.contains('\r'), "line endings must not be local");
    }

    /// The fact says what produced it. Two providers answering one capability about one
    /// subject must be tellable apart by anything holding both.
    #[test]
    fn Test_A_Fact_Should_Name_This_Provider_And_Its_Guarantee()
    {
        let fact = Materialize(Subject("a.rs"), "pub fn one() {}\n", Context());

        assert_eq!(fact.Key().provider, ProviderId::New(PROVIDER));
        assert_eq!(fact.guarantee, Declared_Guarantee());
        assert_eq!(fact.evidence, EvidenceClass::Approximate);
    }

    /// Same bytes, same computation. Without this the two providers would recompute on
    /// every run and the store would grow a fact per pass.
    #[test]
    fn Test_The_Same_Bytes_Should_Reach_The_Same_Key()
    {
        let source = "pub struct S;\npub fn f() {}\n";

        assert_eq!(
            Materialize(Subject("a.rs"), source, Context()).Key().Digest(),
            Materialize(Subject("a.rs"), source, Context()).Key().Digest()
        );
    }

    /// The negative control for the test above.
    #[test]
    fn Test_Different_Bytes_Should_Reach_Different_Keys()
    {
        assert_ne!(
            Materialize(Subject("a.rs"), "pub fn one() {}\n", Context()).Key().Digest(),
            Materialize(Subject("a.rs"), "pub fn two() {}\n", Context()).Key().Digest()
        );
    }

    /// A file with nothing in it produces a payload, not an absence. Zero items is an
    /// answer; the format still carries its `unexpanded` line so a consumer needs no
    /// special case for the empty file.
    #[test]
    fn Test_An_Empty_File_Should_Still_Produce_A_Payload()
    {
        let fact = Materialize(Subject("empty.rs"), "", Context());

        assert_eq!(fact.payload.bytes, b"unexpanded\t0\n");
    }
}
