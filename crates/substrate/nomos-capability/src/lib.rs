//! Zone: Substrate — capability contracts and the registry that resolves them.
//!
//! One contract and one registry, because the alternative measured in the prototype was
//! a capability list, a compatibility matrix, a support table and a provider manifest
//! that all had to agree and eventually did not.
//!
//! The registry's answer is never a bare no. A capability nobody offers resolves to
//! `MissingCapability`, which is coverage debt, and never to `NotApplicable`, which is a
//! statement about a subject that only a rule may make.

#![forbid(unsafe_code)]

// Resolution's parts sit under resolution. Flat, this level was twelve files whose grouping
// a reader had to reconstruct from the names.
//
// The registry's two error types sit here rather than inside `registry/`, because the name
// each is published under already carries the registry: `registry/registry_error.rs` repeats
// its parent folder, and `registry/error.rs` declaring `Error` needs an alias to be published
// at all. See each file's own header.
#[path = "capability_contract.rs"]
mod contract;
mod offer_refusal;
mod provider_offer;
mod registry;
mod registry_error;
mod registry_error_kind;
mod requirement;
mod resolution;

// Shared shape for a capability contract's own tests -- see its module doc for why this
// is a plain public module here rather than `#[cfg(test)]` on each of its five callers.
pub mod contract_testing;

pub use contract::CapabilityContract;
pub use offer_refusal::OfferRefusal;
pub use provider_offer::ProviderOffer;
pub use registry::{Registry, RequiredResolution, RequiredUnmet};
pub use registry_error::RegistryError;
pub use registry_error_kind::RegistryErrorKind;
pub use requirement::Requirement;
pub use resolution::{Remedy, Resolution, Selection, Standing, Unmet};
