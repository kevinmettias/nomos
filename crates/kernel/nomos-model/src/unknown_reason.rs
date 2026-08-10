//! Why an overlap question could not be answered.

use serde::{Deserialize, Serialize};

use crate::set_resolution::SetResolution;

/// Why an overlap question could not be answered.
///
/// Every variant is a reason a caller can act on, because the response to "we cannot
/// resolve symbols yet" is different from the response to "these sets were computed
/// against different snapshots".
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnknownReason
{
    /// The two sets are stated at different granularities and neither can be lowered to
    /// the other without inventing membership.
    IncomparableResolution
    {
        /// The resolution of the first set.
        left: SetResolution,
        /// The resolution of the second set.
        right: SetResolution,
    },
    /// A set names a resolution the system cannot currently compute membership at.
    ResolutionUnavailable
    {
        /// The resolution that was asked for.
        requested: SetResolution,
    },
    /// The sets were derived from different snapshots, so their members are not
    /// comparable even where the identifiers look equal.
    IncomparableSnapshots,
    /// A set includes a pattern whose expansion is not known without touching the
    /// filesystem, and the caller asked for an answer without doing so.
    UnexpandedPattern
    {
        /// The pattern that was not expanded.
        pattern: String,
    },
}

impl UnknownReason
{
    /// A one-line description naming what is unknown and why.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::IncomparableResolution { left, right } => format!(
                "sets are stated at {left} and {right} resolution and cannot be compared \
                 without inventing membership"
            ),
            Self::ResolutionUnavailable { requested } => {
                format!("{requested}-resolution membership cannot be computed yet")
            }
            Self::IncomparableSnapshots =>
            {
                "sets were derived from different snapshots, so equal identifiers are not \
                 necessarily the same subject"
                    .to_owned()
            }
            Self::UnexpandedPattern { pattern } => {
                format!("pattern `{pattern}` was not expanded, so its members are not known")
            }
        };
    }
}
