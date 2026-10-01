//! The predicates. Each is called by an assertion and by a control.
//!
//! `P42-REQUIREMENT-TRACE-STALENESS-RULE-2` promoted these five comparisons into
//! `nomos_cap_requirement_trace::predicates` (the crate a real `nomos-rules` rule's own
//! provider now runs them through) — this module threads the real disk
//! ([`nomos_platform_std::StdFileSystem`]) through the three that read it, rather than
//! reimplementing any of the five. Held apart from both the assertion and the control so
//! they run the same code: a control that re-implemented the comparison would prove the
//! copy right and say nothing about the guard.
//!
//! Each predicate's own result changed shape alongside the promotion: a
//! [`nomos_cap_requirement_trace::Problem`] rather than a pre-rendered `String`, since a
//! real rule needs the structured `kind`/`requirement` a `Finding` is built from and not
//! only a sentence. Every call site in this suite only ever asked `.len()` or
//! `.is_empty()` of what these five return, so the promotion changes no assertion here.
//!
//! # Why the sixth did not go with them
//!
//! [`Unresolved_Rules`] is the one comparison that stays written here rather than wrapping
//! something promoted. `OD-HOST-015` decided why and it is the dependency lattice, not a
//! preference: `nomos-architecture.json` grants `Capability Contract` only `Protocol` and
//! `Substrate`, so `nomos-cap-requirement-trace` cannot name `nomos-rules` and cannot
//! compare a declared rule identifier against [`nomos_rules::DESCRIPTORS`]. `Verification`
//! may name everything, and this suite is above both crates, so this is the only band where
//! the comparison is possible at all. Its consequence is stated rather than hidden: a
//! dangling `rule` line reddens this suite and does **not** redden `nomos check`, which is
//! a narrower promise than the five above make.
//!
//! Its result is a rendered `String` rather than a `Problem` for the same reason: a
//! `Problem` carries a [`nomos_cap_requirement_trace::ProblemKind`], and `OD-HOST-015`
//! refused a sixth kind because the provider that would have to report it cannot see the
//! rule table either.

use crate::assessment::Assessment;
use nomos_cap_requirement_trace::Problem;
use nomos_platform_std::StdFileSystem;
use std::collections::BTreeSet;
use std::path::Path;

/// Every site that is not where its entry says it is.
pub(crate) fn Unresolved_Sites(root: &Path, assessments: &[Assessment]) -> Vec<Problem>
{
    return nomos_cap_requirement_trace::Unresolved_Sites(root, assessments, &StdFileSystem);
}

/// Every `Partial` gap that is not where its entry says it is.
pub(crate) fn Unresolved_Gaps(root: &Path, assessments: &[Assessment]) -> Vec<Problem>
{
    return nomos_cap_requirement_trace::Unresolved_Gaps(root, assessments, &StdFileSystem);
}

/// Every named record that is not a registered governing record.
pub(crate) fn Unresolved_Records(root: &Path, assessments: &[Assessment]) -> Vec<Problem>
{
    return nomos_cap_requirement_trace::Unresolved_Records(root, assessments, &StdFileSystem);
}

/// Every `Partial` entry that names no gap.
pub(crate) fn Partials_With_No_Gap(assessments: &[Assessment]) -> Vec<Problem>
{
    return nomos_cap_requirement_trace::Partials_With_No_Gap(assessments);
}

/// Every entry that departs from a requirement without saying why.
pub(crate) fn Divergences_With_No_Record(assessments: &[Assessment]) -> Vec<Problem>
{
    return nomos_cap_requirement_trace::Divergences_With_No_Record(assessments);
}

/// Every declared `rule` identifier that names no rule this build composes.
///
/// The declared universe is the set of `rule` lines and the reality it claims to enumerate
/// is [`nomos_rules::DESCRIPTORS`] — `OD-COMPLETENESS-001`'s obligation, discharged where
/// both can be seen at once. A typo, a renamed rule and a deleted one all read the same way
/// here, and the repair is the assessment rather than the rule, the direction a vanished
/// `site` already takes.
///
/// Quantifies over declared lines only. An entry with no line owes nothing: absence means
/// nobody declared one, never that no rule bears.
pub(crate) fn Unresolved_Rules(assessments: &[Assessment]) -> Vec<String>
{
    let composed: BTreeSet<&str> = nomos_rules::DESCRIPTORS
        .iter()
        .map(|descriptor| return descriptor.id)
        .collect();

    return assessments
        .iter()
        .flat_map(|assessment| return Dangling_In(assessment, &composed))
        .collect();
}

/// Every rule `assessment` declares that `composed` does not hold, named beside the entry
/// that declared it — both halves, because a bare identifier does not say which entry to
/// repair.
fn Dangling_In(assessment: &Assessment, composed: &BTreeSet<&str>) -> Vec<String>
{
    return assessment
        .rules
        .iter()
        .filter(|rule| return !composed.contains(rule.As_Str()))
        .map(|rule| {
            return format!(
                "{} declares rule `{rule}`, which this build does not compose",
                assessment.requirement
            );
        })
        .collect();
}
