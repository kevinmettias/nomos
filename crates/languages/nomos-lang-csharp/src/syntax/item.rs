//! One declaration, as the parse saw it.

use crate::ItemKind;
use crate::Visibility;

/// One declaration, as the file spells it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Item
{
    /// Position in the walk, dense and zero-based. Source order, so it is stable for a given
    /// file and says nothing about any other file.
    pub ordinal: u32,
    pub kind: ItemKind,
    /// The syntactic nesting above this item, outermost first.
    ///
    /// C# nests further than either language this workspace already reads: a namespace
    /// encloses a type, a type encloses a type, and a type encloses its members, all in one
    /// file. Each enclosing declaration contributes its own name as written — a dotted
    /// namespace name (`Acme.Widgets`) stays one segment, because the file spells it as one
    /// declaration and splitting it would invent nesting the source does not have.
    pub scope: Vec<String>,
    /// The name as written. An operator's is the operator token, an indexer's is `this`, and a
    /// constructor's or finalizer's is the type's own name, because that is what C# spells.
    pub name: String,
    pub visibility: Visibility,
    /// The item's documentation, as written, or `None` when it has none.
    ///
    /// `None` is an absence this provider looked for and did not find, never an inability to
    /// look — it parses, so it always sees every comment. That difference is what the payload
    /// spells, and it is why this is not the same field as the one a line reader would fill.
    pub documentation: Option<String>,
    /// What the item declares beyond its name, in the payload's vocabulary, or `None` when the
    /// form has nothing to describe.
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
        assert_eq!(An_Item(Vec::new(), "Widget").Qualified_Name(), "Widget");
    }

    #[test]
    fn Test_Qualified_Name_Should_Join_Every_Enclosing_Declaration_With_Double_Colons()
    {
        let scope = vec!["Acme.Widgets".to_owned(), "Widget".to_owned()];

        assert_eq!(An_Item(scope, "Reset").Qualified_Name(), "Acme.Widgets::Widget::Reset");
    }
}
