//! Where an item came from.

use serde::Deserialize;
use serde::Serialize;

/// Where an item came from: a person, or a session working this repository.
///
/// The minimum distinction `OD-LEDGER-024`'s `done_when` asks for — "work that was
/// required from work a session proposed" — and no finer than that. A closed set for the
/// same reason [`crate::ItemKind`] is one: an unrecognized origin is refused by the same
/// parse that already refuses an unrecognized key, rather than stored and ignored.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemOrigin
{
    /// Specified by a person: written into the plan this ledger's own `P1`–`P8` batches
    /// built out (`OD-LEDGER-002`), or asked for directly in a session.
    Required,
    /// Opened by a session that observed something the tree did not already declare, with
    /// nobody having asked for that exact item by name. `OD-LEDGER-002`'s own words for
    /// `P9` and after: "findings from an audit of the tree against what it claims."
    Proposed,
}
