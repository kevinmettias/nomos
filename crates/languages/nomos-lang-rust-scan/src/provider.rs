//! Turning a scan into a fact the analysis kernel can store.

use crate::guarantee::Declared_Guarantee;
use crate::guarantee::PROVIDER;
use crate::scan::Scan;
use crate::scanned_file::ScannedFile;
use crate::scan::ScannedItem;
use nomos_analysis::{FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_cap_syntax::{Capability, Payload_Schema, CONTRACT_VERSION};
use nomos_contracts::{
    Guarantee,
    BuildVariantId, ConfigurationId, EvidenceClass, GenerationId, ProviderId, SnapshotId,
    SubjectId,
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
    let key = Keyed(subject, source, guarantee, context);

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

/// What this fact is, as against where it came from.
///
/// The semantic inputs are the file text and only the file text, by the same rule and for
/// the same reason as the parser: two copies of one file are one computation.
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
        semantic_inputs: InputDigest::Of(&[source.as_bytes()]),
        provider: ProviderId::New(PROVIDER),
        provider_version: CONTRACT_VERSION,
        guarantee: GuaranteeDigest::Of(&guarantee),
        variant: context.variant,
        configuration: context.configuration,
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
        Encode_Item(&mut encoded, item);
    }

    return encoded.into_bytes();
}

/// One item as a record, with both unobservable fields marked as unobserved.
///
/// Not observed, twice, and never absent. This reader skips comment lines and associates
/// nothing with the item below them, and it never looks at a declared type at all. Writing
/// `.` here would say it looked and found nothing — which for a list that does declare its
/// mirror is a phantom silently downgraded to an admitted gap. See `OD-SYNTAX-002`.
fn Encode_Item(encoded: &mut String, item: &ScannedItem)
{
    encoded.push_str("item\t");
    encoded.push_str(&item.ordinal.to_string());
    encoded.push('\t');
    encoded.push_str(item.kind.Label());
    encoded.push('\t');
    encoded.push_str(&item.visibility.Label());
    encoded.push('\t');
    encoded.push_str(&item.name);
    encoded.push_str("\t-\t-");
    encoded.push('\n');
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
             item\t0\tFunction\tPublic\tone\t-\t-\n\
             item\t1\tFunction\tPrivate\ttwo\t-\t-\n"
        );
        assert!(!rendered.contains('\r'), "line endings must not be local");
    }

    /// What this provider writes is what the schema says a payload is.
    ///
    /// The encoder above is hand-written and stays that way: what makes two providers of
    /// one capability interchangeable is that each authors the format independently and a
    /// third party can read both. The check the duplication was missing is this one —
    /// against the grammar in `nomos-cap-syntax`, which belongs to neither provider.
    #[test]
    fn Test_This_Providers_Payload_Should_Decode_Under_The_Schemas_Own_Reader()
    {
        let fact = Materialize(Subject("a.rs"), "pub fn one() {}\nfn two() {}\n", Context());

        let payload = nomos_cap_syntax::Parse_Payload(&fact.payload.bytes)
            .expect("this provider writes nomos.syntax.items.v1");

        assert_eq!(payload.unexpanded, 0);
        assert_eq!(payload.items.len(), 2);
        assert!(payload.items.first().expect("two items").Is_Public());
    }

    /// This provider never writes the mark, and that is the schema's caveat from the other
    /// side.
    ///
    /// A line reader cannot see the enclosing trait, so it records the member as private —
    /// a conforming payload that spells the same source construct differently from the
    /// parser's. It is why the schema declines to state that a `Function` without the mark
    /// is a definition: the absence here is blindness rather than an observation, and a
    /// consumer that needs the distinction has to require it of the guarantee instead.
    #[test]
    fn Test_A_Trait_Member_Should_Not_Be_Marked_By_A_Reader_That_Cannot_See_The_Trait()
    {
        let fact = Materialize(
            Subject("a.rs"),
            "pub trait Judged\n{\n    fn Check(&self);\n}\n",
            Context(),
        );

        let payload = nomos_cap_syntax::Parse_Payload(&fact.payload.bytes).expect("well formed");

        let member = payload
            .items
            .iter()
            .find(|item| return item.Own_Name() == "Check")
            .expect("the line reader sees the signature");
        assert!(
            !member.Declares_No_Visibility(),
            "this provider has no such value to write: {member:?}"
        );
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
