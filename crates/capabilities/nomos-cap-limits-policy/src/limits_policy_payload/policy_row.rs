//! One declared `key -> value` mapping, at one [`super::scope::Scope`].

use super::scope::Scope;

/// One row of a repository's declared limits policy.
///
/// `key` is the identifier a repository or a language wrote in `standards.json` —
/// `file-size-review-lines`, `file-size-hard-lines`, `parameter-count-max`, and their
/// like. This crate does not interpret the key's own spelling or apply any resolution
/// over the whole table; that is a judgment owed to whichever rule reads it, the same way
/// `nomos_cap_naming_policy::PolicyRow` leaves `symbol` uninterpreted and
/// `nomos.cap.dependency.edges`' own payload is a flat edge list `Check_Dependency_
/// Direction` is what walks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyRow
{
    pub scope: Scope,
    pub key: String,
    pub value: u32,
}
