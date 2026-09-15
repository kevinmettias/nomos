//! The two predicates that judge an already-parsed set on its own: a `Partial` entry that
//! names no gap, and a verdict that owes a governing record and names none.
//!
//! Each is called by [`crate::provider::Discover_Workspace`] and by a test control -- held
//! apart from both so the control and the provider run the same code. A control that
//! re-implemented the comparison would prove the copy right and say nothing about the guard.
//!
//! Ported from `tests/contract/tests/requirement_trace/predicates.rs`, where they lived
//! beside the three predicates that resolve a citation against the real tree — those now
//! carry a [`nomos_platform::FileSystem`] and live in this module's own [`unresolved`]
//! submodule; these two need nothing but the parsed [`Assessment`]s, which is why they are
//! the ones this file declares directly.
//!
//! Both assert an obligation [`crate::registry::Parse`] already refuses at read time, kept
//! and asserted separately for its own reason: this crate compares an already-parsed set as
//! well as parsing it, so an obligation the reader enforces is also asserted here rather
//! than trusted to have been enforced.

mod unresolved;

pub use unresolved::{Unresolved_Gaps, Unresolved_Records, Unresolved_Sites};

use crate::assessment::{Assessment, Verdict};
use crate::payload::{Problem, ProblemKind};

/// Every `Partial` entry that names no gap.
#[must_use]
pub fn Partials_With_No_Gap(assessments: &[Assessment]) -> Vec<Problem>
{
    return assessments
        .iter()
        .filter(|assessment| return assessment.verdict == Verdict::Partial && assessment.gaps.is_empty())
        .map(|assessment| {
            return Problem {
                kind: ProblemKind::PartialWithNoGap,
                requirement: assessment.requirement.clone(),
                message: format!("{}: Partial with no gap", assessment.requirement),
            };
        })
        .collect();
}

/// Every entry that departs from a requirement without saying why.
#[must_use]
pub fn Divergences_With_No_Record(assessments: &[Assessment]) -> Vec<Problem>
{
    return assessments
        .iter()
        .filter(|assessment| return assessment.verdict.Owes_A_Record() && assessment.record.is_none())
        .map(|assessment| {
            return Problem {
                kind: ProblemKind::DivergenceWithNoRecord,
                requirement: assessment.requirement.clone(),
                message: format!("{}: {} with no governing record", assessment.requirement, assessment.verdict.Label()),
            };
        })
        .collect();
}
