//! A tiny, deliberately minimal crate for `crate::reading`'s tests to point a real
//! `ra_ap_hir` analysis at: one `.clone()` call whose receiver resolves to a `Copy` type,
//! and one whose receiver resolves to a type that is `Clone` but not `Copy` -- the exact
//! distinction `Type::is_copy` exists to draw and `syn`'s parse tree has no way to make,
//! since it never resolves a name to the item that declares it.
//!
//! The `Clone`-but-not-`Copy` half lives in `heavy.rs`, one public type per file.

mod heavy;

pub use heavy::{Heavy, Duplicate_Heavy};

#[derive(Clone, Copy)]
pub struct Point
{
    pub horizontal: i32,
    pub vertical: i32,
}

#[must_use]
pub fn Duplicate_Point(point: &Point) -> Point
{
    return point.clone();
}
