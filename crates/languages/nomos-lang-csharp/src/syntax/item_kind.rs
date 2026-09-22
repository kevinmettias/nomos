//! What sort of declaration the parser found.

/// What kind of declaration an item is.
///
/// Every declaration form inside this crate's stated scope, spelled out rather than collapsed
/// into `Other` — `nomos-lang-rust`'s own [`ItemKind`](item_kind) states why an `Other` bucket
/// is where a form goes to be forgotten. C# has more of them than either language already
/// read here, and that is the language rather than an indulgence: a property, an indexer, an
/// event and a field are four different declarations with four different member kinds, and a
/// consumer that had to tell them apart from a `qualified_name` could not.
///
/// A method records [`nomos_cap_syntax::FUNCTION`] — the shared label its Rust and Go peers
/// both write for a callable — qualified into its declaring type's scope, the same choice
/// `nomos-lang-go` makes for a method on a receiver. A constructor and a finalizer do not: both
/// are callables whose name *is* the type's name, so `qualified_name` cannot tell them from
/// each other or from the type, which is exactly the criterion the payload schema states for
/// when a distinction belongs in `kind`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ItemKind
{
    /// `namespace X { ... }` or `namespace X;`.
    Namespace,
    /// `using X;`, `using static X;`, `using A = X;` or `global using X;` — C#'s own keyword,
    /// in place of `nomos-lang-rust`'s `Use` and `nomos-lang-go`'s `Import`.
    Using,
    Class,
    Struct,
    Interface,
    /// `record`, `record class` or `record struct` — one kind for all three, because the file
    /// spells one keyword and the difference between them is what the compiler generates.
    Record,
    Enum,
    /// `delegate void Handler(...)` — a named function type, which no other language this
    /// workspace reads declares as its own top-level form.
    Delegate,
    /// A method declaration.
    Function,
    Constructor,
    /// `~Type()`. The grammar calls the node a destructor; C# calls the member a finalizer, and
    /// this records what the language calls it.
    Finalizer,
    /// `operator +`, or an `implicit`/`explicit` conversion operator.
    Operator,
    Property,
    /// `this[...]` — a property taking arguments.
    Indexer,
    Field,
    Event,
    /// One name inside an `enum` body.
    EnumMember,
}

impl ItemKind
{
    /// The kind's stable `PascalCase` name, as it appears in an encoded payload.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Namespace => "Namespace",
            Self::Using => "Using",
            Self::Class => "Class",
            Self::Struct => "Struct",
            Self::Interface => "Interface",
            Self::Record => "Record",
            Self::Enum => "Enum",
            Self::Delegate => "Delegate",
            Self::Function => nomos_cap_syntax::FUNCTION,
            Self::Constructor => "Constructor",
            Self::Finalizer => "Finalizer",
            Self::Operator => "Operator",
            Self::Property => "Property",
            Self::Indexer => "Indexer",
            Self::Field => "Field",
            Self::Event => "Event",
            Self::EnumMember => "EnumMember",
        };
    }
}

impl core::fmt::Display for ItemKind
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// Every kind this provider can write, so the assertions below quantify over the enum
    /// rather than over a sample of it.
    const EVERY_KIND: &[ItemKind] = &[
        ItemKind::Namespace,
        ItemKind::Using,
        ItemKind::Class,
        ItemKind::Struct,
        ItemKind::Interface,
        ItemKind::Record,
        ItemKind::Enum,
        ItemKind::Delegate,
        ItemKind::Function,
        ItemKind::Constructor,
        ItemKind::Finalizer,
        ItemKind::Operator,
        ItemKind::Property,
        ItemKind::Indexer,
        ItemKind::Field,
        ItemKind::Event,
        ItemKind::EnumMember,
    ];

    /// The one label that is not this crate's own string: a method writes the vocabulary its
    /// two sibling providers write, so a consumer asking for callables across three languages
    /// gets one answer.
    #[test]
    fn Test_A_Method_Should_Carry_The_Shared_Callable_Label()
    {
        assert_eq!(ItemKind::Function.Label(), nomos_cap_syntax::FUNCTION);
    }

    /// A payload is read back by `kind`, so two kinds sharing a label would be two forms a
    /// consumer cannot tell apart.
    #[test]
    fn Test_Every_Kind_Should_Carry_A_Distinct_Non_Empty_Label()
    {
        use std::collections::BTreeSet;

        let labels: BTreeSet<&str> = EVERY_KIND.iter().map(|kind| return kind.Label()).collect();

        assert_eq!(labels.len(), EVERY_KIND.len(), "{labels:?}");
        assert!(!labels.contains(""), "a kind with no label is a kind a consumer cannot name");
    }

    #[test]
    fn Test_Display_Should_Render_The_Label()
    {
        assert_eq!(ItemKind::EnumMember.to_string(), "EnumMember");
    }
}
