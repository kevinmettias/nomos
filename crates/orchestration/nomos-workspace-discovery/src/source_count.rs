//! How many sources of one registered language the walk found under a root.

/// The number of files under a root whose extension one registered language package
/// recognizes -- one entry per extension [`crate::Registered_Extensions`] names, in that
/// order, so a profile of a tree with no Go in it still says `go: 0` rather than omitting
/// the language a first run would otherwise have to know to ask about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceCount
{
    /// The extension, spelled the way [`crate::Registered_Extensions`] spells it.
    pub extension: &'static str,
    /// How many files carrying it [`crate::Walked_Sources`] read. The same population a
    /// check run judges, so a file the walk does not reach -- under `target`, inside a
    /// nested repository root, or not readable as text -- is not counted here either. That
    /// is the number a person adopting nomos needs: what a run will look at, not what a
    /// directory listing would find.
    pub sources: usize,
}
