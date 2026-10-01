//! The boundary assertions. Each one is a property the architecture claims, made
//! checkable.
//!
//! Twelve properties, one module each: what the README says about the layering, what the
//! dependency graph does about it, who may write a capability id, whether every file is
//! reachable at all, where the Protocol zone is described, whether any manifest still opens
//! with a numeric band, what a finding may take as its subject, what the API transport may
//! project, what the MCP server may depend on, whether the committed lock still pins the
//! XVPE crossing, whether a rule names only the contract half of a crate that bundles a
//! provider with its contract, and whether every provider offer this workspace exports either
//! reaches a real check run or is declared as deliberately not reaching one.
//!
//! `bands` is not one of them. It holds the zone declaration and the walk the others read,
//! and asserts nothing of its own. The count above was already short before this note: it
//! said eight and listed eight while nine modules asserted something, `findings` being the
//! one nobody had added to the sentence.


mod band_zero;
mod capabilities;
mod findings;
mod bands;
mod bundled_contract_half;
mod composed_offers;
mod graph;
mod lock_pinning;
mod manifest_bands;
mod readme;
mod reachability;
mod transport_registry;
mod mcp_registry;
