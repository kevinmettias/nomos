//! Requiring and decoding one file's own syntax fact.
//!
//! Reading is kept apart from judging (`violations.rs`) for the same reason
//! [`crate::dependency::reading`] is: a pure function of an already-decoded payload is
//! testable against hand-written fixture text, and everything in this file is instead
//! about how that payload gets found and decoded in the first place — the half no
//! fixture reaches.

use crate::{SourceFile, Syntax_Requirement_For};
use nomos_analysis::{FactReader, InputDigest, MaterializedFact};
use nomos_cap_syntax::SyntaxPayload;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// One file's decoded syntax payload, or a finding reporting why it could not be read.
///
/// A subject whose fact could not be read must not render as a subject that declared
/// nothing — the same principle [`crate::Check_Completeness_Mirrors`] holds for the same
/// reason, restated here because this rule keeps its own, simpler index rather than
/// sharing the mirror rule's.
pub(in crate::checks) fn Payload_Of(source: &SourceFile, facts: &mut dyn FactReader) -> Result<SyntaxPayload, Finding>
{
    let fact = Require_Fact(source, facts)?;
    Check_Schema(source, fact)?;
    return Parse_Fact(source, fact);
}

/// Every source's own violations, read and judged the one way a rule that judges each whole
/// file does it.
///
/// The walk -- decode this source's payload, hand it to `judge`, and fold either a real
/// reading failure or the violations that came back into one list sorted by subject -- was
/// written out by hand in [`super::boolean_predicates`], [`super::single_letter_names`] and
/// [`super::test_names`] before it lived here, and `check-interfile-duplication` reported all
/// three. Only two things ever varied between them: how a decoded payload becomes findings,
/// and which words an unread file's finding replaces. Both are parameters.
///
/// `unread` is the second one because it is not decoration: [`Unread_Finding`] above reports
/// every read failure under [`super::NAMING_CONVENTION`]'s own identifier and summary, so a
/// rule that passed the finding through unchanged would report another rule's finding as its
/// own. Each caller passes its own relabelling, and one that had none to do would pass
/// `|finding| return finding`.
///
/// `judge` is handed the path as well as the payload rather than the payload alone, because a
/// violation names the file it was found in and a payload does not carry its own path. Every
/// caller's `Violations_In` is already this parameter's exact signature.
///
/// The test-and-example skip [`super::file_names`]'s own copy applies is deliberately *not*
/// here: none of this walk's callers has decided to exempt those sources, and a walk that
/// silently dropped them would answer a question nobody asked it rather than the one it was
/// given.
pub(in crate::checks) fn Judged_Sources(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
    judge: fn(&SyntaxPayload, &str) -> Vec<Finding>,
    unread: fn(Finding) -> Finding,
) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        match Payload_Of(source, facts)
        {
            Ok(payload) =>
            {
                let violations = judge(&payload, &source.path);
                findings.extend(violations);
            }
            Err(finding) => findings.push(unread(finding)),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Requires this file's syntax fact, turning an inadmissible answer into an [`Unread`]
/// finding.
fn Require_Fact<'a>(source: &SourceFile, facts: &'a mut dyn FactReader) -> Result<&'a MaterializedFact, Finding>
{
    let need = Syntax_Requirement_For(source.preferred_syntax_provider.clone());
    let capability = nomos_cap_syntax::Capability();
    let inputs = InputDigest::Of(&[source.text.as_bytes()]);

    return match facts.Require(&capability, &source.subject, inputs, &need)
    {
        Ok(fact) => Ok(fact),
        Err(applicability) => Err(Unread_Finding(
            source,
            applicability,
            &format!("no admitted provider answered for it ({})", applicability.Label()),
        )),
    };
}

/// Confirms `fact`'s payload schema is the one this rule knows how to decode.
fn Check_Schema(source: &SourceFile, fact: &MaterializedFact) -> Result<(), Finding>
{
    if fact.payload.schema != nomos_cap_syntax::Payload_Schema()
    {
        return Err(Unread_Finding(
            source,
            Applicability::Unparseable,
            &format!(
                "the fact for this file carries payload schema `{}`, which this build does \
                 not read",
                fact.payload.schema
            ),
        ));
    }

    return Ok(());
}

/// Decodes `fact`'s payload bytes into this rule's own [`SyntaxPayload`] shape.
fn Parse_Fact(source: &SourceFile, fact: &MaterializedFact) -> Result<SyntaxPayload, Finding>
{
    return nomos_cap_syntax::Parse_Payload(&fact.payload.bytes)
        .map_err(|refusal| return Unread_Finding(source, Applicability::Unparseable, &refusal.Describe()));
}

/// A finding for a subject whose fact this rule could not read.
fn Unread_Finding(source: &SourceFile, applicability: Applicability, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(super::NAMING_CONVENTION),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("this file's naming could not be judged: {because}"),
        locations: vec![source.path.clone()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::checks::test_support::{self, OfferedProvider, Test_Context, TestOffering};
    use nomos_analysis::Reader;
    use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, SubjectId};
    use nomos_model::Content_Digest;

    const PARSER: &str = "nomos.test.naming.reading.parses";

    #[test]
    fn Test_Payload_Of_Should_Decode_A_Materialized_Fact()
    {
        let source = Source_File(SourceText { path: "src/lib.rs", text: "fn Good_Name() {}" });
        let mut offering = Offering();
        let mut reader = test_support::Reader_Over_A_Syntax_Fact(
            &mut offering,
            &source,
            "unexpanded\t0\nitem\t0\tFunction\tPublic\tGood_Name\t.\t+fn/0\n",
        ).expect("the fixture's store holds no fact under this key at a newer generation");

        let payload = Payload_Of(&source, &mut reader).expect("the fact was just materialized");

        assert_eq!(payload.items.len(), 1, "{payload:?}");
    }

    #[test]
    fn Test_Payload_Of_Should_Report_A_Subject_With_No_Fact_As_A_Finding()
    {
        let source = Source_File(SourceText { path: "src/lib.rs", text: "fn Good_Name() {}" });
        let TestOffering { store, registry, .. } = Offering();
        let mut reader = Reader::On(&store, &registry, Test_Context());

        let refused = Payload_Of(&source, &mut reader);

        assert!(refused.is_err(), "{refused:?}");
    }

        /// A fixture source's two halves, grouped so a call site names which string is the path
    /// and which is the text, rather than counting two adjacent `&str` positions a caller
    /// could transpose without the compiler objecting.
    struct SourceText<'text>
    {
        path: &'text str,
        text: &'text str,
    }

    fn Source_File(source: SourceText<'_>) -> SourceFile
    {
        let SourceText { path, text } = source;
        return SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
    }

    fn Offering() -> TestOffering
    {
        return test_support::Offered_Registry(
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
