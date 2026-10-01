//! What one node of a definition run reported.

use super::NodeDisposition;

/// What one node of a definition run reported, and where in the definition it sat.
///
/// [`Self::index`] is the node's position in the definition's own declared order, and it
/// is what these reports are sorted by -- never the order the nodes were visited in. That
/// is the whole of the determinism guarantee: a definition's nodes are grouped into waves
/// of mutually independent work, the members of one group have no order between them, and
/// a report ordered by anything other than the declaration would have made which order a
/// group happened to be visited in visible to a caller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeReport
{
    /// The node's own name, which is also the value it publishes.
    pub name: String,
    /// The node's position in the definition's declared order.
    pub index: usize,
    /// What became of it.
    pub disposition: NodeDisposition,
}
