#![forbid(unsafe_code)]

mod invalidation;
mod component;
mod context;
mod determinism;
mod fact;
mod identity;
mod propagation;
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
pub use store::{Condensation_Of, GenerationCause, RematerializationGroup};
pub use invalidation::{Broadening, InvalidationReport, Supersession};
