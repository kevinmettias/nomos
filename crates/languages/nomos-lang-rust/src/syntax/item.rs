//! One declaration, as the parse saw it.

use crate::Visibility;
use crate::ItemKind;
/// One declaration, as the file spells it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Item
{
    /// Position in the walk, dense and zero-based. Source order, so it is stable for a
    /// given file and says nothing about any other file.
    pub ordinal: u32,
    pub kind: ItemKind,
    /// The syntactic nesting above this item, outermost first.
    ///
    /// Nesting, not a resolved module path. It does not begin at a crate root, because
    /// this provider does not know which crate the file belongs to — that is workspace
    /// structure, and reading it would make the answer a function of more than this file.
    pub scope: Vec<String>,
    /// The name as written. For an `impl` block, the head of the self type; for a `use`
    /// leaf, the binding it introduces.
    pub name: String,
    pub visibility: Visibility,
    /// The item's documentation, as written, or `None` when it has none.
    ///
    /// `None` is an absence this provider looked for and did not find, never an inability
    /// to look — it parses, so it always sees the attributes. That difference is what the
    /// payload spells, and it is why this is not the same field as the one a line reader
    /// would fill.
    pub documentation: Option<String>,
    /// What the item declares beyond its name, in the payload's vocabulary, or `None` when
    /// the form has nothing to describe.
    pub shape: Option<String>,
}

impl Item
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

#[cfg(test)]
mod tests
{
    use super::*;

    fn An_Item(scope: Vec<String>, name: &str) -> Item
    {
        return Item {
            ordinal: 0,
            kind: ItemKind::Function,
            scope,
            name: name.to_owned(),
            visibility: Visibility::Public,
            documentation: None,
            shape: None,
        };
    }

    #[test]
    fn Test_Qualified_Name_Should_Be_The_Bare_Name_With_No_Scope()
    {
        assert_eq!(An_Item(Vec::new(), "One").Qualified_Name(), "One");
    }

    #[test]
    fn Test_Qualified_Name_Should_Join_Scope_And_Name_With_Double_Colons()
    {
        assert_eq!(
            An_Item(vec!["Counter".to_owned()], "Increment").Qualified_Name(),
            "Counter::Increment"
        );
    }
}
