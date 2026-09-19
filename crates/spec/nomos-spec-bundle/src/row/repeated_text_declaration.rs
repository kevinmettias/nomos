//! One declaration a corpus makes about the text its own layout repeats.

use serde::{Deserialize, Serialize};

/// One declaration that a canonical block is intentionally projected into a named
/// structural role.
///
/// `OD-SPEC-004` version 3 decided a corpus may declare the text its own layout repeats, and
/// that the declaration names structural roles rather than the literal strings. So a row
/// here is `normalized_hash` — the identity of the repeated text, the same column
/// `NSV-PRESERVE-004` groups `source_blocks` by — plus the `role` the text plays and the
/// `multiplicity` the role implies. The text itself is never carried here; `source_blocks`
/// already holds it, and storing it again under a better name would be the second compiled-in
/// list the amendment refused.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepeatedTextDeclaration
{
    pub normalized_hash: String,
    pub role: String,
    pub multiplicity: i64,
}
