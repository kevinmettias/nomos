//! How finely a subject set was resolved.

use serde::{Deserialize, Serialize};

const FILE_LABEL: &str = "File";

const REGION_LABEL: &str = "Region";

const SYMBOL_LABEL: &str = "Symbol";

/// The granularity at which membership of a [`SubjectSet`] is decided.
///
/// Nomos starts at [`SetResolution::File`] — one writer per file — and stays there
/// until the architecture graph can produce region facts with a proven guarantee.
/// Requests at a finer resolution than the system can actually resolve do not silently
/// degrade to file comparison; they return [`Intersection::Unknown`], which forces
/// serialization. Claiming precision you do not have is how two edits to one function
/// get applied concurrently.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SetResolution
{
    /// Whole files. The only resolution available today.
    File,
    /// Named symbols within a file.
    Symbol,
    /// Sub-symbol regions.
    Region,
}

impl SetResolution
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::File => FILE_LABEL,
            Self::Symbol => SYMBOL_LABEL,
            Self::Region => REGION_LABEL,
        };
    }
}

impl core::fmt::Display for SetResolution
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}
