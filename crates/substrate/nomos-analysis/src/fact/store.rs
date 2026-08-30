//! The store contract, sealed so its invariants stay this crate's to keep.

use nomos_contracts::GenerationId;
use crate::InvalidationReport;
use crate::GenerationCause;
use crate::Supersession;
use crate::FactKey;
use crate::MaterializedFact;
use crate::FactIdentity;

pub(crate) mod sealed
{
    pub trait Sealed
    {}
}

pub trait Store: sealed::Sealed
{
    fn Current(&self, identity: &FactIdentity, at: GenerationId) -> Option<MaterializedFact>;

    fn Historical(&self, key: &FactKey) -> Option<(MaterializedFact, Supersession)>;

    fn Invalidate(&mut self, cause: &GenerationCause, from: GenerationId) -> InvalidationReport;
}
