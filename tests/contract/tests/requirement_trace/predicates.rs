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

use crate::assessment::Assessment;
use nomos_cap_requirement_trace::Problem;
use nomos_platform_std::StdFileSystem;
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
