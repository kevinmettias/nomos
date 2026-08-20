//! Requiring and decoding a workspace member's own dependency fact.
//!
//! Reading is kept apart from judging (`violations.rs`) for the same reason
//! [`crate::naming::reading`] is: a pure function of an already-decoded payload is testable
//! against hand-built fixtures, and everything in this file is instead about how that
//! payload gets found and decoded in the first place — the half no fixture reaches.

use crate::SourceFile;
use nomos_analysis::{FactReader, InputDigest, MaterializedFact};
use nomos_capability::Requirement;
use nomos_cap_dependency::DependencyPayload;
use nomos_contracts::{
    Applicability, Assurance, EvidenceClass, FactVariant, Finding, GateCategory, Guarantee,
    IncrementalGranularity, RuleId,
};

/// What this rule needs from `nomos.cap.dependency.edges` before it will believe an
/// answer.
///
/// The ceiling itself: nothing weaker than Cargo's own resolution could tell a first-party
/// path dependency apart from an unrelated registry crate of the same name, and a rule
/// judging architecture cannot afford that ambiguity. Soundness `Sound` because every edge
/// this rule acts on must really be a resolved one; completeness `Sound` because
/// `nomos-lang-rust-cargo`'s own guarantee states it honestly achieves that, unlike a
/// syntax provider bounded by what a macro might hide.
#[must_use]
pub(crate) fn Dependency_Requirement() -> Requirement
{
    let guarantee = Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::Project,
    );

    return Requirement::New(
        nomos_cap_dependency::Capability(),
        nomos_cap_dependency::CONTRACT_VERSION,
        guarantee,
    );
}

/// One member's decoded dependency payload, or a finding reporting why it could not be
/// read.
pub(super) fn Payload_Of(source: &SourceFile, facts: &mut dyn FactReader) -> Result<DependencyPayload, Finding>
{
    let fact = Require_Fact(source, facts)?;
    Check_Schema(source, fact)?;
    return Parse_Fact(source, fact);
}

/// Requires this member's dependency fact, turning an inadmissible answer into an
/// [`Unread`] finding.
fn Require_Fact<'a>(source: &SourceFile, facts: &'a mut dyn FactReader) -> Result<&'a MaterializedFact, Finding>
{
    let need = Dependency_Requirement();
    let capability = nomos_cap_dependency::Capability();
    // The provider's semantic input is the encoded payload it wrote, not `source.text` —
    // this rule does not know that encoding and must not guess it to key a lookup. A
    // reader resolves a fact by subject and requirement, not by recomputing the
    // provider's own input digest, so an empty digest here names nothing this rule is
    // required to get right.
    let inputs = InputDigest::Of(&[]);

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
    if fact.payload.schema != nomos_cap_dependency::Payload_Schema()
    {
        return Err(Unread(
            source,
            Applicability::Unparseable,
            &format!(
                "the fact for this member carries payload schema `{}`, which this build \
                 does not read",
                fact.payload.schema
            ),
        ));
    }

    return Ok(());
}

/// Decodes `fact`'s payload bytes into this rule's own [`DependencyPayload`] shape.
fn Parse_Fact(source: &SourceFile, fact: &MaterializedFact) -> Result<DependencyPayload, Finding>
{
    return nomos_cap_dependency::Parse_Payload(&fact.payload.bytes)
        .map_err(|refusal| return Unread(source, Applicability::Unparseable, &refusal.to_string()));
}

fn Unread(source: &SourceFile, applicability: Applicability, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(super::DEPENDENCY_DIRECTION),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("this member's dependency direction could not be judged: {because}"),
        locations: vec![source.path.clone()],
    };
}
