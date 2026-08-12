//! The boundary assertions. Each one is a property the architecture claims, made
//! checkable.
//!
//! Five properties, one module each: what the README says about the layering, what the
//! dependency graph does about it, who may write a capability id, whether every file is
//! reachable at all, and where band 0 is described.


mod band_zero;
mod capabilities;
mod common;
mod graph;
mod readme;
mod reachability;
