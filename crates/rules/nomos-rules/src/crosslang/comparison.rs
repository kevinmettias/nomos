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
use std::collections::BTreeSet;

/// One declared correspondence, judged — `None` for a clean match, the same "clean is
/// silent" convention `Check_Lint_Diagnostics` already holds for a tool that already
/// decided.
pub(super) fn Judged(source: &SourceFile, item: &PayloadItem, target_name: &str, index: &[(&SourceFile, SyntaxPayload)]) -> Option<Finding>
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
