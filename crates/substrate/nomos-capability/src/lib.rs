//! Band 2 — capability contracts and the registry that resolves them.
//!
//! One contract and one registry, because the alternative measured in the prototype was
//! a capability list, a compatibility matrix, a support table and a provider manifest
//! that all had to agree and eventually did not.
//!
//! The registry's answer is never a bare no. A capability nobody offers resolves to
//! `MissingCapability`, which is coverage debt, and never to `NotApplicable`, which is a
//! statement about a subject that only a rule may make.

#![forbid(unsafe_code)]

mod contract;
mod offer_refusal;
mod provider_offer;
mod registry;
mod registry_error;
mod registry_error_kind;
mod requirement;
mod resolution;
mod selection;
mod standing;
mod unmet;

pub use contract::CapabilityContract;
pub use offer_refusal::OfferRefusal;
pub use provider_offer::ProviderOffer;
pub use registry::Registry;
pub use registry_error::RegistryError;
pub use registry_error_kind::RegistryErrorKind;
pub use requirement::Requirement;
pub use resolution::Resolution;
pub use selection::Selection;
pub use standing::Standing;
pub use unmet::Unmet;
