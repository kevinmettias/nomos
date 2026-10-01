//! What supports a claim, and what was never looked at.
//!
//! The two halves are here together because a report needs both and neither is
//! recoverable from the other. [`Evidence`] says how the claims a run did make were come
//! by; [`Coverage`] says what the run never reached. A run that produced sound evidence
//! about four subjects out of nine hundred looks, without the second half, exactly like a
//! run that examined all nine hundred.

mod coverage;
mod coverage_gap;
mod evidence;
mod r#ref;

pub use coverage::Coverage;
pub use coverage_gap::CoverageGap;
pub use evidence::Evidence;
pub use r#ref::Ref as EvidenceRef;
