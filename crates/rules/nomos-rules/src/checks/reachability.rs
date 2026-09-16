//! Every control-flow path that begins at a fact-read failure must reach a `Finding`
//! before the enclosing function returns — the property `Applicability`'s own module doc
//! names as this product's first principle: unknown is not pass.
//!
//! # `OD-RULES-008` specified this; `P13-CONTROLFLOW-REACHABILITY-CAPABILITY` built it
//!
//! `OD-RULES-008` named `Check_Unread_Reaches_A_Finding` as the candidate for the first
//! rule outside syntax-local judgment and dependency-graph reachability, and traced what
//! building it would force: an intraprocedural control-flow capability, `OD-ANALYSIS-007`
//! narrowed rather than closed. This is that rule's tier-1 form — a heuristic over one
//! file's parse tree, not the sound tier the same record's own analysis names.
//!
//! # This rule states its own floor honestly, below the capability's ceiling
//!
//! `nomos_cap_controlflow`'s ceiling is `FactVariant::SemanticallyResolved` — what a sound
//! answer needs. `nomos-capability`'s own resolution code admits an offer only once it
//! clears a caller's stated floor (`crates/substrate/nomos-capability/src/registry.rs`'s
//! `Selected`), so a `Requirement` asking for `SemanticallyResolved` today would find
//! nothing — no offer against this capability claims it yet. This rule's own
//! [`Reachability_Requirement`] asks for exactly [`nomos_contracts::FactVariant::Syntactic`],
//! what `nomos-lang-rust`'s tier-1 offer actually delivers, and every finding it raises
//! carries [`nomos_contracts::Applicability::PartiallySupported`] rather than `Supported` —
//! evaluated over part of the question (the syntactically obvious wrong shapes), not the
//! whole one `OD-RULES-008`'s sound tier states.
//!
//! # Composed into `nomos-check-orchestration::Run`
//!
//! The same split `Check_Dependency_Direction` used: `P13-CONTROLFLOW-REACHABILITY-
//! CAPABILITY` built and tested this rule and its provider, and `P13-CONTROLFLOW-
//! REACHABILITY-WIRE` composed it into a real run, the way `P13-DEPENDENCY-WIRE-1`
//! followed `P13-DEPENDENCY-EDGES-2`.

use crate::SourceFile;
use nomos_analysis::{FactReader, InputDigest, MaterializedFact};
use nomos_capability::Requirement;
use nomos_cap_controlflow::{ArmShape, ReachabilityPayload, ReachabilitySite};
use nomos_contracts::{
    Applicability, Assurance, EvidenceClass, FactVariant, Finding, GateCategory,
    Guarantee, IncrementalGranularity, RuleId,
};

/// This rule's own identifier.
pub const UNREAD_REACHES_FINDING: &str = "unread-reaches-finding";

/// The record this implementation's contract is written in.
///
/// `OD-RULES-008` named this rule's candidate and traced what building it would force;
/// `P13-CONTROLFLOW-REACHABILITY-CAPABILITY` amended its `Status` once the tier-1 provider
/// and this rule existed, which is why this citation names version 2 rather than the
/// record's original 1 — `tests/contract/tests/rule_contract_citation.rs` reads the
/// record's own front matter on every run and compares it against
/// [`UNREAD_REACHES_FINDING_CONTRACT_RECORD_VERSION`] for exactly this reason.
pub const UNREAD_REACHES_FINDING_CONTRACT_RECORD: &str = "OD-RULES-008";

/// The version of [`UNREAD_REACHES_FINDING_CONTRACT_RECORD`] this implementation was
/// written against.
pub const UNREAD_REACHES_FINDING_CONTRACT_RECORD_VERSION: u32 = 2;

/// What this rule needs from `nomos.cap.controlflow.reachability` before it will believe
/// an answer.
///
/// Asks for exactly `Syntactic` — what this item's own tier-1 provider offers — rather
/// than the capability's `SemanticallyResolved` ceiling. This module's own doc says why:
/// asking above every installed offer's guarantee would make the offer unreachable, not
/// merely graded as a fallback. Soundness `Sound` because a flagged site must really be
/// one of `ArmShape`'s four shapes; completeness `Unknown` because this rule does not
/// require a bound on what the provider missed — it already knows, from its own floor,
/// that it is reading a heuristic and reports accordingly.
#[must_use]
pub(crate) fn Reachability_Requirement() -> Requirement
{
    let guarantee = Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );

    return Requirement::New(
        nomos_cap_controlflow::Capability(),
        nomos_cap_controlflow::CONTRACT_VERSION,
        guarantee,
    );
}

/// Judges every source `sources` names for arms whose control flow does not reach a
/// `Finding`.
///
/// The same shape [`crate::Check_Dependency_Direction`] and
/// [`crate::Check_Naming_Convention`] both have: one fact per source, read and judged.
#[must_use]
pub fn Check_Unread_Reaches_A_Finding(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        match Payload_Of(source, facts)
        {
            Ok(payload) =>
            {
                let violations = Violations_In(&payload, source);
                findings.extend(violations);
            }
            Err(finding) => findings.push(finding),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// One source's decoded reachability payload, or a finding reporting why it could not be
/// read.
fn Payload_Of(source: &SourceFile, facts: &mut dyn FactReader) -> Result<ReachabilityPayload, Finding>
{
    let fact = Require_Fact(source, facts)?;
    Check_Schema(source, fact)?;
    return Parse_Fact(source, fact);
}

/// Requires this source's reachability fact, turning an inadmissible answer into an
/// [`Unread`] finding.
fn Require_Fact<'a>(source: &SourceFile, facts: &'a mut dyn FactReader) -> Result<&'a MaterializedFact, Finding>
{
    let need = Reachability_Requirement();
    let capability = nomos_cap_controlflow::Capability();
    // Not an empty digest the way `dependency.rs`'s own `Payload_Of` uses — that provider's
    // own `semantic_inputs` is empty by its own stated convention
    // (`nomos-lang-rust-cargo`'s `provider.rs`), so an empty digest here matches it. This
    // capability's provider is content-keyed the same way `nomos.cap.syntax.items` is —
    // `naming.rs`'s own `Payload_Of` is the precedent this follows instead.
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
    if fact.payload.schema != nomos_cap_controlflow::Payload_Schema()
    {
        return Err(Unread_Finding(
            source,
            Applicability::Unparseable,
            &format!(
                "the fact for this source carries payload schema `{}`, which this build \
                 does not read",
                fact.payload.schema
            ),
        ));
    }

    return Ok(());
}

/// Decodes `fact`'s payload bytes into this rule's own [`ReachabilityPayload`] shape.
fn Parse_Fact(source: &SourceFile, fact: &MaterializedFact) -> Result<ReachabilityPayload, Finding>
{
    return nomos_cap_controlflow::Parse_Payload(&fact.payload.bytes)
        .map_err(|refusal| return Unread_Finding(source, Applicability::Unparseable, &refusal.to_string()));
}

/// Every flagged site `payload` carries, as findings.
///
/// A pure function of an already-decoded payload, testable against hand-built fixtures —
/// no registry, no store, no reader — the same split [`crate::naming::Violations_In`] and
/// [`crate::dependency::Violations_In`] both draw for the same reason.
fn Violations_In(payload: &ReachabilityPayload, source: &SourceFile) -> Vec<Finding>
{
    return payload.sites.iter().map(|site| return Violation_For_Site(source, site)).collect();
}

fn Violation_For_Site(source: &SourceFile, site: &ReachabilitySite) -> Finding
{
    return Finding {
        rule: RuleId::New(UNREAD_REACHES_FINDING),
        subject: source.subject,
        subject_name: source.path.clone(),
        // Not `Supported` — this module's own doc says why: a heuristic pattern match
        // over one arm's body judged part of the question, not the whole one a sound,
        // call-resolving provider would answer.
        applicability: Applicability::PartiallySupported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "{}: the `Err({})` arm in `{}` is {}, so a control-flow path from this \
             fact-read failure does not reach a Finding — the exact defect `Applicability`'s \
             own module doc calls unknown is not pass.",
            source.path,
            site.binding,
            site.function,
            Shape_Description(site.shape)
        ),
        locations: vec![source.path.clone()],
    };
}

fn Shape_Description(shape: ArmShape) -> &'static str
{
    return match shape
    {
        ArmShape::Empty => "empty",
        ArmShape::BareContinue => "a bare `continue` with no other effect",
        ArmShape::BareReturn => "a bare `return` with no value",
        ArmShape::TailOk => "a tail call to `Ok(...)`, treating the failure as success",
    };
}

fn Unread_Finding(source: &SourceFile, applicability: Applicability, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(UNREAD_REACHES_FINDING),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("this source's reachability could not be judged: {because}"),
        locations: vec![source.path.clone()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_model::Content_Digest;
    use nomos_contracts::SubjectId;

    fn Source_File(path: &str) -> SourceFile
    {
        return SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), String::new());
    }

    fn Reachability_Site(function: &str, shape: ArmShape) -> ReachabilitySite
    {
        return ReachabilitySite {
            function: function.to_owned(),
            binding: "applicability".to_owned(),
            shape,
        };
    }

    mod judging
    {
        use super::{ArmShape, Applicability, GateCategory, Reachability_Site, ReachabilityPayload, Source_File, Violations_In};

        /// The three sites the fixture declares below, one per arm shape, and therefore the
        /// three findings a relay of it must produce.
        const EXPECTED_SITE_FINDINGS: usize = 3;

        #[test]
        fn Test_A_Payload_With_No_Sites_Should_Produce_No_Finding()
        {
            let payload = ReachabilityPayload { sites: Vec::new() };

            let findings = Violations_In(&payload, &Source_File("a.rs"));

            assert!(findings.is_empty(), "{findings:?}");
        }

        #[test]
        fn Test_One_Flagged_Site_Should_Produce_One_Finding()
        {
            let payload = ReachabilityPayload {
                sites: vec![Reachability_Site("Payload_Of", ArmShape::Empty)],
            };

            let findings = Violations_In(&payload, &Source_File("a.rs"));

            assert_eq!(findings.len(), 1, "{findings:?}");
            let found = findings.first().expect("asserted len 1 above");
            assert_eq!(found.applicability, Applicability::PartiallySupported);
            assert_eq!(found.gate, GateCategory::Advisory);
            assert!(found.summary.contains("Payload_Of"), "{}", found.summary);
        }

        #[test]
        fn Test_Every_Site_Should_Produce_Its_Own_Finding()
        {
            let payload = ReachabilityPayload {
                sites: vec![
                    Reachability_Site("One", ArmShape::BareContinue),
                    Reachability_Site("Two", ArmShape::BareReturn),
                    Reachability_Site("Three", ArmShape::TailOk),
                ],
            };

            let findings = Violations_In(&payload, &Source_File("a.rs"));

            assert_eq!(findings.len(), EXPECTED_SITE_FINDINGS, "{findings:?}");
        }
    }

    /// [`Check_Unread_Reaches_A_Finding`] itself, through a real registry, store and
    /// reader — the half [`Violations_In`]'s own tests do not reach.
    mod reading_a_fact
    {
        use super::{
            ArmShape, Assurance, Check_Unread_Reaches_A_Finding, FactVariant, Guarantee, IncrementalGranularity, Reachability_Site,
            ReachabilityPayload, Source_File, SourceFile,
        };
        use crate::checks::test_support::{self, FactToFile, OfferedProvider, Test_Context, TestOffering};
        use nomos_analysis::{InputDigest, MemoryFactStore, Reader};
        use nomos_capability::ProviderOffer;

        const PROVIDER: &str = "nomos.test.reachability.resolves";

        /// Also [`super::super::Reachability_Requirement`]'s own shape: the registered offer
        /// is built at exactly that floor, which is what admits the fact this test reads back.
        #[test]
        fn Test_Reachability_Requirement_Should_Admit_A_Real_Fact_And_Have_It_Judged()
        {
            let source = Source_File("a.rs");
            let TestOffering { mut store, registry, offer } = Offering();
            Materialize_Reachability_Fact(
                &mut store,
                &source,
                &offer,
                &ReachabilityPayload {
                    sites: vec![Reachability_Site("Payload_Of", ArmShape::Empty)],
                },
            );

            let mut reader = Reader::On(&store, &registry, Test_Context());
            let findings = Check_Unread_Reaches_A_Finding(&[source], &mut reader);

            assert_eq!(findings.len(), 1, "{findings:?}");
            assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "a.rs");
        }

        #[test]
        fn Test_A_Subject_With_No_Fact_Should_Be_Reported_Rather_Than_Silently_Clean()
        {
            let source = Source_File("a.rs");
            let TestOffering { store, registry, .. } = Offering();

            let mut reader = Reader::On(&store, &registry, Test_Context());
            let findings = Check_Unread_Reaches_A_Finding(&[source], &mut reader);

            assert_eq!(findings.len(), 1, "an unread subject must not render as a clean one: {findings:?}");
        }

        #[test]
        fn Test_Check_Unread_Reaches_A_Finding_Should_Produce_No_Finding_For_A_Source_With_No_Flagged_Sites()
        {
            let source = Source_File("a.rs");
            let TestOffering { mut store, registry, offer } = Offering();
            Materialize_Reachability_Fact(&mut store, &source, &offer, &ReachabilityPayload { sites: Vec::new() });

            let mut reader = Reader::On(&store, &registry, Test_Context());
            let findings = Check_Unread_Reaches_A_Finding(&[source], &mut reader);

            assert!(findings.is_empty(), "{findings:?}");
        }

        fn Offering() -> TestOffering
        {
            return test_support::Offering(
                OfferedProvider {
                    contract: nomos_cap_controlflow::Capability_Contract(),
                    capability: nomos_cap_controlflow::Capability(),
                    version: nomos_cap_controlflow::CONTRACT_VERSION,
                    provider: PROVIDER,
                    guarantee: Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unsound, IncrementalGranularity::File),
                },
            ).expect("a fresh Registry holds neither this contract nor this provider");
        }

        fn Materialize_Reachability_Fact(store: &mut MemoryFactStore, source: &SourceFile, offer: &ProviderOffer, payload: &ReachabilityPayload)
        {
            let bytes = nomos_cap_controlflow::Encode_Payload(payload);
            let inputs = InputDigest::Of(&[source.text.as_bytes()]);
            test_support::Materialize(store, FactToFile { subject: source.subject, offer, semantic_inputs: inputs, schema: nomos_cap_controlflow::Payload_Schema(), bytes }).expect("the fixture's store holds no fact under this key at a newer generation");
        }
    }
}
