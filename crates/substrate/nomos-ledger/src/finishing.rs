//! Who is finishing what.

use crate::ItemId;
/// Who is finishing what.
///
/// The two are consulted together at every step — the item to find the predicate, the holder
/// to prove entitlement to record the result — and a call that named one without the other
/// could not do either.
#[derive(Clone, Copy)]
pub struct Finishing<'a>
{
    /// The item whose predicate is being run.
    pub item: &'a ItemId,
    /// Who claims to hold it.
    pub holder: &'a str,
}
