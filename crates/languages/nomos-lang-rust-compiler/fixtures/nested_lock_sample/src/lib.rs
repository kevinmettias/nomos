//! A tiny, deliberately minimal crate for `crate::nested_lock_reading`'s tests to point a
//! real `ra_ap_hir` analysis at: one `std::sync::Mutex` guarding a value whose own type is
//! itself already a lock, reachable only by resolving a type alias -- the exact case a
//! syntax-only scan cannot see, since nothing at the nesting site is spelled `Mutex` a
//! second time -- and one plain, unnested `Mutex` that must not fire.

use std::sync::Mutex;

/// Nothing about this name says "this is already a `Mutex`" -- only resolving the alias
/// tells you.
pub type GuardedCounter = Mutex<i32>;

pub struct Registry
{
    /// A real double lock: acquiring this outer `Mutex` still leaves the counter behind
    /// its own, separate lock. Only `ra_ap_hir` resolving `GuardedCounter` back to
    /// `Mutex<i32>` can see it -- the syntax here never repeats the word `Mutex`.
    pub counters: Mutex<GuardedCounter>,

    /// The negative control: a plain, unnested `Mutex` that must not fire.
    pub plain: Mutex<i32>,
}
