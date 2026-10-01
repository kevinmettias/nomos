//! A fact ceasing to be current, and what that reaches.
//!
//! The report is the answer; broadening is the case where the answer is wider than the
//! change that provoked it; supersession is the replacement it records. None of the three
//! is read without the others, which is why they are one module -- the report is declared at
//! the crate root, and `lib.rs` says why where it declares it.

// Where an invalidation had to widen, and the replacement it recorded.
mod broadening;
mod supersession;

pub use broadening::Broadening;
pub use supersession::Supersession;
