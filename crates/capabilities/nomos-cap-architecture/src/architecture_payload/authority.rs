//! One package a repository declares a sole write authority, and the doors into it.

/// One declared authority: a package nothing may depend on except the packages named here.
///
/// An authority with no doors is representable and means what it says -- nothing may depend on
/// it directly -- which is why the encoding declares the authority on its own line rather than
/// inferring it from the presence of a door.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Authority
{
    pub package: String,
    pub doors: Vec<String>,
}
