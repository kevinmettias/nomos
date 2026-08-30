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
pub enum Availability
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

impl Availability
{
    /// Whether this response may be treated as knowledge.
    ///
    /// Only [`Availability::Answered`]. A partial answer is knowledge about what it
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

#[cfg(test)]
mod tests
{
    use super::*;

    /// The whole reason the type exists: an outage must not be indistinguishable from
    /// an empty result.
    #[test]
    fn Test_Silence_Should_Never_Read_As_Knowledge()
    {
        let outage = Availability::Unavailable {
            reason: "connection refused".to_owned(),
        };
        let partial = Availability::Partial {
            reason: "index rebuilding".to_owned(),
        };

        assert!(!outage.Is_Knowledge());
        assert!(!partial.Is_Knowledge());
        assert!(Availability::Answered.Is_Knowledge());
    }

    /// A reason that never reaches the user leaves them staring at an empty list with
    /// no way to tell why.
    #[test]
    fn Test_Unavailability_Should_Carry_Its_Reason_Into_The_Description()
    {
        let outage = Availability::Unavailable {
            reason: "connection refused".to_owned(),
        };

        assert!(outage.Describe().contains("connection refused"));
    }
}
