//! One package's full set of first-party dependency edges.

use super::dependency_edge::DependencyEdge;

/// One package's full set of first-party dependency edges.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyPayload
{
    pub package: String,
    pub edges: Vec<DependencyEdge>,
}
