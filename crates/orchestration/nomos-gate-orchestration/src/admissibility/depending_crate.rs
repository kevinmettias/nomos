//! The crate that would do the naming.

/// The crate that would do the naming: a newtype rather than the bare `&str` it wraps, because
/// [`crate::Admits_Edge`] takes this beside a [`crate::DependedCrate`] and both are strings, so two
/// positions of one type would let a caller describe the reverse edge and nothing would say so.
pub struct DependingCrate<'a>(pub &'a str);
