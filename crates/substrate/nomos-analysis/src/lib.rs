#![forbid(unsafe_code)]

mod invalidation;
mod component;
mod context;
#[path = "fact_reuse.rs"] mod determinism;
#[path = "fact_payload.rs"] mod fact;
#[path = "input_digest.rs"] mod identity;
mod propagation;
#[path = "read_outcome.rs"] mod reader;
#[path = "reader.rs"] mod reading;
#[path = "generation_cause.rs"] mod store;

pub use component::Component;
pub use context::Context;
pub use determinism::FactReuse;
pub use fact::{FactError, FactIdentity, FactKey, FactPayload, FactReader, FactStore, GuaranteeDigest, MaterializedFact, MemoryFactStore};
pub use identity::InputDigest;
pub use reader::ReadOutcome;
pub use reading::{Dependency, Reader};
pub use store::{Condensation_Of, GenerationCause, RematerializationGroup};
pub use invalidation::{Broadening, InvalidationReport, Supersession};
