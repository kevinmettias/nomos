//! The repository-declared value a declared rule's detector reads, when one takes one.
//!
//! Its own file rather than two fields on [`super::DeclaredTextRule`]: a key and a default
//! are one input and mean nothing apart, which is exactly how `checks::structure`'s
//! `Resolve_Limit` and `checks::naming`'s `Resolve_Case` already take them.

/// A policy key and the value a repository that declares nothing is judged against.
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DeclaredParameter
{
    /// The policy key a repository declares this rule's value under.
    pub(crate) key: &'static str,
    /// What the rule is judged against where a repository declares nothing.
    pub(crate) default: u32,
}
