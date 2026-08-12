//! One value of one field, and where it came from.

use crate::origin::Origin;

/// One value of one field, and where it came from.
///
/// Several of these may name one field. The later one supersedes the earlier for reading and
/// never replaces it in storage, which is what makes what was originally asked recoverable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldValue
{
    /// Which field this is a value of.
    pub field: String,
    /// The value, as given.
    pub value: String,
    /// Where it came from.
    pub origin: Origin,
}
