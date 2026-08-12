//! One strategy declaration, as text.
//!
//! Carried as text rather than as the three `nomos-contracts` enums, because this crate
//! does not link `nomos-contracts` and must not start: the whole point of that crate is
//! that it is reimplemented by peers who never compile it, and an observer that compiled
//! it would be a participant in the protocol it is watching.

use crate::DomainRow;

/// One declaration: the type that made it, and the triple it declared.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Declaration
{
    /// The type name following `impl Strategy for`.
    pub strategy: String,
    /// The variant named by `const STRENGTH`.
    pub strength: String,
    /// The variant named by `const SCOPE`.
    pub scope: String,
    /// The variant named by `const TRACE`.
    pub trace: String,
}

impl Declaration
{
    /// Whether this declaration is the row's occupant.
    ///
    /// All three axes, because two rows of the table differ only in scope and matching on
    /// strength alone would report the weaker one as occupied by the stronger one's
    /// declaration — a completeness guard passing because it compared too little.
    #[must_use]
    pub fn Occupies(&self, row: &DomainRow) -> bool
    {
        return self.strength == row.strength
            && self.scope == row.scope
            && self.trace == row.trace;
    }
}
