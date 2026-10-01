//! The columns a node carries, as one value.

/// The columns a node carries, as one value.
///
/// Grouped because they only mean anything together: an identifier without the authority
/// that speaks for it says nothing about whether a later writer may overwrite it, and five
/// bare strings in a fixed order is a shape a caller gets wrong silently.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeRow<'a>
{
    pub node_id: &'a str,
    pub kind: &'a str,
    pub authority: &'a str,
    pub representation: &'a str,
    pub title: &'a str,
}
