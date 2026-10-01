//! [`DiscardedValue`], one value assigned to the blank identifier.

/// One value a Go file assigns to `_`: where the blank identifier stands, the value's type as the
/// type checker resolved it, and whether that type is `error`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DiscardedValue
{
    /// The one-based line of the blank identifier.
    pub line: u32,
    /// The one-based column of the blank identifier, in bytes, as the Go toolchain counts it.
    pub column: u32,
    /// Whether the value's type is `error` -- the predeclared interface, or a type that implements
    /// it.
    pub is_error: bool,
    /// The value's type, spelled as `go/types` spells it: `error`, `int`, `*os.File`.
    pub type_name: String,
}
