//! A relation name, its tier, the name of its inverse, and what it constrains.

use serde::{Deserialize, Serialize};

/// `OD-SPEC-012`: which node kinds this type may join at each end, and how many edges of
/// it one node may carry. Both `domain` and `range` are sorted before they reach here — the
/// store sorts before it serializes, and the bundle's byte-identical round trip depends on
/// that the same way it depends on every other ordering in this crate being by natural key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationType
{
    pub name: String,
    pub tier: String,
    pub inverse_of: Option<String>,
    pub domain: Vec<String>,
    pub range: Vec<String>,
    pub max_per_node: u32,
}
