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

    /// The same answer as [`Self::Current`], lent rather than copied.
    ///
    /// A fact carries its payload — a parse result, a finding set, kilobytes of it — so
    /// `Current` copies the whole thing to answer a question most callers settle by reading
    /// one field of it. A caller that only reads the fact takes it from here and copies
    /// nothing; a caller that has to keep it past the borrow uses `Current` and pays for
    /// what it keeps.
    ///
    /// The two answer the same question and are not allowed to drift: `Current` is defined
    /// as this, cloned. What a fact *is* does not change either — a borrow of the same
    /// [`MaterializedFact`] is what comes back, not a view onto the store's internals, which
    /// stay this crate's.
    fn Current_Borrowed(
        &self,
        identity: &FactIdentity,
        at: GenerationId,
    ) -> Option<&MaterializedFact>;

    fn Historical(&self, key: &FactKey) -> Option<(MaterializedFact, Supersession)>;

    fn Invalidate(&mut self, cause: &GenerationCause, from: GenerationId) -> InvalidationReport;
}
