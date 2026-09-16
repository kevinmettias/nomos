//! The fixture's `Clone`-but-not-`Copy` half: a `.clone()` call whose receiver resolves
//! to a type that is `Clone` and not `Copy`, which a real `ra_ap_hir` analysis must leave
//! unreported -- the negative control for the positive case `lib.rs`'s `Duplicate_Point`
//! supplies.

#[derive(Clone)]
pub struct Heavy
{
    pub data: Vec<u8>,
}

#[must_use]
pub fn Duplicate_Heavy(heavy: &Heavy) -> Heavy
{
    return heavy.clone();
}
