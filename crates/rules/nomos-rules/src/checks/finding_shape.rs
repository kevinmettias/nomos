//! The shape a rule's own verdict takes, stated once.
//!
//! Ten rule modules built this by hand before it lived here, and `check-interfile-duplication`
//! reported every one of them: the same `SubjectId` from a digest of the qualified name, the
//! same `Applicability::Supported`, the same `EvidenceClass::Derived`, the same single-element
//! `locations`. Only three things ever varied between them -- how the qualified name is spelled,
//! which name the subject is *called*, and whether the rule blocks or advises -- and each of
//! those is now a parameter rather than a copied paragraph.
//!
//! This is the same judgment `Relay_Findings` and [`crate::checks::test_support`] record for
//! their own plumbing: a rule author with no declared alternative to copying will copy, so the
//! alternative is declared.
//!
//! Nothing here decides anything a rule decides. Which subjects are violations, what the
//! summary says, and which rule is speaking all stay in the rule's own module; this file holds
//! only the four lines that were identical.
//!
//! [`crate::checks::test_support`]: crate::checks::test_support

use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

/// One rule's verdict about one subject, named by an already-computed string.
///
/// The three `*_Finding` functions below are the shapes the ten original sites actually
/// used; this is the one they all reduce to, kept visible because a fourth shape added
/// later should be written here rather than inlined at its first call site.
pub(in crate::checks) fn Subject_Finding(
    rule: &str,
    path: &str,
    subject_name: &str,
    qualified: &str,
    gate: GateCategory,
    summary: String,
) -> Finding
{
    use nomos_model::Content_Digest;

    return Finding {
        rule: RuleId::New(rule),
        subject: SubjectId::From_Digest(Content_Digest(qualified.as_bytes())),
        subject_name: subject_name.to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate,
        summary,
        locations: vec![path.to_owned()],
    };
}

/// A blocking verdict about one named member of `item`.
///
/// The qualified name carries the member, so two members of one type file apart --
/// `{path}::{item}::{member}`.
pub(in crate::checks) fn Member_Finding(
    rule: &str,
    path: &str,
    item: &nomos_cap_syntax::PayloadItem,
    member: &str,
    summary: String,
) -> Finding
{
    let qualified = format!("{path}::{}::{member}", item.qualified_name);

    return Subject_Finding(rule, path, member, &qualified, GateCategory::Blocking, summary);
}

/// A blocking verdict about `item` itself, called by its own declared name.
///
/// [`nomos_cap_syntax::PayloadItem::Own_Name`] rather than `qualified_name`, because a
/// summary that says "`Helper` is not UpperCamelCase" is naming the declaration, not the
/// path it happens to sit at.
pub(in crate::checks) fn Own_Name_Finding(
    rule: &str,
    path: &str,
    item: &nomos_cap_syntax::PayloadItem,
    summary: String,
) -> Finding
{
    let qualified = format!("{path}::{}", item.qualified_name);

    return Subject_Finding(rule, path, &item.Own_Name(), &qualified, GateCategory::Blocking, summary);
}

/// A blocking verdict about `item` itself, called by its fully qualified name.
///
/// The two rules that use this one judge a declaration whose *lookup* name is the point --
/// a test function and a conventionally named function -- so the summary quotes the name a
/// caller would have to write rather than the one the declaration spells.
pub(in crate::checks) fn Qualified_Name_Finding(
    rule: &str,
    path: &str,
    item: &nomos_cap_syntax::PayloadItem,
    gate: GateCategory,
    summary: String,
) -> Finding
{
    let qualified = format!("{path}::{}", item.qualified_name);

    return Subject_Finding(rule, path, &item.qualified_name, &qualified, gate, summary);
}
