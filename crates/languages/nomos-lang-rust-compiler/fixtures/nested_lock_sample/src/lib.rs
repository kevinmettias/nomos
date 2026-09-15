//! A tiny, deliberately minimal crate for `crate::nested_lock_reading`'s tests to point a
//! real `ra_ap_hir` analysis at: one `std::sync::Mutex` guarding a value whose own type is
//! itself already a lock, reachable only by resolving a type alias -- the exact case a
//! syntax-only scan cannot see, since nothing at the nesting site is spelled `Mutex` a
//! second time -- and one plain, unnested `Mutex` that must not fire.
//!
//! The double lock itself lives in `registry.rs`, one public type per file.

use std::sync::Mutex;

/// Nothing about this name says "this is already a `Mutex`" -- only resolving the alias
/// tells you.
pub type GuardedCounter = Mutex<i32>;

mod registry;

pub use registry::Registry;
