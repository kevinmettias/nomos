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

/// The record this implementation's contract is written in -- `OD-CAPABILITY-010` decided
/// the doc-comment-declared-correspondence shape this rule judges.
/// `tests/contract/tests/rule_contract_citation.rs` reads `OD-CAPABILITY-010`'s own front
/// matter on every run and compares it against
/// [`CROSS_LANGUAGE_CONTRACT_RECORD_VERSION`], the same `nomos_rules::DEPENDENCY_CONTRACT_
/// RECORD` shape, so an amendment this implementation has not caught up to is a red test
/// rather than silent drift.
pub const CROSS_LANGUAGE_CONTRACT_RECORD: &str = "OD-CAPABILITY-010";

/// The version of [`CROSS_LANGUAGE_CONTRACT_RECORD`] this implementation was written
/// against.
pub const CROSS_LANGUAGE_CONTRACT_RECORD_VERSION: u32 = 1;

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
    use crate::checks::test_support::{self, Test_Context, TestOffering};
    use nomos_analysis::{InputDigest, MemoryFactStore, Reader};
    use nomos_capability::ProviderOffer;
    use nomos_cap_syntax::{Observation, PayloadItem, PUBLIC};
    use nomos_contracts::{Applicability, Assurance, FactVariant, Guarantee, IncrementalGranularity};

    const PROVIDER: &str = "nomos.test.crosslang.resolves";

    #[test]
    fn Test_Check_Cross_Language_Correspondence_Should_Produce_No_Finding_For_Matching_Structs()
    {
        let rust_source = Source_File("counter.rs");
        let go_source = Source_File("counter.go");
        let TestOffering { mut store, registry, offer } = Offering();
        let rust_item = Struct_Item(0, "Counter", Observation::Present("Corresponds to `Counter`.".to_owned()), &[("n", "u32")]);
        let go_item = Struct_Item(0, "Counter", Observation::Absent, &[("n", "int")]);

        Materialize_One_Struct(&mut store, &rust_source, &offer, rust_item);
        Materialize_One_Struct(&mut store, &go_source, &offer, go_item);

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Cross_Language_Correspondence(&[rust_source, go_source], &mut reader);

        assert!(findings.is_empty(), "same field names must not report a finding: {findings:?}");
    }

    #[test]
    fn Test_Judged_Correspondence_Should_Report_A_Missing_Field_On_One_Side()
    {
        let rust_source = Source_File("wide.rs");
        let go_source = Source_File("wide.go");
        let TestOffering { mut store, registry, offer } = Offering();
        let rust_item = Struct_Item(0, "Wide", Observation::Present("Corresponds to `Wide`.".to_owned()), &[("a", "u32"), ("b", "u32")]);
        let go_item = Struct_Item(0, "Wide", Observation::Absent, &[("a", "int")]);

        Materialize_One_Struct(&mut store, &rust_source, &offer, rust_item);
        Materialize_One_Struct(&mut store, &go_source, &offer, go_item);

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
        let rust_item = Struct_Item(0, "Wrapper", Observation::Present("Corresponds to `Marker`.".to_owned()), &[("x", "u32")]);
        let go_item = Struct_Item(0, "Marker", Observation::Absent, &[]);

        Materialize_One_Struct(&mut store, &rust_source, &offer, rust_item);
        Materialize_One_Struct(&mut store, &go_source, &offer, go_item);

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Cross_Language_Correspondence(&[rust_source, go_source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").applicability, Applicability::Unparseable);
    }

    /// Also [`reading::Struct_Index`]'s own shape: a struct with no correspondence still
    /// takes its place in the index, and the index alone must not manufacture a finding.
    #[test]
    fn Test_Struct_Index_Should_Admit_A_Struct_That_Declares_No_Correspondence()
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

    /// Materializes a `nomos_cap_syntax` fact holding exactly `item` — the one-item fact every
    /// correspondence test below needs on each of its two sides.
    fn Materialize_One_Struct(store: &mut MemoryFactStore, source: &SourceFile, offer: &ProviderOffer, item: PayloadItem)
    {
        Materialize_Syntax_Fact(store, source, offer, &SyntaxPayload { unexpanded: 0, items: vec![item] });
    }

    fn Source_File(path: &str) -> SourceFile
    {
        return SourceFile::New(path, nomos_model::Subject_Of_Path(path), String::new());
    }

    fn Guarantee_At_Floor() -> Guarantee
    {
        return Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File);
    }

    fn Offering() -> TestOffering
    {
        return test_support::Offering(
            nomos_cap_syntax::Capability_Contract(),
            nomos_cap_syntax::Capability(),
            nomos_cap_syntax::CONTRACT_VERSION,
            PROVIDER,
            Guarantee_At_Floor(),
        );
    }

    fn Materialize_Syntax_Fact(store: &mut MemoryFactStore, source: &SourceFile, offer: &ProviderOffer, payload: &SyntaxPayload)
    {
        let bytes = nomos_cap_syntax::Render_Payload(payload);
        let inputs = InputDigest::Of(&[source.text.as_bytes()]);
        test_support::Materialize(store, source.subject, offer, inputs, nomos_cap_syntax::Payload_Schema(), bytes);
    }
}
