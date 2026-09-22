//! Shared plumbing every declaration recorder in [`super`] uses: reading a node's own named
//! fields and modifier words out of the parse tree, and turning what was found into one recorded
//! item.

use super::super::{Documentation_Of_Declaration, ItemKind, Visibility};
use super::recording::Recording;
use super::site::Site;
use tree_sitter::Node;

/// The keyword that opens a parameter this grammar gives no node of its own.
const PARAMETER_ARRAY_KEYWORD: &str = "params";

/// The fields one recorded declaration needs, grouped so [`Recording::Push`] takes a small, fixed
/// number of parameters regardless of how many facts a declaration carries.
pub(super) struct ItemRecord
{
    pub(super) kind: ItemKind,
    pub(super) scope: Vec<String>,
    pub(super) name: String,
    pub(super) visibility: Visibility,
    pub(super) documentation: Option<String>,
    pub(super) shape: Option<String>,
}

/// Records one declaration whose accessibility is whatever its own modifier words state and whose
/// form has no shape to report — every type declaration, in other words.
pub(super) fn Push_Modifiable_Declaration(recording: &mut Recording, site: Site, named: (ItemKind, &str))
{
    let (kind, name) = named;

    recording.Push(ItemRecord {
        kind,
        scope: site.scope.to_vec(),
        name: name.to_owned(),
        visibility: Visibility::Of_Modifiers(&Modifier_Words(site.node, site.source)),
        documentation: Documentation_Of_Declaration(site.node, site.source),
        shape: None,
    });
}

/// Records one declaration whose form cannot carry an accessibility modifier at all — a namespace,
/// a `using` directive, an enum member.
///
/// [`Visibility::NotApplicable`] rather than [`Visibility::Unspecified`], and the difference is the
/// whole reason there are two: `Unspecified` says the file could have stated an accessibility and
/// did not, which for these forms would be false.
pub(super) fn Push_Unmodifiable_Declaration(recording: &mut Recording, site: Site, named: (ItemKind, &str))
{
    let (kind, name) = named;

    recording.Push(ItemRecord {
        kind,
        scope: site.scope.to_vec(),
        name: name.to_owned(),
        visibility: Visibility::NotApplicable,
        documentation: Documentation_Of_Declaration(site.node, site.source),
        shape: None,
    });
}

/// The name a `name`-field-carrying node declares.
pub(super) fn Declared_Name(node: Node, source: &[u8]) -> Option<String>
{
    let name_node = node.child_by_field_name("name")?;
    let name = name_node.utf8_text(source).ok()?;

    return Some(name.to_owned());
}

/// Every modifier word a declaration states, in source order.
///
/// Read as `modifier` child nodes rather than by scanning the declaration's own leading text:
/// `internal` is also a valid identifier position elsewhere in a declaration, and a text scan
/// would find the word in a parameter's type name. The grammar already separates the two, and this
/// reads its answer.
pub(super) fn Modifier_Words<'source>(node: Node, source: &'source [u8]) -> Vec<&'source str>
{
    let mut cursor = node.walk();
    let mut words = Vec::new();

    for child in node.children(&mut cursor)
    {
        if child.kind() != "modifier"
        {
            continue;
        }

        if let Ok(word) = child.utf8_text(source)
        {
            words.push(word);
        }
    }

    return words;
}

/// How many parameters a parameter list declares.
///
/// Counts `parameter` children, plus the `params` keyword — and the second half is not
/// belt-and-braces. A parameter array (`params int[] rest`) is *not* wrapped in a node of its own
/// in this grammar: its keyword, its type and its name are spliced directly into the
/// `parameter_list`, so a count of `parameter` children alone reports one parameter fewer than the
/// signature declares. Confirmed against the real tree, after a fixture counting three reported
/// two.
///
/// Unlike Go's own equivalent there is no shared-type form to unfold: C# spells a type per
/// parameter.
pub(super) fn Parameter_Arity(list: Node) -> usize
{
    let mut cursor = list.walk();
    let mut total = 0_usize;

    for child in list.children(&mut cursor)
    {
        if child.kind() == "parameter" || child.kind() == PARAMETER_ARRAY_KEYWORD
        {
            total = total.saturating_add(1);
        }
    }

    return total;
}

/// The scope a declaration's own members sit in: everything above it, then its own name.
pub(super) fn Nested_Scope(scope: &[String], name: &str) -> Vec<String>
{
    let mut nested = scope.to_vec();
    nested.push(name.to_owned());

    return nested;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::syntax::fixture::{Find_Kind, Parsed_Fixture_Tree};

    /// The words `protected internal virtual` states, of which one is an accessibility.
    const FIXTURE_MODIFIER_COUNT: usize = 3;

    /// The parameters `(int a, string b, params int[] rest)` declares.
    const FIXTURE_PARAMETER_COUNT: usize = 3;

    #[test]
    fn Test_Declared_Name_Should_Read_The_Nodes_Own_Name_Field()
    {
        let source = "public class Widget { }\n";
        let tree = Parsed_Fixture_Tree(source);
        let declaration = Find_Kind(tree.root_node(), "class_declaration").expect("the fixture declares a class");

        assert_eq!(Declared_Name(declaration, source.as_bytes()), Some("Widget".to_owned()));
    }

    #[test]
    fn Test_Modifier_Words_Should_Read_Every_Modifier_A_Declaration_States()
    {
        let source = "class Widget { protected internal virtual void Reset() { } }\n";
        let tree = Parsed_Fixture_Tree(source);
        let declaration = Find_Kind(tree.root_node(), "method_declaration").expect("the fixture declares a method");

        let words = Modifier_Words(declaration, source.as_bytes());

        assert_eq!(words.len(), FIXTURE_MODIFIER_COUNT, "{words:?}");
        assert_eq!(Visibility::Of_Modifiers(&words), Visibility::ProtectedInternal);
    }

    #[test]
    fn Test_Parameter_Arity_Should_Count_A_Parameter_Array_As_One_Parameter()
    {
        let source = "class Widget { void Take(int a, string b, params int[] rest) { } }\n";
        let tree = Parsed_Fixture_Tree(source);
        let list = Find_Kind(tree.root_node(), "parameter_list").expect("the fixture declares parameters");

        assert_eq!(Parameter_Arity(list), FIXTURE_PARAMETER_COUNT);
    }

    #[test]
    fn Test_Nested_Scope_Should_Append_The_Declarations_Own_Name()
    {
        let scope = vec!["Acme".to_owned()];

        assert_eq!(Nested_Scope(&scope, "Widget"), vec!["Acme".to_owned(), "Widget".to_owned()]);
    }
}
