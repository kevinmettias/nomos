//! How a thing became a different thing, and how sure we are.

use serde::{Deserialize, Serialize};

/// How much weight a claim carries, on a stated scale.
///
/// A bounded value with an explicit constructor rather than a bare `f64`, so that a
/// confidence can never be constructed outside its range and never compared with `==`.
/// Both of those are lint-level errors in this workspace, and both were real defects in
/// the prototype.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Confidence(f64);

impl Confidence
{
    /// Complete confidence.
    pub const CERTAIN: Self = Self(1.0);

    /// No confidence at all.
    pub const NONE: Self = Self(0.0);

    /// Constructs a confidence, clamping to the unit interval.
    ///
    /// Clamps rather than rejecting: a provider that computed 1.0000001 through
    /// floating-point accumulation has not made an error worth failing a run over, and
    /// the alternative is every caller writing its own clamp slightly differently.
    #[must_use]
    pub fn Of(value: f64) -> Self
    {
        if value.is_nan()
        {
            return Self::NONE;
        }
        let clamped = value.clamp(0.0, 1.0);

        return Self(clamped);
    }

    /// The value, in the unit interval.
    #[must_use]
    pub const fn Value(self) -> f64
    {
        return self.0;
    }

    /// Whether this confidence is at least `threshold`.
    #[must_use]
    pub fn Meets(self, threshold: Self) -> bool
    {
        return self.0 >= threshold.0;
    }
}

/// What happened to a thing's identity between two snapshots.
///
/// Recording the *kind* of change is what lets a finding, a suppression and a metric
/// history follow their subject through a rename instead of being orphaned by it. The
/// prototype could not express any of these, so every one of them presented as a
/// deletion followed by an unrelated arrival.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdentityTransitionKind
{
    /// The same thing, unchanged.
    ExactContinuity,
    /// Probably the same thing under a new name.
    ProbableRename,
    /// Probably the same thing in a new location.
    ProbableMove,
    /// The same thing with a changed signature.
    SignatureEvolution,
    /// One thing became several.
    SplitInto,
    /// Several things became one.
    MergedFrom,
    /// A thing with this identity existed, went away, and something with the same
    /// identity came back. Not continuity, and it must not be reported as such.
    Recreated,
    /// Produced by a generator from a source that is itself the thing to track.
    GeneratedFrom,
    /// Continuity could not be established either way.
    Unresolved,
}

impl IdentityTransitionKind
{
    /// Whether this transition preserves the subject's accumulated history.
    ///
    /// [`IdentityTransitionKind::Recreated`] and
    /// [`IdentityTransitionKind::Unresolved`] deliberately do not. Carrying a
    /// suppression across a recreation would silence a finding on code nobody has
    /// reviewed, and carrying one across an unresolved link would do so on the strength
    /// of a guess.
    #[must_use]
    pub const fn Preserves_History(self) -> bool
    {
        return matches!(
            self,
            Self::ExactContinuity
                | Self::ProbableRename
                | Self::ProbableMove
                | Self::SignatureEvolution
        );
    }
}

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

/// A change of identity between two snapshots.
pub type IdentityTransition = Transition<IdentityTransitionKind>;

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Confidence_Should_Clamp_To_The_Unit_Interval()
    {
        assert!(Confidence::Of(1.5).Value() <= 1.0);
        assert!(Confidence::Of(-0.5).Value() >= 0.0);
    }

    /// A NaN confidence compares false against everything, including any threshold, so
    /// it would silently behave as "never good enough" in one place and as "never
    /// rejected" in another depending on how the comparison was written.
    #[test]
    fn Test_Nan_Confidence_Should_Become_None()
    {
        assert!(Confidence::Of(f64::NAN).Value() <= 0.0);
        assert!(!Confidence::Of(f64::NAN).Meets(Confidence::Of(0.1)));
    }

    /// The rule that stops a suppression from surviving into code nobody reviewed.
    #[test]
    fn Test_Recreation_Should_Not_Preserve_History()
    {
        assert!(!IdentityTransitionKind::Recreated.Preserves_History());
        assert!(!IdentityTransitionKind::Unresolved.Preserves_History());
        assert!(!IdentityTransitionKind::SplitInto.Preserves_History());
        assert!(!IdentityTransitionKind::MergedFrom.Preserves_History());
    }

    #[test]
    fn Test_Renames_And_Moves_Should_Preserve_History()
    {
        assert!(IdentityTransitionKind::ExactContinuity.Preserves_History());
        assert!(IdentityTransitionKind::ProbableRename.Preserves_History());
        assert!(IdentityTransitionKind::ProbableMove.Preserves_History());
        assert!(IdentityTransitionKind::SignatureEvolution.Preserves_History());
    }

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
