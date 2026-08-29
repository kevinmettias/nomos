//! What this store was assembled from, and what it is missing.

use crate::corpus::Absence;

/// What went into this store, and what did not.
///
/// The rendering-free half of `nomos-cli::spec::verb::listing::Sources`: everything that
/// function used to write directly, kept here as data so a second caller can render it its
/// own way instead of parsing the lines `nomos-cli` happened to print.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourcesAnswer
{
    /// One line per input the assembly read, in the order it was read.
    pub read: Vec<String>,
    /// Every input the assembly expected and did not find.
    pub absent: Vec<Absence>,
}

impl SourcesAnswer
{
    /// Whether anything this store was expected to hold is missing.
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
