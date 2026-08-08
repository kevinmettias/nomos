//! Talking to another system without letting its silence read as agreement.

use serde::{Deserialize, Serialize};

/// Whether a peer answered a question.
///
/// The distinction this type protects is between "the knowledge service says there are
/// no related decisions" and "the knowledge service did not answer". Both render as an
/// empty list, and only one of them is information.
///
/// There is deliberately no `Default`. A peer response that was never populated must
/// not construct itself as an answer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerAvailability
{
    /// The peer answered. Only this variant may be read as a statement about the world.
    Answered,
    /// The peer did not answer, for the stated reason.
    Unavailable
    {
        /// Why the peer could not answer, in terms a user can act on.
        reason: String,
    },
    /// The peer answered in part, for the stated reason. What is missing is missing —
    /// not empty.
    Partial
    {
        /// What the peer could not cover, and why.
        reason: String,
    },
}

impl PeerAvailability
{
    /// Whether this response may be treated as knowledge.
    ///
    /// Only [`PeerAvailability::Answered`]. A partial answer is knowledge about what it
    /// covered and silence about the rest, so a caller must handle it explicitly rather
    /// than through this predicate.
    #[must_use]
    pub const fn Is_Knowledge(&self) -> bool
    {
        return matches!(self, Self::Answered);
    }

    /// A short description for display and diagnostics.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::Answered => "Answered".to_owned(),
            Self::Unavailable { reason } => format!("Unavailable: {reason}"),
            Self::Partial { reason } => format!("Partial: {reason}"),
        };
    }
}

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
    pub const fn Requires_Authorized_Resolution(self) -> bool
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

    /// The whole reason the type exists: an outage must not be indistinguishable from
    /// an empty result.
    #[test]
    fn Test_Silence_Should_Never_Read_As_Knowledge()
    {
        let outage = PeerAvailability::Unavailable {
            reason: "connection refused".to_owned(),
        };
        let partial = PeerAvailability::Partial {
            reason: "index rebuilding".to_owned(),
        };

        assert!(!outage.Is_Knowledge());
        assert!(!partial.Is_Knowledge());
        assert!(PeerAvailability::Answered.Is_Knowledge());
    }

    /// A reason that never reaches the user leaves them staring at an empty list with
    /// no way to tell why.
    #[test]
    fn Test_Unavailability_Should_Carry_Its_Reason_Into_The_Description()
    {
        let outage = PeerAvailability::Unavailable {
            reason: "connection refused".to_owned(),
        };

        assert!(outage.Describe().contains("connection refused"));
    }

    #[test]
    fn Test_Divergence_Should_Require_An_Authorized_Decision()
    {
        assert!(SynchronizationState::Diverged.Requires_Authorized_Resolution());
        assert!(SynchronizationState::Rejected.Requires_Authorized_Resolution());
        assert!(!SynchronizationState::Current.Requires_Authorized_Resolution());
        assert!(!SynchronizationState::SourceNewer.Requires_Authorized_Resolution());
    }

    /// `Unavailable` is not agreement, and it is not divergence either — it is the
    /// absence of an answer, and it must not be auto-resolved in either direction.
    #[test]
    fn Test_Unavailable_Should_Not_Read_As_Current()
    {
        assert_ne!(SynchronizationState::Unavailable, SynchronizationState::Current);
        assert!(!SynchronizationState::Unavailable.Requires_Authorized_Resolution());
    }
}
