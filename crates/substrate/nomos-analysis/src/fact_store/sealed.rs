//! The marker that seals [`super::FactStore`].

/// Implemented only inside this crate, so [`super::FactStore`]'s invariants stay this
/// crate's to keep.
///
/// Split out of `fact_store.rs` so the file is named for the type it declares. The trait is
/// `pub` inside a `pub(crate)` module and so reaches no external caller -- it is absent
/// from `tests/contract/surface/nomos-analysis.txt` for exactly that reason -- but
/// `file-name-matches-declared-type` reads the visibility written on the item rather than
/// the one the module path leaves it with, and a file of its own is a truthful answer that
/// costs less than teaching the rule to resolve effective visibility.
pub trait Sealed
{}
