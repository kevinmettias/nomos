//! The store contract, sealed so its invariants stay this crate's to keep.
//!
//! Declared at the crate root rather than in `fact/`, and `lib.rs` says why where it
//! declares it: the name it is published under already carries the fact, so a file named
//! for it cannot also sit inside the folder that name would otherwise group it with.

use nomos_contracts::GenerationId;
use crate::InvalidationReport;
use crate::GenerationCause;
use crate::Supersession;
use crate::FactKey;
use crate::MaterializedFact;
use crate::FactIdentity;

#[path = "fact_store/sealed.rs"] pub(crate) mod sealed;

pub trait FactStore: sealed::Sealed
{
    fn Current(&self, identity: &FactIdentity, at: GenerationId) -> Option<MaterializedFact>;

    fn Historical(&self, key: &FactKey) -> Option<(MaterializedFact, Supersession)>;

    fn Invalidate(&mut self, cause: &GenerationCause, from: GenerationId) -> InvalidationReport;
}
