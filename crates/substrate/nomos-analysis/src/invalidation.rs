//! A fact ceasing to be current, and what that reaches.
//!
//! The report is the answer; broadening is the case where the answer is wider than the
//! change that provoked it; supersession is the replacement it records. None of the three
//! is read without the others, which is why they are one module.

// What an invalidation reached, where it had to widen, and the replacement it recorded.
mod broadening;
#[path = "invalidation/report.rs"] mod report;
mod supersession;

pub use broadening::Broadening;
pub use report::Report as InvalidationReport;
pub use supersession::Supersession;
