//! The boundary assertions. Each one is a property the architecture claims, made
//! checkable.
//!
//! Seven properties, one module each: what the README says about the layering, what the
//! dependency graph does about it, who may write a capability id, whether every file is
//! reachable at all, where band 0 is described, what the API transport may project, and
//! what the MCP server may depend on.


mod band_zero;
mod capabilities;
mod bands;
mod graph;
mod readme;
mod reachability;
mod transport_registry;
mod mcp_registry;
