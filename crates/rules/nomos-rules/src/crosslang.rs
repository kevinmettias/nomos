//! The workspace's own claim that two providers' structs share a wire shape, checked
//! against a declared correspondence rather than assumed from proximity in the store.
//!
//! `OD-CAPABILITY-010` decided the shape: a Rust struct's doc comment names a Go
//! counterpart by qualified name (``/// Corresponds to `Name`.``, the identical marker
//! shape `nomos-rules::universe`'s own `Mirrored by` already uses for a declared mirror),
//! and this rule reads both sides' already-materialized `nomos.cap.syntax.items` facts and
//! compares their field names and arity, set-wise. No new capability: both sides are the
//! same fact, read twice, over a subject pair a declared correspondence names rather than
//! a subject the walk handed this rule directly — the same "the party that can see both
//! sides is the only party that can ask" reasoning `OD-CAPABILITY-006` already gives.
//!
//! # Composed into `nomos-check-orchestration::Run`
//!
//! Unconditionally, the same shape a fourth rule already took (`OD-HOST-004`) and behind
//! `OD-GATE-017`'s own `Wants(selected, ...)` gate every rule sits behind — wiring this
//! rule in is `nomos-check-orchestration`'s own territory, not this crate's. No new
//! materialization: every source's `nomos.cap.syntax.items` fact is already written by
//! `Materialize_Syntax` before any rule runs.
//!
//! # Split by responsibility
//!
//! [`reading`] requires and decodes every source's own syntax fact into the index this
//! judgment reads. [`declaration`] parses the doc-comment marker a struct declares a
//! correspondence behind, `OD-CAPABILITY-010`'s own textual convention. [`comparison`]
//! judges one declared correspondence against whichever struct its name resolves to. This
//! file keeps only what composes the three: the rule's own identifier and
//! [`Check_Cross_Language_Correspondence`] itself, plus the end-to-end tests that exercise
//! all three together through a real reader — the same split `dependency.rs` and
//! `naming.rs` already draw for their own rules.

mod comparison;
mod declaration;
mod reading;

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_cap_syntax::SyntaxPayload;
use nomos_contracts::Finding;

/// This rule's own identifier.
pub const CROSS_LANGUAGE_CORRESPONDENCE: &str = "cross-language-correspondence";

/// Judges every struct among `sources` that declares a correspondence, against whichever
/// other struct among `sources` its declared name resolves to.
///
/// Reads every source's `nomos.cap.syntax.items` fact once, up front — a source whose fact
/// could not be read is silently absent from the index rather than reported here: this
/// rule's own claim is about a declared pair, not about whether every file in `sources` is
/// individually readable, which `nomos-rules::{Check_Completeness_Mirrors, Check_Naming_
/// Convention}` already report on. A pair whose target genuinely cannot be found — whether
/// because nothing declares it or because its own fact failed to read — is [`Applicability
/// ::MissingCapability`] either way: this rule cannot tell those two absences apart, since
/// an unread source and an absent one look identical from here, and `OD-CAPABILITY-006`'s
/// own text is that a missing side is reported the same regardless of why.
#[must_use]
pub fn Check_Cross_Language_Correspondence(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let index = reading::Struct_Index(sources, facts);
    let mut findings = Findings_Over_Index(&index);

    Sort_Findings(&mut findings);
    return findings;
}

/// Every struct among `index`'s own entries that declares a correspondence, judged against
/// whichever other struct among `index` its declared name resolves to — the middle section
/// of [`Check_Cross_Language_Correspondence`], factored out on its own so that function reads
/// as index, judge, sort rather than one longer body.
fn Findings_Over_Index(index: &[(&SourceFile, SyntaxPayload)]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for (source, payload) in index
    {
        for item in &payload.items
        {
            if item.kind != "Struct"
            {
                continue;
            }

            let Some(target_name) = declaration::Declared_Correspondence(item.documentation.Value()) else { continue };

            if let Some(finding) = comparison::Judged_Correspondence(source, item, &target_name, index)
            {
                findings.push(finding);
            }
        }
    }

    return findings;
}

fn Sort_Findings(findings: &mut [Finding])
{
    findings.sort_by(|left, right| return (&left.subject_name, &left.summary).cmp(&(&right.subject_name, &right.summary)));
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_analysis::{
        Context, FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact as Fact, MemoryFactStore, Reader,
    };
    use nomos_capability::{ProviderOffer, Registry};
    use nomos_cap_syntax::{Observation, PayloadItem, PUBLIC};
    use nomos_contracts::{
        Applicability, Assurance, BuildVariantId, ConfigurationId, Digest128, EvidenceClass, FactVariant, GenerationId,
        Guarantee, IncrementalGranularity, ProviderId, SnapshotId,
    };

    const PROVIDER: &str = "nomos.test.crosslang.resolves";

    fn Source_File(path: &str) -> SourceFile
    {
        return SourceFile::New(path, nomos_model::Subject_Of_Path(path), String::new());
    }

    fn Guarantee_At_Floor() -> Guarantee
    {
        return Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File);
    }

    fn Test_Context() -> Context
    {
        return Context {
            snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; 16])),
            variant: BuildVariantId::From_Digest(Digest128::From_Bytes([2; 16])),
            configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([3; 16])),
            generation: GenerationId::INITIAL,
        };
    }

    struct TestOffering
    {
        store: MemoryFactStore,
        registry: Registry,
        offer: ProviderOffer,
    }

    fn Offering() -> TestOffering
    {
        let mut registry = Registry::New();
        registry.Declare(nomos_cap_syntax::Capability_Contract()).expect("declared once");

        let offer = ProviderOffer {
            provider: ProviderId::New(PROVIDER),
            capability: nomos_cap_syntax::Capability(),
            version: nomos_cap_syntax::CONTRACT_VERSION,
            guarantee: Guarantee_At_Floor(),
        };
        registry.Offer(offer.clone()).expect("within the ceiling");

        return TestOffering { store: MemoryFactStore::New(), registry, offer };
    }

    fn Materialize_Syntax_Fact(store: &mut MemoryFactStore, source: &SourceFile, offer: &ProviderOffer, payload: &SyntaxPayload)
    {
        let context = Test_Context();
        let bytes = nomos_cap_syntax::Render_Payload(payload);
        let key = FactKey {
            contract: nomos_cap_syntax::Capability(),
            contract_version: offer.version,
            subject: source.subject,
            semantic_inputs: InputDigest::Of(&[source.text.as_bytes()]),
            provider: offer.provider.clone(),
            provider_version: offer.version,
            guarantee: GuaranteeDigest::Of(&offer.guarantee),
            variant: context.variant,
            configuration: context.configuration,
        };

        store
            .Materialize(
                Fact {
                    identity: key.At(context.generation),
                    snapshot: context.snapshot,
                    evidence: EvidenceClass::Verified,
                    guarantee: offer.guarantee,
                    payload: FactPayload::New(nomos_cap_syntax::Payload_Schema(), bytes),
                },
                &[],
            )
            .expect("nothing here is backdated");
    }

    fn Struct_Item(ordinal: u32, qualified_name: &str, documentation: Observation, fields: &[(&str, &str)]) -> PayloadItem
    {
        let owned: Vec<(String, String)> = fields.iter().map(|(field_name, t)| return ((*field_name).to_owned(), (*t).to_owned())).collect();
        let shape = nomos_cap_syntax::Struct_Shape(&owned).map_or(Observation::Absent, Observation::Present);

        return PayloadItem {
            ordinal,
            kind: "Struct".to_owned(),
            visibility: PUBLIC.to_owned(),
            qualified_name: qualified_name.to_owned(),
            documentation,
            shape,
        };
    }

    #[test]
    fn Test_Two_Structs_With_The_Same_Fields_Should_Produce_No_Finding()
    {
        let rust_source = Source_File("counter.rs");
        let go_source = Source_File("counter.go");
        let TestOffering { mut store, registry, offer } = Offering();

        Materialize_Syntax_Fact(
            &mut store,
            &rust_source,
            &offer,
            &SyntaxPayload {
                unexpanded: 0,
                items: vec![Struct_Item(
                    0,
                    "Counter",
                    Observation::Present("Corresponds to `Counter`.".to_owned()),
                    &[("n", "u32")],
                )],
            },
        );
        Materialize_Syntax_Fact(
            &mut store,
            &go_source,
            &offer,
            &SyntaxPayload { unexpanded: 0, items: vec![Struct_Item(0, "Counter", Observation::Absent, &[("n", "int")])] },
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Cross_Language_Correspondence(&[rust_source, go_source], &mut reader);

        assert!(findings.is_empty(), "same field names must not report a finding: {findings:?}");
    }

    #[test]
    fn Test_A_Missing_Field_On_One_Side_Should_Be_Reported()
    {
        let rust_source = Source_File("wide.rs");
        let go_source = Source_File("wide.go");
        let TestOffering { mut store, registry, offer } = Offering();

        Materialize_Syntax_Fact(
            &mut store,
            &rust_source,
            &offer,
            &SyntaxPayload {
                unexpanded: 0,
                items: vec![Struct_Item(
                    0,
                    "Wide",
                    Observation::Present("Corresponds to `Wide`.".to_owned()),
                    &[("a", "u32"), ("b", "u32")],
                )],
            },
        );
        Materialize_Syntax_Fact(
            &mut store,
            &go_source,
            &offer,
            &SyntaxPayload { unexpanded: 0, items: vec![Struct_Item(0, "Wide", Observation::Absent, &[("a", "int")])] },
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Cross_Language_Correspondence(&[rust_source, go_source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.applicability, Applicability::Supported);
        assert!(found.summary.contains('b'), "{}", found.summary);
    }

    #[test]
    fn Test_A_Correspondence_Naming_Nothing_Should_Report_Missing_Capability()
    {
        let source = Source_File("lonely.rs");
        let TestOffering { mut store, registry, offer } = Offering();

        Materialize_Syntax_Fact(
            &mut store,
            &source,
            &offer,
            &SyntaxPayload {
                unexpanded: 0,
                items: vec![Struct_Item(0, "Lonely", Observation::Present("Corresponds to `Nowhere`.".to_owned()), &[("x", "u32")])],
            },
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Cross_Language_Correspondence(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").applicability, Applicability::MissingCapability);
    }

    #[test]
    fn Test_A_Target_With_No_Fields_Should_Report_Unparseable()
    {
        let rust_source = Source_File("wrapper.rs");
        let go_source = Source_File("marker.go");
        let TestOffering { mut store, registry, offer } = Offering();

        Materialize_Syntax_Fact(
            &mut store,
            &rust_source,
            &offer,
            &SyntaxPayload {
                unexpanded: 0,
                items: vec![Struct_Item(0, "Wrapper", Observation::Present("Corresponds to `Marker`.".to_owned()), &[("x", "u32")])],
            },
        );
        Materialize_Syntax_Fact(
            &mut store,
            &go_source,
            &offer,
            &SyntaxPayload { unexpanded: 0, items: vec![Struct_Item(0, "Marker", Observation::Absent, &[])] },
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Cross_Language_Correspondence(&[rust_source, go_source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").applicability, Applicability::Unparseable);
    }

    #[test]
    fn Test_A_Struct_With_No_Correspondence_Should_Produce_No_Finding()
    {
        let source = Source_File("plain.rs");
        let TestOffering { mut store, registry, offer } = Offering();

        Materialize_Syntax_Fact(
            &mut store,
            &source,
            &offer,
            &SyntaxPayload { unexpanded: 0, items: vec![Struct_Item(0, "Plain", Observation::Absent, &[("x", "u32")])] },
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Cross_Language_Correspondence(&[source], &mut reader);

        assert!(findings.is_empty());
    }
}
