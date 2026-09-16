//! Requiring and decoding every source's own syntax fact, and indexing those whose fact
//! could be read — the half [`super::comparison`]'s judgment never reaches, because by the
//! time it runs, the payload is already a plain, already-decoded value.
//!
//! Reading is kept apart from judging for the same reason [`crate::naming::reading`] and
//! [`crate::dependency::reading`] both already are: a pure function of an already-decoded
//! payload is testable against hand-built fixtures, and everything in this file is instead
//! about how that payload gets found and decoded in the first place — the half no fixture
//! reaches.

use crate::{SourceFile, Syntax_Requirement_For};
use nomos_analysis::{FactReader, InputDigest, MaterializedFact};
use nomos_cap_syntax::{PayloadRefusal, SyntaxPayload};
use nomos_contracts::{Applicability, SchemaId};

/// Every source's decoded `nomos.cap.syntax.items` payload, for the sources whose fact
/// could be read.
pub(super) fn Struct_Index<'a>(sources: &'a [SourceFile], facts: &mut dyn FactReader) -> Vec<(&'a SourceFile, SyntaxPayload)>
{
    return sources
        .iter()
        .filter_map(|source| return Payload_Of(source, facts).ok().map(|payload| return (source, payload)))
        .collect();
}

/// Why one source's own syntax payload could not be produced.
///
/// The cause is carried, not replaced. [`Struct_Index`] asks only *whether* a payload
/// resolved, and leaves a subject out of the index either way — but "no provider answered"
/// and "this build cannot decode what one answered with" are different answers, and which
/// one happened is known only at the point the refusal is raised. Building a fresh `()` or
/// a rendered sentence there would discard exactly the value whoever debugs an empty index
/// needs, so each variant names the value the reader or the decoder actually handed back.
#[derive(Clone, Debug, PartialEq, Eq)]
enum PayloadFailure
{
    /// The reader resolved no fact for this subject, naming the state it answered with.
    Unread(Applicability),
    /// The fact's payload was encoded against a schema this build does not read, naming the
    /// schema it carried in place of [`nomos_cap_syntax::Payload_Schema`].
    ForeignSchema(SchemaId),
    /// This build's own decoder refused the payload's bytes.
    Undecodable(PayloadRefusal),
}

/// This source's decoded syntax payload, or the [`PayloadFailure`] naming what prevented it
/// under this rule's own floor — the identical `Require`-then-decode shape
/// `nomos_rules::naming::reading::Payload_Of` already uses for the same capability, kept
/// local rather than shared: each rule that reads `nomos.cap.syntax.items` states and
/// discharges its own floor, the same independence `Check_Lint_Diagnostics` and
/// `Check_Dependency_Policy` already keep from each other for their own capabilities.
///
/// # Errors
///
/// [`PayloadFailure::Unread`] when the reader admits no provider for this subject or holds
/// no fact under it, [`PayloadFailure::ForeignSchema`] when the fact carries another
/// build's payload schema, and [`PayloadFailure::Undecodable`] when the bytes are not a
/// well-formed payload under this build's grammar.
fn Payload_Of(source: &SourceFile, facts: &mut dyn FactReader) -> Result<SyntaxPayload, PayloadFailure>
{
    let need = Syntax_Requirement_For(source.preferred_syntax_provider.clone());
    let capability = nomos_cap_syntax::Capability();
    let inputs = InputDigest::Of(&[source.text.as_bytes()]);

    let fact: &MaterializedFact = facts
        .Require(&capability, &source.subject, inputs, &need)
        .map_err(PayloadFailure::Unread)?;

    if fact.payload.schema != nomos_cap_syntax::Payload_Schema()
    {
        return Err(PayloadFailure::ForeignSchema(fact.payload.schema.clone()));
    }

    return nomos_cap_syntax::Parse_Payload(&fact.payload.bytes).map_err(PayloadFailure::Undecodable);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::checks::test_support::{self, FactToFile, OfferedProvider, Test_Context, TestOffering};
    use nomos_analysis::Reader;
    use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, SubjectId};
    use nomos_model::Content_Digest;

    const PARSER: &str = "nomos.test.crosslang.reading.parses";

    #[test]
    fn Test_Struct_Index_Should_Include_Only_The_Sources_Whose_Fact_Could_Be_Read()
    {
        let readable = Source("readable.rs");
        let unread = Source("unread.rs");
        let TestOffering { mut store, registry, offer } = Offering();
        test_support::Materialize(
            &mut store,
            FactToFile {
                subject: readable.subject,
                offer: &offer,
                semantic_inputs: nomos_analysis::InputDigest::Of(&[readable.text.as_bytes()]),
                schema: nomos_cap_syntax::Payload_Schema(),
                bytes: "unexpanded\t0\n".as_bytes().to_vec(),
            },
        ).expect("the fixture's store holds no fact under this key at a newer generation");
        let mut reader = Reader::On(&store, &registry, Test_Context());

        let sources = [readable.clone(), unread.clone()];
        let index = Struct_Index(&sources, &mut reader);

        assert_eq!(index.len(), 1, "expected exactly the readable source: {}", index.len());
        assert_eq!(index.first().map(|(source, _)| source.path.clone()), Some(readable.path));
    }

    fn Source(path: &str) -> SourceFile
    {
        return SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), String::new());
    }

    fn Offering() -> TestOffering
    {
        return test_support::Offering(
            OfferedProvider {
                contract: nomos_cap_syntax::Capability_Contract(),
                capability: nomos_cap_syntax::Capability(),
                version: nomos_cap_syntax::CONTRACT_VERSION,
                provider: PARSER,
                guarantee: Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File),
            },
        ).expect("a fresh Registry holds neither this contract nor this provider");
    }
}
