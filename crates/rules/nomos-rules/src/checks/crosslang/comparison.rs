//! Judging one declared correspondence against whichever struct its name resolves to —
//! field-name and arity drift, set-wise, the actual comparison `OD-CAPABILITY-010`'s
//! worked example describes.
//!
//! A pure function of an already-built index and an already-parsed declared name, grouped
//! apart from `reading.rs` (how the index gets built) and `declaration.rs` (how a name gets
//! parsed out of a doc comment) for the same reason `crate::dependency::violations` already
//! is: testable against hand-built fixtures, no registry, no store, no reader.

use super::CROSS_LANGUAGE_CORRESPONDENCE;
use crate::SourceFile;
use nomos_cap_syntax::{PayloadItem, Struct_Fields, SyntaxPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// One declared correspondence, judged — `None` for a clean match, the same "clean is
/// silent" convention `Check_Lint_Diagnostics` already holds for a tool that already
/// decided.
pub(super) fn Judged_Correspondence(source: &SourceFile, item: &PayloadItem, target_name: &str, index: &[(&SourceFile, SyntaxPayload)]) -> Option<Finding>
{
    let Some(own_fields) = Struct_Fields(&item.shape) else {
        return Some(Unparseable_Finding(
            source,
            DeclaringName(&item.qualified_name),
            Reason(&format!("`{}` declares a correspondence to `{target_name}` but has no named fields of its own to compare", item.qualified_name)),
        ));
    };

    let Some(target) = Find_Struct(source, item, target_name, index) else {
        return Some(Missing_Finding(source, item, target_name));
    };

    let Some(target_fields) = Struct_Fields(&target.shape) else {
        return Some(Unparseable_Finding(
            source,
            DeclaringName(&item.qualified_name),
            Reason(&format!("`{target_name}` has no named fields to compare `{}` against", item.qualified_name)),
        ));
    };

    return Field_Drift(source, item, target_name, &FieldSets { own: &own_fields, target: &target_fields });
}

/// The two sides of a declared correspondence's field sets, compared as a pair — grouped so
/// [`Drift`] takes one thing to compare rather than two separate slices.
struct FieldSets<'a>
{
    own: &'a [(String, String)],
    target: &'a [(String, String)],
}

/// The struct declaring a correspondence, by its own qualified name -- named so a caller
/// cannot transpose it with the struct it names a correspondence to, since both are
/// otherwise identically-shaped `&str`s.
struct DeclaringName<'a>(&'a str);

/// Why a correspondence could not be judged -- named so [`Unparseable`] cannot mistake it
/// for the struct name beside it.
struct Reason<'a>(&'a str);

fn Unparseable_Finding(source: &SourceFile, declaring_name: DeclaringName<'_>, because: Reason<'_>) -> Finding
{
    return Finding {
        rule: RuleId::New(CROSS_LANGUAGE_CORRESPONDENCE),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability: Applicability::Unparseable,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("`{}`'s declared correspondence could not be judged: {}", declaring_name.0, because.0),
        locations: vec![source.path.clone()],
    };
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

fn Missing_Finding(source: &SourceFile, item: &PayloadItem, target_name: &str) -> Finding
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

/// The field-name drift between two sides of a declared correspondence, or `None` when
/// both sides name exactly the same fields — arity and order are read as a consequence of
/// the name sets agreeing or not, never compared positionally: `OD-CAPABILITY-010`'s own
/// worked example is a claim about which names exist on each side.
fn Field_Drift(source: &SourceFile, item: &PayloadItem, target_name: &str, fields: &FieldSets<'_>) -> Option<Finding>
{
    let missing = Missing_Field_Names(fields);

    if missing.on_target.is_empty() && missing.on_own.is_empty()
    {
        return None;
    }

    return Some(Drift_Finding(source, item, target_name, &missing));
}

/// The two sides of a field-name-drift check's own verdict — which names `fields.own`
/// lacks that `fields.target` declares, and which names `fields.target` lacks that
/// `fields.own` declares. A named pair rather than a tuple: both sides are the same type,
/// and a tuple would let a caller swap them without the compiler noticing.
struct MissingFields<'a>
{
    on_target: Vec<&'a str>,
    on_own: Vec<&'a str>,
}

/// The field names present on `fields.own` but absent from `fields.target`, and the field
/// names present on `fields.target` but absent from `fields.own` — set-wise, never
/// positional, the reasoning [`Field_Drift`]'s own doc names.
fn Missing_Field_Names<'a>(fields: &FieldSets<'a>) -> MissingFields<'a>
{
    use std::collections::BTreeSet;

    let own_names: BTreeSet<&str> = fields.own.iter().map(|(name, _)| return name.as_str()).collect();
    let target_names: BTreeSet<&str> = fields.target.iter().map(|(name, _)| return name.as_str()).collect();

    let on_target: Vec<&str> = own_names.difference(&target_names).copied().collect();
    let on_own: Vec<&str> = target_names.difference(&own_names).copied().collect();

    return MissingFields { on_target, on_own };
}

/// The [`Finding`] for a declared correspondence whose two sides' field names disagree —
/// factored out of [`Field_Drift`] so that function reads as compute-then-report rather
/// than one longer body.
fn Drift_Finding(source: &SourceFile, item: &PayloadItem, target_name: &str, missing: &MissingFields<'_>) -> Finding
{
    return Finding {
        rule: RuleId::New(CROSS_LANGUAGE_CORRESPONDENCE),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: Drift_Summary(DeclaringName(&item.qualified_name), TargetName(target_name), &missing.on_target, &missing.on_own),
        locations: vec![source.path.clone()],
    };
}

/// The struct a correspondence names, on the other side from [`DeclaringName`] -- its own
/// type for the identical reason.
struct TargetName<'a>(&'a str);

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_cap_syntax::{Observation, PUBLIC};
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Judged_Correspondence_Should_Report_Nothing_When_Both_Sides_Name_The_Same_Fields()
    {
        let source = Source("counter.rs");
        let item = Struct_Item("Counter", &[("n", "u32")]);
        let target_source = Source("counter.go");
        let index = vec![(&target_source, SyntaxPayload { unexpanded: 0, items: vec![Struct_Item("Counter", &[("n", "int")])] })];

        let judged = Judged_Correspondence(&source, &item, "Counter", &index);

        assert!(judged.is_none(), "{judged:?}");
    }

    #[test]
    fn Test_Judged_Correspondence_Should_Report_A_Missing_Field_On_One_Side()
    {
        let source = Source("wide.rs");
        let item = Struct_Item("Wide", &[("a", "u32"), ("b", "u32")]);
        let target_source = Source("wide.go");
        let index = vec![(&target_source, SyntaxPayload { unexpanded: 0, items: vec![Struct_Item("Wide", &[("a", "int")])] })];

        let judged = Judged_Correspondence(&source, &item, "Wide", &index).expect("the sides disagree on fields");

        assert_eq!(judged.applicability, Applicability::Supported);
        assert!(judged.summary.contains('b'), "{}", judged.summary);
    }

    fn Struct_Item(qualified_name: &str, fields: &[(&str, &str)]) -> PayloadItem
    {
        let owned: Vec<(String, String)> = fields.iter().map(|(name, kind)| return ((*name).to_owned(), (*kind).to_owned())).collect();
        let shape = nomos_cap_syntax::Struct_Shape(&owned).map_or(Observation::Absent, Observation::Present);

        return PayloadItem {
            ordinal: 0,
            kind: "Struct".to_owned(),
            visibility: PUBLIC.to_owned(),
            qualified_name: qualified_name.to_owned(),
            documentation: Observation::Absent,
            shape,
        };
    }

    fn Source(path: &str) -> SourceFile
    {
        return SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), String::new());
    }
}

fn Drift_Summary(declaring_name: DeclaringName<'_>, target_name: TargetName<'_>, missing_on_target: &[&str], missing_on_own: &[&str]) -> String
{
    let declaring_name = declaring_name.0;
    let target_name = target_name.0;
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
