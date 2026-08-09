#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]

mod fact;
mod identity;
mod reader;
mod store;

pub use fact::{FactError, FactPayload, MaterializedFact, Supersession};
pub use identity::{Component, FactIdentity, FactKey, GuaranteeDigest, InputDigest};
pub use reader::{Context, Dependency, FactReader, ReadOutcome, Reader};
pub use store::{
    Broadening, FactStore, GenerationCause, InvalidationReport, MemoryFactStore,
};
