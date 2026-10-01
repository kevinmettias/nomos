use serde::{Deserialize, Serialize};

/// What kind of declaration a [`Symbol`](super::Symbol) is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind
{
    /// A module or namespace.
    Module,
    /// A type declaration.
    Type,
    /// A callable.
    Function,
    /// A field or property.
    Field,
    /// A constant or static.
    Constant,
    /// A trait, interface or protocol.
    Interface,
    /// An anonymous callable.
    Closure,
    /// A block or region within a callable.
    Block,
}
