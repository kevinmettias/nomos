//! The fixture's real double lock: an outer `Mutex` whose type argument is the type alias
//! `crate::GuardedCounter`, resolving back to a second `Mutex<i32>` -- plus the plain,
//! unnested `Mutex` that must not fire.

use crate::GuardedCounter;
use std::sync::Mutex;

pub struct Registry
{
    /// A real double lock: acquiring this outer `Mutex` still leaves the counter behind
    /// its own, separate lock. Only `ra_ap_hir` resolving `GuardedCounter` back to
    /// `Mutex<i32>` can see it -- the syntax here never repeats the word `Mutex`.
    pub counters: Mutex<GuardedCounter>,

    /// The negative control: a plain, unnested `Mutex` that must not fire.
    pub plain: Mutex<i32>,
}
