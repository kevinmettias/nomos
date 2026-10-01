//! One list a guard quantifies over.

use crate::UniverseKind;
/// One list that some completeness guard quantifies over.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeclaredUniverse
{
    /// Repo-relative, forward slashes. For reporting; never identity.
    pub path: String,
    /// `GOVERNING_RECORD_IDS`, or `Table::All` for an enumeration.
    ///
    /// This is the stable name. A universe that moves file keeps it, which is what lets
    /// a finding about one survive a refactor instead of closing and reopening.
    pub name: String,
    /// How it is written down.
    pub kind: UniverseKind,
    /// The mirror this universe claims, if it claims one.
    ///
    /// A claim, not a fact. Whether the named check exists is what the rule resolves,
    /// and a claim that resolves to nothing is worse than no claim at all — it reads as
    /// coverage while checking nothing.
    pub claimed_mirror: Option<String>,
}
