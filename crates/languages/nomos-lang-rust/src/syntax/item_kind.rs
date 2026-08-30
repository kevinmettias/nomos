//! What sort of declaration the parser found.

/// What kind of declaration an item is.
///
/// Every item form Rust has, spelled out rather than collapsed into `Other`. An `Other`
/// bucket is where a form goes to be forgotten: the count stays right, nothing reports
/// it, and the day somebody needs `ForeignModule` they find it was never distinguished.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ItemKind
{
    Constant,
    Enum,
    ExternCrate,
    ForeignModule,
    Function,
    Implementation,
    MacroDefinition,
    Module,
    Static,
    Struct,
    Trait,
    TraitAlias,
    TypeAlias,
    Union,
    Use,
}

impl ItemKind
{
    /// The kind's stable `PascalCase` name, as it appears in an encoded payload.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Constant => "Constant",
            Self::Enum => "Enum",
            Self::ExternCrate => "ExternCrate",
            Self::ForeignModule => "ForeignModule",
            Self::Function => "Function",
            Self::Implementation => "Implementation",
            Self::MacroDefinition => "MacroDefinition",
            Self::Module => "Module",
            Self::Static => "Static",
            Self::Struct => "Struct",
            Self::Trait => "Trait",
            Self::TraitAlias => "TraitAlias",
            Self::TypeAlias => "TypeAlias",
            Self::Union => "Union",
            Self::Use => "Use",
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

    /// The stable vocabulary an encoded payload is read back through.
    #[test]
    fn Test_Label_Should_Match_The_Shared_Vocabulary()
    {
        assert_eq!(ItemKind::Function.Label(), "Function");
        assert_eq!(ItemKind::Struct.Label(), "Struct");
        assert_eq!(ItemKind::ForeignModule.Label(), "ForeignModule");
        assert_eq!(ItemKind::TraitAlias.Label(), "TraitAlias");
    }
}
