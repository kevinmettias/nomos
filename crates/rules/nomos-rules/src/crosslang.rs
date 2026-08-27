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

use crate::{SourceFile, Syntax_Requirement_For};
use nomos_analysis::{FactReader, InputDigest, MaterializedFact};
use nomos_cap_syntax::{PayloadItem, Struct_Fields, SyntaxPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};
use std::collections::BTreeSet;

/// This rule's own identifier.
pub const CROSS_LANGUAGE_CORRESPONDENCE: &str = "cross-language-correspondence";

/// The doc-comment marker a struct declares a correspondence behind — `OD-CAPABILITY-010`'s
/// own choice, the identical shape `nomos-rules::universe`'s `Mirrored by` already uses.
const CORRESPONDS_TO_MARKER: &str = "Corresponds to ";

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
    let index = Struct_Index(sources, facts);
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

            let Some(target_name) = Declared_Correspondence(item.documentation.Value()) else { continue };

            if let Some(finding) = Judged(source, item, &target_name, index)
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

/// Every source's decoded `nomos.cap.syntax.items` payload, for the sources whose fact
/// could be read.
fn Struct_Index<'a>(sources: &'a [SourceFile], facts: &mut dyn FactReader) -> Vec<(&'a SourceFile, SyntaxPayload)>
{
    return sources
        .iter()
        .filter_map(|source| return Payload_Of(source, facts).ok().map(|payload| return (source, payload)))
        .collect();
}

/// This source's decoded syntax payload, or `None` if its fact could not be read under
/// this rule's own floor — the identical `Require`-then-decode shape `nomos_rules::naming::
/// reading::Payload_Of` already uses for the same capability, kept local rather than
/// shared: each rule that reads `nomos.cap.syntax.items` states and discharges its own
/// floor, the same independence `Check_Lint_Diagnostics` and `Check_Dependency_Policy`
/// already keep from each other for their own capabilities.
fn Payload_Of(source: &SourceFile, facts: &mut dyn FactReader) -> Result<SyntaxPayload, ()>
{
    let need = Syntax_Requirement_For(source.preferred_syntax_provider.clone());
    let capability = nomos_cap_syntax::Capability();
    let inputs = InputDigest::Of(&[source.text.as_bytes()]);

    let fact: &MaterializedFact = facts.Require(&capability, &source.subject, inputs, &need).map_err(|_| ())?;

    if fact.payload.schema != nomos_cap_syntax::Payload_Schema()
    {
        return Err(());
    }

    return nomos_cap_syntax::Parse_Payload(&fact.payload.bytes).map_err(|_| ());
}

/// The struct name one `documentation` observation declares a correspondence to, if it
/// declares one — the identical parsing `nomos_rules::universe`'s own `Claimed_Mirror`
/// already uses for `Mirrored by`, applied to this rule's own marker.
fn Declared_Correspondence(documentation: Option<&str>) -> Option<String>
{
    for line in documentation?.lines()
    {
        let (_, after) = line.split_once(CORRESPONDS_TO_MARKER)?;
        let quoted = after.strip_prefix('`')?;
        let (name, _) = quoted.split_once('`')?;

        if !name.trim().is_empty()
        {
            return Some(name.trim().to_owned());
        }
    }

    return None;
}

/// One declared correspondence, judged — `None` for a clean match, the same "clean is
/// silent" convention `Check_Lint_Diagnostics` already holds for a tool that already
/// decided.
fn Judged(source: &SourceFile, item: &PayloadItem, target_name: &str, index: &[(&SourceFile, SyntaxPayload)]) -> Option<Finding>
{
    let Some(own_fields) = Struct_Fields(&item.shape) else {
        return Some(Unparseable(
            source,
            &item.qualified_name,
            &format!("`{}` declares a correspondence to `{target_name}` but has no named fields of its own to compare", item.qualified_name),
        ));
    };

    let Some(target) = Find_Struct(source, item, target_name, index) else {
        return Some(Missing(source, item, target_name));
    };

    let Some(target_fields) = Struct_Fields(&target.shape) else {
        return Some(Unparseable(
            source,
            &item.qualified_name,
            &format!("`{target_name}` has no named fields to compare `{}` against", item.qualified_name),
        ));
    };

    return Drift(source, item, target_name, &FieldSets { own: &own_fields, target: &target_fields });
}

/// The two sides of a declared correspondence's field sets, compared as a pair — grouped so
/// [`Drift`] takes one thing to compare rather than two separate slices.
struct FieldSets<'a>
{
    own: &'a [(String, String)],
    target: &'a [(String, String)],
}

/// The one struct anywhere in `index` named `target_name`, other than `declaring` itself —
/// a correspondence may legitimately name a struct that shares its own qualified name (the
/// common case: a Rust `Wide` and a Go `Wide`), so the search must not let a struct resolve
/// to itself just because two languages happened to spell the same name the same way.
fn Find_Struct<'a>(declaring_source: &SourceFile, declaring: &PayloadItem, target_name: &str, index: &'a [(&SourceFile, SyntaxPayload)]) -> Option<&'a PayloadItem>
{
    for (source, payload) in index
    {
        for item in &payload.items
        {
            if item.kind != "Struct" || item.qualified_name != target_name
            {
                continue;
            }
            if source.subject == declaring_source.subject && item.ordinal == declaring.ordinal
            {
                continue;
            }

            return Some(item);
        }
    }

    return None;
}

fn Missing(source: &SourceFile, item: &PayloadItem, target_name: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(CROSS_LANGUAGE_CORRESPONDENCE),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability: Applicability::MissingCapability,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "`{}` declares a correspondence to `{target_name}`, but no struct named that was found among the sources this run judged",
            item.qualified_name
        ),
        locations: vec![source.path.clone()],
    };
}

fn Unparseable(source: &SourceFile, declaring_name: &str, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(CROSS_LANGUAGE_CORRESPONDENCE),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability: Applicability::Unparseable,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("`{declaring_name}`'s declared correspondence could not be judged: {because}"),
        locations: vec![source.path.clone()],
    };
}

/// The field-name drift between two sides of a declared correspondence, or `None` when
/// both sides name exactly the same fields — arity and order are read as a consequence of
/// the name sets agreeing or not, never compared positionally: `OD-CAPABILITY-010`'s own
/// worked example is a claim about which names exist on each side.
fn Drift(source: &SourceFile, item: &PayloadItem, target_name: &str, fields: &FieldSets<'_>) -> Option<Finding>
{
    let own_names: BTreeSet<&str> = fields.own.iter().map(|(name, _)| return name.as_str()).collect();
    let target_names: BTreeSet<&str> = fields.target.iter().map(|(name, _)| return name.as_str()).collect();

    let missing_on_target: Vec<&str> = own_names.difference(&target_names).copied().collect();
    let missing_on_own: Vec<&str> = target_names.difference(&own_names).copied().collect();

    if missing_on_target.is_empty() && missing_on_own.is_empty()
    {
        return None;
    }

    return Some(Finding {
        rule: RuleId::New(CROSS_LANGUAGE_CORRESPONDENCE),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: Drift_Summary(&item.qualified_name, target_name, &missing_on_target, &missing_on_own),
        locations: vec![source.path.clone()],
    });
}

fn Drift_Summary(declaring_name: &str, target_name: &str, missing_on_target: &[&str], missing_on_own: &[&str]) -> String
{
    let mut parts = Vec::new();

    if !missing_on_target.is_empty()
    {
        parts.push(format!("`{target_name}` has no field for {}", missing_on_target.join(", ")));
    }
    if !missing_on_own.is_empty()
    {
        parts.push(format!("`{declaring_name}` has no field for {}", missing_on_own.join(", ")));
    }

    return format!("`{declaring_name}` and its declared correspondence `{target_name}` disagree on fields: {}", parts.join("; "));
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_analysis::{
        Context, FactKey, FactPayload, GuaranteeDigest, MaterializedFact as Fact, MemoryFactStore, Reader,
    };
    use nomos_capability::{ProviderOffer, Registry};
    use nomos_cap_syntax::{Observation, PUBLIC};
    use nomos_contracts::{
        Assurance, BuildVariantId, ConfigurationId, Digest128, FactVariant, GenerationId, Guarantee, IncrementalGranularity,
        ProviderId, SnapshotId,
    };

    const PROVIDER: &str = "nomos.test.crosslang.resolves";

    fn Source(path: &str) -> SourceFile
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

    fn Materialize(store: &mut MemoryFactStore, source: &SourceFile, offer: &ProviderOffer, payload: &SyntaxPayload)
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
        let owned: Vec<(String, String)> = fields.iter().map(|(n, t)| return ((*n).to_owned(), (*t).to_owned())).collect();
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
        let rust_source = Source("counter.rs");
        let go_source = Source("counter.go");
        let TestOffering { mut store, registry, offer } = Offering();

        Materialize(
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
        Materialize(
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
        let rust_source = Source("wide.rs");
        let go_source = Source("wide.go");
        let TestOffering { mut store, registry, offer } = Offering();

        Materialize(
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
        Materialize(
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
        let source = Source("lonely.rs");
        let TestOffering { mut store, registry, offer } = Offering();

        Materialize(
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
        let rust_source = Source("wrapper.rs");
        let go_source = Source("marker.go");
        let TestOffering { mut store, registry, offer } = Offering();

        Materialize(
            &mut store,
            &rust_source,
            &offer,
            &SyntaxPayload {
                unexpanded: 0,
                items: vec![Struct_Item(0, "Wrapper", Observation::Present("Corresponds to `Marker`.".to_owned()), &[("x", "u32")])],
            },
        );
        Materialize(
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
        let source = Source("plain.rs");
        let TestOffering { mut store, registry, offer } = Offering();

        Materialize(
            &mut store,
            &source,
            &offer,
            &SyntaxPayload { unexpanded: 0, items: vec![Struct_Item(0, "Plain", Observation::Absent, &[("x", "u32")])] },
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Cross_Language_Correspondence(&[source], &mut reader);

        assert!(findings.is_empty());
    }

    #[test]
    fn Test_Declared_Correspondence_Requires_A_Nonempty_Backticked_Name()
    {
        assert_eq!(Declared_Correspondence(Some("Corresponds to `Counter`.")), Some("Counter".to_owned()));
        assert_eq!(Declared_Correspondence(Some("Corresponds to nothing in particular.")), None);
        assert_eq!(Declared_Correspondence(Some("Corresponds to ``.")), None);
        assert_eq!(Declared_Correspondence(None), None);
    }
}
