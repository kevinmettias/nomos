//! The store contract, sealed so its invariants stay this crate's to keep.

use nomos_contracts::GenerationId;
use crate::invalidation::InvalidationReport;
use crate::store::GenerationCause;
use crate::invalidation::Supersession;
use crate::fact::FactKey;
use crate::fact::MaterializedFact;
use crate::fact::FactIdentity;

pub(crate) mod sealed
{
    pub trait Sealed
    {}
}

pub trait FactStore: sealed::Sealed
{
    fn Current(&self, identity: &FactIdentity, at: GenerationId) -> Option<MaterializedFact>;

    fn Historical(&self, key: &FactKey) -> Option<(MaterializedFact, Supersession)>;

    fn Invalidate(&mut self, cause: &GenerationCause, from: GenerationId) -> InvalidationReport;
}
