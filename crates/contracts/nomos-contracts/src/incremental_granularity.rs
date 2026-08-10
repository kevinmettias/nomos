//! The finest region a provider can refresh without recomputing everything.

use serde::{Deserialize, Serialize};

const NONE_GRANULARITY_LABEL: &str = "None";
const WHOLE_WORKSPACE_LABEL: &str = "WholeWorkspace";
const PROJECT_LABEL: &str = "Project";
const FILE_LABEL: &str = "File";
const SYMBOL_LABEL: &str = "Symbol";
const REGION_LABEL: &str = "Region";

/// The finest granularity at which a provider can update its output incrementally.
///
/// The invalidation engine computes the logically minimal affected region and then
/// **broadens** it to whatever the selected provider can actually deliver. Recording
/// this per provider is what stops the engine from assuming a precision no tool has —
/// a compiler-backed provider that can only refresh a whole project is common, and
/// asking it for a symbol-level update silently gets you a stale answer.
///
/// Ordered coarsest first, so broadening is a `max`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IncrementalGranularity
{
    /// No incremental capability. Any change means a full recomputation.
    None,
    /// The whole workspace refreshes together.
    WholeWorkspace,
    /// One project or compilation unit refreshes together.
    Project,
    /// One file refreshes independently.
    File,
    /// One symbol refreshes independently.
    Symbol,
    /// A sub-symbol region refreshes independently.
    Region,
}

impl IncrementalGranularity
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::None => NONE_GRANULARITY_LABEL,
            Self::WholeWorkspace => WHOLE_WORKSPACE_LABEL,
            Self::Project => PROJECT_LABEL,
            Self::File => FILE_LABEL,
            Self::Symbol => SYMBOL_LABEL,
            Self::Region => REGION_LABEL,
        };
    }

    /// The granularity actually achievable when a region computed at `self` must be
    /// refreshed by a provider capable of `provider`.
    ///
    /// Always the coarser of the two. Asking for finer than a provider offers does not
    /// get you finer; it gets you wrong.
    #[must_use]
    pub fn Broadened_To(self, provider: Self) -> Self
    {
        return core::cmp::min(self, provider);
    }
}

impl core::fmt::Display for IncrementalGranularity
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Broadening_Should_Take_The_Coarser_Granularity()
    {
        assert_eq!(
            IncrementalGranularity::Region.Broadened_To(IncrementalGranularity::Project),
            IncrementalGranularity::Project
        );
        assert_eq!(
            IncrementalGranularity::Project.Broadened_To(IncrementalGranularity::Region),
            IncrementalGranularity::Project
        );
        assert_eq!(
            IncrementalGranularity::Symbol.Broadened_To(IncrementalGranularity::None),
            IncrementalGranularity::None
        );
    }
}
