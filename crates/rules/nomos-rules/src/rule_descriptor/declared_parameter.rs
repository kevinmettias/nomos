//! The repository-declared value a declared rule's detector reads, when one takes one.
//!
//! Its own file rather than a field of a bare axis on [`super::DeclaredTextRule`], because what
//! it records is not only which axis but why no detector reads one yet.

use super::policy_axis::PolicyAxis;

/// The limits axis a declared rule's detector is judged against.
///
/// `OD-RULES-034`'s eighth input, carried here rather than left out, and the one input the
/// vocabulary cannot yet consume. Measured while building the form, by reading every call
/// site of the two engines the record names: `Resolve_Limit`'s eight and `Resolve_Case`'s
/// seven all belong to `SubjectKind::SourceFacts` rules -- `function_shape`, `naming`,
/// `nesting_depth` and `structure` -- and not one of them is a per-line predicate over raw
/// text. So no detector in [`super::DeclaredDetector`] takes a parameter today, and a
/// declaration naming one is refused by [`super::DeclaredTextRule::Is_Well_Formed`] rather
/// than resolved and silently dropped, which is the failure `OD-RULES-034` forbids by name.
///
/// The observation that would change it is the first per-line detector whose judgment moves
/// with a repository-declared number -- at which point [`super::DeclaredDetector`] grows a
/// variant that answers [`super::DeclaredDetector::Is_Taking_A_Parameter`] with `true`, and
/// the refusal stops firing for it alone.
///
/// It names an axis rather than carrying a key and a default of its own, because
/// `OD-RULES-035` decided an axis is declared once: the key a repository writes and the value
/// an undeclared one is judged against are the axis's, in [`super::policy_axis`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DeclaredParameter
{
    /// The axis a repository declares this rule's value under.
    pub(crate) axis: &'static PolicyAxis<u32>,
}
