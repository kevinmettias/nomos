#![forbid(unsafe_code)]

// Six of this crate's types are declared here rather than inside the folder that would
// otherwise hold them, because the name each is published under already carries that
// folder's name. The three rules leave no other corner: `fact/fact_error.rs` repeats its
// parent, `fact/error.rs` declares a type its file is not named for, and keeping `Error` and
// republishing it as `FactError` is the facade alias this crate used to carry. At the root
// the name repeats no parent, the file matches the type, and no alias is needed. Each file
// says the same in its own header, and `invalidation_report.rs` is the same story one folder
// over.
mod fact_error;
mod fact_identity;
mod fact_key;
mod fact_reader;
mod fact_store;
mod invalidation_report;

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
pub use fact::{FactPayload, GuaranteeDigest, MaterializedFact, MemoryFactStore};
pub use fact_error::FactError;
pub use fact_identity::FactIdentity;
pub use fact_key::FactKey;
pub use fact_reader::FactReader;
pub use fact_store::FactStore;
pub use identity::InputDigest;
pub use reader::ReadOutcome;
pub use reading::{Dependency, Reader};
pub use store::{Condensation_Of, GenerationCause, RematerializationGroup};
pub use invalidation::{Broadening, Supersession};
pub use invalidation_report::InvalidationReport;
