//! What a read of a file, or a fact materialized from it, can come back as.
//!
//! [`Recognition`] decides whether this provider looks at a file at all. [`Reading`] is
//! what parsing one it recognized came back as. [`Materialization`] is what asking for a
//! fact about it came back as, and mirrors `Reading` deliberately -- both share
//! [`ParseFailure`] as their unparseable variant, because the two functions fail for the
//! identical reason at the identical byte.

mod materialization;
mod parse_failure;
mod reading;
mod recognition;

pub use materialization::Materialization;
pub use parse_failure::ParseFailure;
pub use reading::Reading;
pub use recognition::{Recognition, RUST_EXTENSION};
