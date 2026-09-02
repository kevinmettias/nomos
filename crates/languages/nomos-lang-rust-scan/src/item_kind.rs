//! What sort of declaration the scan recognised.

/// What kind of declaration a line looks like.
///
/// The labels are the same strings `nomos-lang-rust` writes, and the list is deliberately
/// re-authored rather than imported. Two providers of one capability agree on a payload
/// *format*, which is an interface; sharing an enum would make them agree by construction
/// and there would be nothing left for the slice to check.
///
/// It is a shorter list than a parser's. A line-reader cannot see a `ForeignModule`'s
/// contents or tell a trait alias from a type alias without following the tokens, and a
/// kind it cannot distinguish is a kind it must not claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemKind
{
    Constant,
    Enum,
    ExternCrate,
    Function,
    Implementation,
    MacroDefinition,
    Module,
    Static,
    Struct,
    Trait,
    TypeAlias,
    Union,
    Use,
}

impl ItemKind
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Constant => "Constant",
            Self::Enum => "Enum",
            Self::ExternCrate => "ExternCrate",
            Self::Function => "Function",
            Self::Implementation => "Implementation",
            Self::MacroDefinition => "MacroDefinition",
            Self::Module => "Module",
            Self::Static => "Static",
            Self::Struct => "Struct",
            Self::Trait => "Trait",
            Self::TypeAlias => "TypeAlias",
            Self::Union => "Union",
            Self::Use => "Use",
        };
    }

    /// The keyword that introduces this kind, in the order a scanner must try them.
    ///
    /// `macro_rules` before `macro`, and `const` before nothing — order matters because
    /// these are matched as prefixes and a shorter keyword that is a prefix of a longer one
    /// would claim it first.
    pub(crate) const fn Table() -> &'static [(&'static str, Self)]
    {
        return &[
            ("macro_rules!", Self::MacroDefinition),
            ("extern crate", Self::ExternCrate),
            // SAFETY: no unsafe code here. This is the table entry for the item form of that
            // name -- a string matched as a prefix against a scanned line, in a crate that
            // forbids unsafe at its root. The rule asking for this comment is text-local and
            // cannot see that the keyword sits inside a string literal; the `unsafe fn` entry
            // below escapes it only because that rule spells its own pattern with a trailing
            // space this table has no reason to carry.
            ("unsafe impl", Self::Implementation),
            ("async fn", Self::Function),
            ("unsafe fn", Self::Function),
            ("const fn", Self::Function),
            ("fn", Self::Function),
            ("struct", Self::Struct),
            ("enum", Self::Enum),
            ("union", Self::Union),
            ("trait", Self::Trait),
            ("impl", Self::Implementation),
            ("mod", Self::Module),
            ("type", Self::TypeAlias),
            ("const", Self::Constant),
            ("static", Self::Static),
            ("use", Self::Use),
        ];
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The stable vocabulary an encoded payload is read back through.
    #[test]
    fn Test_Label_Should_Match_The_Shared_Vocabulary()
    {
        assert_eq!(ItemKind::Function.Label(), "Function");
        assert_eq!(ItemKind::Struct.Label(), "Struct");
        assert_eq!(ItemKind::MacroDefinition.Label(), "MacroDefinition");
    }

    #[test]
    fn Test_Table_Should_Try_Macro_Rules_Before_Any_Shorter_Prefix_Of_It()
    {
        let table = ItemKind::Table();

        assert_eq!(table.first(), Some(&("macro_rules!", ItemKind::MacroDefinition)));
    }

    #[test]
    fn Test_Table_Should_Name_Every_Keyword_This_Scanner_Recognizes()
    {
        let keywords: Vec<&str> = ItemKind::Table().iter().map(|(keyword, _)| return *keyword).collect();

        assert!(keywords.contains(&"fn"));
        assert!(keywords.contains(&"struct"));
        assert!(keywords.contains(&"use"));
    }
}
