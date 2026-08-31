//! Requiring and decoding a workspace member's own dependency fact.
//!
//! Reading is kept apart from judging (`violations.rs`) for the same reason
//! [`crate::naming::reading`] is: a pure function of an already-decoded payload is testable
//! against hand-built fixtures, and everything in this file is instead about how that
//! payload gets found and decoded in the first place — the half no fixture reaches.
//!
//! [`Payload_Of`] takes the calling rule's own [`RuleId`] label rather than naming
//! [`super::DEPENDENCY_DIRECTION`] itself: both [`super::Check_Dependency_Direction`] and
//! [`super::Check_Every_Member_Declares_A_Band`] read the identical
//! `nomos.cap.dependency.edges` fact through the identical requirement, and an unread
//! subject has to be reported under whichever rule actually asked for it, not always the
//! first rule that needed this reader. Two real callers is what licenses the parameter —
//! before the second rule existed there was nothing to disambiguate.

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
/// read, filed under `rule` — whichever rule actually asked.
pub(super) fn Payload_Of(source: &SourceFile, facts: &mut dyn FactReader, rule: &'static str) -> Result<DependencyPayload, Finding>
{
    let fact = Require_Fact(source, facts, rule)?;
    Check_Schema(source, fact, rule)?;
    return Parse_Fact(source, fact, rule);
}

/// Requires this member's dependency fact, turning an inadmissible answer into an
/// [`Unread`] finding under `rule`.
fn Require_Fact<'a>(source: &SourceFile, facts: &'a mut dyn FactReader, rule: &'static str) -> Result<&'a MaterializedFact, Finding>
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
        Err(applicability) => Err(Unread_Finding(
            source,
            applicability,
            rule,
            &format!("no admitted provider answered for it ({})", applicability.Label()),
        )),
    };
}

/// Confirms `fact`'s payload schema is the one this rule knows how to decode.
fn Check_Schema(source: &SourceFile, fact: &MaterializedFact, rule: &'static str) -> Result<(), Finding>
{
    if fact.payload.schema != nomos_cap_dependency::Payload_Schema()
    {
        return Err(Unread_Finding(
            source,
            Applicability::Unparseable,
            rule,
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
fn Parse_Fact(source: &SourceFile, fact: &MaterializedFact, rule: &'static str) -> Result<DependencyPayload, Finding>
{
    return nomos_cap_dependency::Parse_Payload(&fact.payload.bytes)
        .map_err(|refusal| return Unread_Finding(source, Applicability::Unparseable, rule, &refusal.to_string()));
}

fn Unread_Finding(source: &SourceFile, applicability: Applicability, rule: &'static str, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(rule),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("this member's dependency fact could not be read: {because}"),
        locations: vec![source.path.clone()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::checks::test_support::{self, Test_Context, TestOffering};
    use nomos_analysis::{InputDigest, MemoryFactStore, Reader};
    use nomos_capability::ProviderOffer;
    use nomos_cap_dependency::DependencyPayload;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    const PROVIDER: &str = "nomos.test.dependency.reading.resolves";

    #[test]
    fn Test_Dependency_Requirement_Should_Be_Met_By_The_Real_Providers_Own_Guarantee()
    {
        assert!(
            nomos_cap_dependency::Capability_Contract().ceiling.Satisfies(&Dependency_Requirement().minimum),
            "the capability's own ceiling must be able to satisfy this rule's floor, or no real provider ever could"
        );
    }

    #[test]
    fn Test_Payload_Of_Should_Decode_A_Materialized_Fact_Under_The_Asking_Rule()
    {
        let source = Source("nomos-cap-syntax");
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Dependency_Fact(
            &mut store,
            &source,
            &offer,
            &DependencyPayload { package: "nomos-cap-syntax".to_owned(), edges: Vec::new() },
        );
        let mut reader = Reader::On(&store, &registry, Test_Context());

        let payload = Payload_Of(&source, &mut reader, "example-rule").expect("the fact was just materialized");

        assert_eq!(payload.package, "nomos-cap-syntax");
    }

    fn Materialize_Dependency_Fact(store: &mut MemoryFactStore, source: &SourceFile, offer: &ProviderOffer, payload: &DependencyPayload)
    {
        let bytes = nomos_cap_dependency::Encode_Payload(payload);
        test_support::Materialize(store, source.subject, offer, InputDigest::Of(&[]), nomos_cap_dependency::Payload_Schema(), bytes);
    }

    #[test]
    fn Test_Payload_Of_Should_Report_An_Unread_Subject_Under_Whichever_Rule_Asked()
    {
        let source = Source("nomos-cap-syntax");
        let TestOffering { store, registry, .. } = Offering();
        let mut reader = Reader::On(&store, &registry, Test_Context());

        let refused = Payload_Of(&source, &mut reader, "example-rule").expect_err("no fact was materialized");

        assert_eq!(refused.rule, RuleId::New("example-rule"));
    }

    fn Source(package: &str) -> SourceFile
    {
        return SourceFile::New(package, SubjectId::From_Digest(Content_Digest(package.as_bytes())), String::new());
    }

    fn Offering() -> TestOffering
    {
        return test_support::Offering(
            nomos_cap_dependency::Capability_Contract(),
            nomos_cap_dependency::Capability(),
            nomos_cap_dependency::CONTRACT_VERSION,
            PROVIDER,
            Dependency_Requirement().minimum,
        );
    }
}
