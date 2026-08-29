//! One declaration, as the parse saw it.

use crate::ItemKind;
use crate::Visibility;

/// One declaration, as the file spells it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SyntaxItem
{
    /// Position in the walk, dense and zero-based. Source order, so it is stable for a
    /// given file and says nothing about any other file.
    pub ordinal: u32,
    pub kind: ItemKind,
    /// The syntactic nesting above this item, outermost first.
    ///
    /// For a method, the receiver type's name — Go's closest analogue to
    /// `nomos-lang-rust`'s `impl` scope, without an `impl` item of its own to carry it. Not
    /// a package path: this provider does not know which package the file belongs to.
    pub scope: Vec<String>,
    /// The name as written. For a method, the identifier after the receiver.
    pub name: String,
    pub visibility: Visibility,
    /// The item's documentation, as written, or `None` when it has none.
    ///
    /// `None` is an absence this provider looked for and did not find, never an inability to
    /// look — it parses, so it always sees every comment. That difference is what the
    /// payload spells, and it is why this is not the same field as the one a line reader
    /// would fill.
    pub documentation: Option<String>,
    /// What the item declares beyond its name, in the payload's vocabulary, or `None` when
    /// the form has nothing to describe.
    pub shape: Option<String>,
}

impl SyntaxItem
{
    /// The item's name qualified by its syntactic nesting.
    #[must_use]
    pub fn Qualified_Name(&self) -> String
    {
        if self.scope.is_empty()
        {
            return self.name.clone();
        }

        return format!("{}::{}", self.scope.join("::"), self.name);
    }
}
