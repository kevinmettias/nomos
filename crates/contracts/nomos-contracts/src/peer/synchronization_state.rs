use serde::{Deserialize, Serialize};

const CURRENT_LABEL: &str = "Current";
const SOURCE_NEWER_LABEL: &str = "SourceNewer";
const OPERATIONAL_NEWER_LABEL: &str = "OperationalNewer";
const DIVERGED_LABEL: &str = "Diverged";
const REJECTED_LABEL: &str = "Rejected";
const UNAVAILABLE_LABEL: &str = "Unavailable";

/// The relationship between an authored record and the operational projection of it.
///
/// **No last-writer-wins synchronization is permitted.** Divergence stays explicit
/// until an authorized action resolves it, which is why
/// [`SynchronizationState::Diverged`] is a state a system can sit in rather than an
/// error a system recovers from by picking a side.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SynchronizationState
{
    /// Both sides agree.
    Current,
    /// The authoring side has moved ahead; the operational projection is stale.
    SourceNewer,
    /// The operational side has moved ahead; the authored record has not caught up.
    OperationalNewer,
    /// Both sides changed independently. Requires an authorized decision; nothing here
    /// may be resolved by timestamp.
    Diverged,
    /// A proposed synchronization was refused, and the refusal stands as the state.
    Rejected,
    /// The relationship could not be determined. Not an agreement.
    Unavailable,
}

impl SynchronizationState
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Current => CURRENT_LABEL,
            Self::SourceNewer => SOURCE_NEWER_LABEL,
            Self::OperationalNewer => OPERATIONAL_NEWER_LABEL,
            Self::Diverged => DIVERGED_LABEL,
            Self::Rejected => REJECTED_LABEL,
            Self::Unavailable => UNAVAILABLE_LABEL,
        };
    }

    /// Whether this state requires a human decision before synchronization proceeds.
    #[must_use]
    pub const fn Is_Authorized_Resolution_Required(self) -> bool
    {
        return matches!(self, Self::Diverged | Self::Rejected);
    }
}

impl core::fmt::Display for SynchronizationState
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
    fn Test_Divergence_Should_Require_An_Authorized_Decision()
    {
        assert!(SynchronizationState::Diverged.Is_Authorized_Resolution_Required());
        assert!(SynchronizationState::Rejected.Is_Authorized_Resolution_Required());
        assert!(!SynchronizationState::Current.Is_Authorized_Resolution_Required());
        assert!(!SynchronizationState::SourceNewer.Is_Authorized_Resolution_Required());
    }

    /// `Unavailable` is not agreement, and it is not divergence either — it is the
    /// absence of an answer, and it must not be auto-resolved in either direction.
    #[test]
    fn Test_Unavailable_Should_Not_Read_As_Current()
    {
        assert_ne!(SynchronizationState::Unavailable, SynchronizationState::Current);
        assert!(!SynchronizationState::Unavailable.Is_Authorized_Resolution_Required());
    }
}
