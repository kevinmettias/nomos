//! What an invalidation reached, and where it had to widen.

use nomos_contracts::GenerationId;
use crate::broadening::Broadening;
use crate::fact_key::FactKey;
use crate::store::GenerationCause;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvalidationReport
{
    pub cause: GenerationCause,
    pub from: GenerationId,
    pub direct: Vec<FactKey>,
    pub dependent: Vec<FactKey>,
    pub broadened: Vec<Broadening>,
    pub retained: u32,
}

impl InvalidationReport
{
    #[must_use]
    pub fn Invalidated(&self) -> usize
    {
        return self.direct.len().saturating_add(self.dependent.len());
    }

    #[must_use]
    pub fn Report(&self) -> String
    {
        return format!(
            "{} invalidated {} directly and {} through dependency edges at {}, retaining {}",
            self.cause.Describe(),
            self.direct.len(),
            self.dependent.len(),
            self.from,
            self.retained
        );
    }
}
