//! A tiny, deliberately minimal crate for `crate::reading`'s tests to point a real
//! `ra_ap_hir` analysis at: one `.clone()` call whose receiver resolves to a `Copy` type,
//! and one whose receiver resolves to a type that is `Clone` but not `Copy` -- the exact
//! distinction `Type::is_copy` exists to draw and `syn`'s parse tree has no way to make,
//! since it never resolves a name to the item that declares it.

#[derive(Clone, Copy)]
pub struct Point
{
    pub horizontal: i32,
    pub vertical: i32,
}

#[derive(Clone)]
pub struct Heavy
{
    pub data: Vec<u8>,
}

#[must_use]
pub fn duplicate_point(point: &Point) -> Point
{
    return point.clone();
}

#[must_use]
pub fn duplicate_heavy(heavy: &Heavy) -> Heavy
{
    return heavy.clone();
}
