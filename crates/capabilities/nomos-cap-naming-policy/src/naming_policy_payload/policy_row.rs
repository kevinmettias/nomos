//! One declared `symbol -> case` mapping, at one [`super::scope::Scope`].

use super::scope::Scope;
use crate::Case;

/// One row of a repository's declared naming policy.
///
/// `symbol` is the key a repository or a language wrote in `standards.json` — `function`,
/// `type`, `constant`, optionally refined by visibility as `function.exported` or
/// `function.unexported`. This crate does not interpret the key's own shape or apply
/// "most specific wins" resolution; that is a judgment over the whole table, owed to
/// whichever rule reads it, the same way `nomos.cap.dependency.edges`' own payload is a
/// flat edge list and `Check_Dependency_Direction` is what walks it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyRow
{
    pub scope: Scope,
    pub symbol: String,
    pub case: Case,
}
