//! How a thing became a different thing, and how sure we are.

use serde::{Deserialize, Serialize};

use crate::confidence::Confidence;

/// A typed, evidenced change of state.
///
/// Generic because the same shape serves identity changes, finding lifecycle changes
/// and synchronization state changes: in every case what matters is *what* changed,
/// *how sure* we are, and *what makes us think so*. Three copies of that shape would be
/// three places for the evidence field to be forgotten.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transition<K>
{
    /// What kind of change this is.
    pub kind: K,
    /// How sure we are.
    pub confidence: Confidence,
    /// What supports the conclusion. An inference with nothing behind it is a guess,
    /// and this field is where that becomes visible instead of implied.
    pub evidence: Vec<crate::EvidenceRef>,
}

impl<K> Transition<K>
{
    /// A transition asserted without supporting evidence.
    ///
    /// Legitimate for [`IdentityTransitionKind::ExactContinuity`], where the absence of
    /// change is the evidence. Anything else constructed this way is an assertion, and
    /// the empty evidence list is what says so.
    #[must_use]
    pub const fn Asserted(kind: K, confidence: Confidence) -> Self
    {
        return Self {
            kind,
            confidence,
            evidence: Vec::new(),
        };
    }

    /// Whether this transition is supported by anything.
    #[must_use]
    pub fn Is_Evidenced(&self) -> bool
    {
        return !self.evidence.is_empty();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::identity::IdentityTransition;
    use crate::identity::IdentityTransitionKind;

    /// An unevidenced inference must be visibly unevidenced. This is the field a review
    /// looks at when asking why the system thinks two declarations are the same one.
    #[test]
    fn Test_An_Asserted_Transition_Should_Report_Itself_As_Unevidenced()
    {
        let asserted = IdentityTransition::Asserted(
            IdentityTransitionKind::ProbableRename,
            Confidence::Of(0.8),
        );

        assert!(!asserted.Is_Evidenced());
    }
}
