#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]

mod invalidation;
mod component;
mod context;
mod determinism;
mod fact;
mod identity;
mod reader;
mod reading;
mod store;

pub use component::Component;
pub use context::Context;
pub use determinism::FactReuse;
pub use fact::{FactError, FactIdentity, FactKey, FactPayload, FactReader, FactStore, GuaranteeDigest, MaterializedFact, MemoryFactStore};
pub use identity::InputDigest;
pub use reader::ReadOutcome;
pub use reading::{Dependency, Reader};
pub use store::GenerationCause;
pub use invalidation::{Broadening, InvalidationReport, Supersession};
