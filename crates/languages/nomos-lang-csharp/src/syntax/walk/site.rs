//! Where a declaration sits: the node, the scope above it, and the bytes it was read from.
//!
//! The three travel together through every recorder in [`super`], and they are one value rather
//! than three parameters because this repository caps a function at four of them and a recorder
//! that also takes a kind and a `Recording` would be at six. Grouping them is also the honest
//! shape: no recorder ever wants one of the three without the other two.

use tree_sitter::Node;

/// The scope a `compilation_unit`'s own children sit in — nothing above them yet.
///
/// A named constant rather than an inline `&[]`, because an empty slice literal of a `Drop` type
/// is not something to rely on the compiler promoting.
const NO_SCOPE: &[String] = &[];

/// One place in the parse tree, with everything a recorder needs in order to read it.
#[derive(Clone, Copy)]
pub(super) struct Site<'reading>
{
    /// The node this recorder was handed.
    pub(super) node: Node<'reading>,
    /// The declarations enclosing it, outermost first.
    pub(super) scope: &'reading [String],
    /// The file's own bytes, which every name and every comment is read out of.
    pub(super) source: &'reading [u8],
}

impl<'reading> Site<'reading>
{
    /// A whole file: its compilation unit, with nothing above it.
    pub(super) fn At_Root(node: Node<'reading>, source: &'reading [u8]) -> Self
    {
        return Self { node, scope: NO_SCOPE, source };
    }

    /// The same reading, at another node and in another scope.
    ///
    /// The returned site borrows for no longer than the scope it is given, which is what lets a
    /// recorder build a nested scope as a local and hand it down.
    pub(super) fn Moved_To<'inner>(&self, node: Node<'inner>, scope: &'inner [String]) -> Site<'inner>
    where
        'reading: 'inner,
    {
        return Site { node, scope, source: self.source };
    }
}
