//! What sort of declaration the parser found.

/// What kind of declaration an item is.
///
/// Every package-level declaration form the Go spec has, spelled out rather than collapsed
/// into `Other` — `nomos-lang-rust`'s own [`ItemKind`](item_kind) states why an `Other`
/// bucket is where a form goes to be forgotten.
///
/// There is no separate `Method` kind. A method — `func (r T) Name()` — records the same
/// [`ItemKind::Function`] a free function does, qualified into `T`'s scope instead of
/// carrying its own kind — the same choice `nomos-lang-rust` makes for an `impl` member,
/// and for the same reason: the distinction a consumer wants is which type a name belongs
/// to, and `qualified_name` already carries that. Go has no `impl` block to hang a separate
/// `Implementation` item on, so a method's receiver type never gets an item of its own the
/// way `impl Type` does in Rust; the method's own `scope` is the only place that type name
/// is recorded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ItemKind
{
    /// A `func` declaration — free or with a receiver.
    Function,
    /// `type X struct { ... }`.
    Struct,
    /// `type X interface { ... }`.
    Interface,
    /// `type X = Y` — a true alias; `X` and `Y` name one type.
    TypeAlias,
    /// `type X Y` where `Y` is neither a struct nor an interface literal — a new, distinct
    /// type with `Y`'s underlying representation.
    TypeDefinition,
    Constant,
    /// `var` — Go's own keyword, not folded into `nomos-lang-rust`'s `Static`: the two
    /// languages name the same package-level-mutable-storage concept differently, and this
    /// provider records what the file says rather than what its closest Rust analogue is
    /// called.
    Variable,
    /// `import "path"` — Go's own keyword, in place of `nomos-lang-rust`'s `Use`.
    Import,
}

impl ItemKind
{
    /// The kind's stable `PascalCase` name, as it appears in an encoded payload.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Function => nomos_cap_syntax::FUNCTION,
            Self::Struct => "Struct",
            Self::Interface => "Interface",
            Self::TypeAlias => "TypeAlias",
            Self::TypeDefinition => "TypeDefinition",
            Self::Constant => "Constant",
            Self::Variable => "Variable",
            Self::Import => "Import",
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
