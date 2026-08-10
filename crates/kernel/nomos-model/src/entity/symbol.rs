use serde::{Deserialize, Serialize};

use super::{EntityId, SymbolKind};

/// A language-semantic declaration or executable unit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Symbol
{
    /// Stable identity.
    pub id: EntityId,
    /// Composite identity, from which [`Symbol::id`] is derived.
    pub identity: crate::CompositeIdentity,
    /// What kind of declaration this is.
    pub kind: SymbolKind,
    /// The enclosing symbol, if any.
    pub container: Option<EntityId>,
    /// The artifact this symbol is declared in.
    pub declared_in: EntityId,
}
