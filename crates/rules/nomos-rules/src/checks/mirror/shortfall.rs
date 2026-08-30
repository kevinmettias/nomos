//! The ways a universe falls short of the mirror it claims.

use super::Applicability;

/// Why a claim that failed to resolve is not thereby established false.
///
/// Two shapes and not one flag, because they are different sentences with different
/// remedies, and because only one of them is a statement about the claim.
pub(super) enum Shortfall<'index>
{
    /// Nothing admitted could answer for any subject, so there is no index at all.
    ///
    /// Not a claim about this name: a name cannot be established absent from an index that
    /// was never built. This one is whole-run by construction rather than by policy —
    /// `Resolution::Applicability` reads the registry and the requirement, and neither
    /// varies by subject, so `MissingCapability` holds for every subject of a run or for
    /// none of them. The remedy is registering or installing a provider, which is why it
    /// must not be reported as the narrower "the store had nothing for this subject".
    NoIndex,
    /// The index is short at least one subject whose own text spells the claimed name.
    Withheld
    {
        /// What the reader said about the first of them.
        ///
        /// The first in source order, and the summary names all of them. The two values
        /// that can appear here — `DependencyUnavailable` and `Unparseable` — are both
        /// statements about one file, so neither outranks the other and a deterministic pick
        /// is the honest one. `MissingCapability` cannot appear: it is
        /// [`Shortfall::NoIndex`].
        applicability: Applicability,
        /// Where they are, for the reader.
        subjects: Vec<&'index str>,
    },
}

impl Shortfall<'_>
{
    /// How a claim this shortfall bears on must be reported.
    pub(super) fn Applicability(&self) -> Applicability
    {
        return match *self
        {
            Self::NoIndex => Applicability::MissingCapability,
            Self::Withheld { applicability, .. } => applicability,
        };
    }

    /// Why the claim is not established false, in terms somebody can act on.
    ///
    /// Names the subjects rather than counting them. `OD-RULES-002`'s `done_when` asks the
    /// record to state how the scoping is decided; a finding that said only "the index is
    /// incomplete" would leave the reader unable to check that decision against the tree.
    pub(super) fn Describe(&self) -> String
    {
        return match self
        {
            Self::NoIndex => "no admitted provider could answer for any subject, so there is \
                              no check index for this name to be absent from and this rule \
                              cannot tell a false claim of coverage from a name it did not \
                              get to look for"
                .to_owned(),
            Self::Withheld { subjects, .. } => format!(
                "the check index is short {}, whose text spells this name, so this rule \
                 cannot tell a false claim of coverage from a name it did not get to look for",
                subjects.join(", ")
            ),
        };
    }
}
