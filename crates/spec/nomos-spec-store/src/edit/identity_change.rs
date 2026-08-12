//! A front matter field the edit changes.

/// A front matter field the edit changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentityChange
{
    pub field: String,
    pub before: String,
    pub after: String,
}
