//! The boundary assertions. Each one is a property the architecture claims, made
//! checkable.
//!
//! Six properties, one module each: what the README says about the layering, what the
//! dependency graph does about it, who may write a capability id, whether every file is
//! reachable at all, where band 0 is described, and what the API transport may project.


mod band_zero;
mod capabilities;
mod bands;
mod graph;
mod readme;
mod reachability;
mod transport_registry;
