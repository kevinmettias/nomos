//! What a relation type constrains: the node kinds it may join at each end, and how many
//! edges of it one node may carry.

/// What a relation type constrains: the node kinds it may join at each end, and how many
/// edges of it one node may carry.
///
/// Grouped because `OD-SPEC-012` requires all three together — a relation type that
/// declares domain and range but not cardinality, or the reverse, is not a lighter-weight
/// registration, it is the absence of the thing `SpecificationStore::Put_Relation_Type`
/// exists to record.
pub struct Constraint<'a>
{
    pub domain: &'a [&'a str],
    pub range: &'a [&'a str],
    pub max_per_node: u32,
}
