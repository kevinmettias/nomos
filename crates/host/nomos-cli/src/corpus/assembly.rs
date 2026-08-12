//! A store assembled from a corpus, and what it is missing.

use crate::corpus::Absence;
use nomos_spec_store::SpecificationStore;
/// A store, what went into it, and what did not.
pub(crate) struct Assembly
{
    pub store: SpecificationStore,
    /// One line per input that was read, in the order it was read.
    pub read: Vec<String>,
    pub absent: Vec<Absence>,
}

impl Assembly
{
    /// Whether anything the store was expected to hold is missing.
    #[must_use]
    pub fn Is_Complete(&self) -> bool
    {
        return self.absent.is_empty();
    }

    /// Every absence, one after another.
    #[must_use]
    pub fn Describe_Absences(&self) -> String
    {
        return self
            .absent
            .iter()
            .map(Absence::Describe)
            .collect::<Vec<String>>()
            .join("\n");
    }
}
