//! Requiring and decoding one file's own syntax fact.
//!
//! Reading is kept apart from judging (`violations.rs`) for the same reason
//! [`crate::dependency::reading`] is: a pure function of an already-decoded payload is
//! testable against hand-written fixture text, and everything in this file is instead
//! about how that payload gets found and decoded in the first place — the half no
//! fixture reaches.

use crate::{SourceFile, Syntax_Requirement};
use nomos_analysis::{FactReader, InputDigest, MaterializedFact};
use nomos_cap_syntax::SyntaxPayload;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// One file's decoded syntax payload, or a finding reporting why it could not be read.
///
/// A subject whose fact could not be read must not render as a subject that declared
/// nothing — the same principle [`crate::Check_Completeness_Mirrors`] holds for the same
/// reason, restated here because this rule keeps its own, simpler index rather than
/// sharing the mirror rule's.
pub(super) fn Payload_Of(source: &SourceFile, facts: &mut dyn FactReader) -> Result<SyntaxPayload, Finding>
{
    let fact = Require_Fact(source, facts)?;
    Check_Schema(source, fact)?;
    return Parse_Fact(source, fact);
}

/// Requires this file's syntax fact, turning an inadmissible answer into an [`Unread`]
/// finding.
fn Require_Fact<'a>(source: &SourceFile, facts: &'a mut dyn FactReader) -> Result<&'a MaterializedFact, Finding>
{
    let need = Syntax_Requirement();
    let capability = nomos_cap_syntax::Capability();
    let inputs = InputDigest::Of(&[source.text.as_bytes()]);

    return match facts.Require(&capability, &source.subject, inputs, &need)
    {
        Ok(fact) => Ok(fact),
        Err(applicability) => Err(Unread(
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
        return Err(Unread(
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
        .map_err(|refusal| return Unread(source, Applicability::Unparseable, &refusal.Describe()));
}

/// A finding for a subject whose fact this rule could not read.
fn Unread(source: &SourceFile, applicability: Applicability, because: &str) -> Finding
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
