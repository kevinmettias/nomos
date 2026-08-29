//! What sort of thing a subject addresses.

use serde::{Deserialize, Serialize};

/// What kind of thing a subject denotes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SubjectKind
{
    /// A persisted or generated repository object.
    Artifact,
    /// A language-semantic declaration.
    Symbol,
    /// A non-code entity.
    Resource,
    /// A named group of subjects treated as one.
    Aggregate,
}
