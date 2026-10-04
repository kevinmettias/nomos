//! [`OptionalRead`], one value a rule reads from a family a repository may leave undeclared.
//!
//! `OD-RULES-011` version 3 decision 5 places the declaration on the rule's descriptor, as the one
//! statement its body also resolves by -- the placement `OD-ANALYSIS-012` version 2 decision 1 gave
//! a population, for the same reason: a second statement beside the body's own is the defect
//! `OD-GATE-020` measured. A row names its reads with [`super::RuleDescriptor::Reading`], each a
//! constant in `checks::optional_reads`, and the body hands that same constant to its family's one
//! resolver. Whether the repository declared the value is answered after the rule is judged, from
//! the run's own facts, by [`OptionalRead::Undeclared`]: each family's resolver says it, so the axis
//! table stays this crate's own (`OD-RULES-035` decision 2) and the composition root interprets no
//! key.
//!
//! Decision 1's first three kinds are the families here. The words additions, the test-material
//! locations and the keys `undeclared-policy-key` reads are decision 2's, and have no read: a
//! declaration there only adds to a norm the rule owns, so nothing is substituted or withheld.

use super::policy_axis::{CaseRead, PolicyAxis};
use super::UndeclaredValue;
use crate::SourceFile;
use nomos_analysis::FactReader;

/// One value a rule reads from a family a repository may leave undeclared, declared on the rule's
/// descriptor and resolved by its body through this very value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OptionalRead(Read);

/// Which value, from which family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Read
{
    /// A limit, from `nomos-limits.json`.
    Limit(LimitRead),
    /// A case, from `standards.json`'s naming blocks.
    Case(NamingRead),
    /// `standards.json`'s `scripting.tooling_language`.
    ToolingLanguage,
    /// `standards.json`'s goals and their ceiling.
    Goals,
    /// The roots `nomos-standards-corpus.json` declares.
    StandardsCorpus,
    /// The corpus under `tests/contract/requirements/`.
    RequirementTrace,
}

/// A limit a rule reads, and the language it reads it for -- `None` for repository-wide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LimitRead
{
    pub(crate) axis: &'static PolicyAxis<u32>,
    pub(crate) language: Option<&'static str>,
}

/// A case a rule reads through one or more naming keys, and the language it reads it for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NamingRead
{
    pub(crate) read: CaseRead<'static>,
    pub(crate) language: ReadLanguage,
}

/// The language a case is read for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReadLanguage
{
    /// Repository-wide, whatever the source is written in.
    Repository,
    /// For one language.
    Language(&'static str),
    /// For the language each judged source is written in, as `function-naming-convention` reads a
    /// function's case in whichever language declares it.
    OfEachSource,
}

impl NamingRead
{
    /// The language this read looks a case up for, given the language of the source being judged.
    #[must_use]
    pub(crate) fn Language_For<'language>(&self, source: Option<&'language str>) -> Option<&'language str>
    {
        return match self.language
        {
            ReadLanguage::Repository => None,
            ReadLanguage::Language(language) => Some(language),
            ReadLanguage::OfEachSource => source,
        };
    }
}

impl OptionalRead
{
    /// `standards.json`'s `scripting.tooling_language`.
    pub(crate) const TOOLING_LANGUAGE: Self = Self(Read::ToolingLanguage);
    /// `standards.json`'s goals and their ceiling.
    pub(crate) const GOALS: Self = Self(Read::Goals);
    /// The roots `nomos-standards-corpus.json` declares.
    pub(crate) const STANDARDS_CORPUS: Self = Self(Read::StandardsCorpus);
    /// The corpus under `tests/contract/requirements/`.
    pub(crate) const REQUIREMENT_TRACE: Self = Self(Read::RequirementTrace);

    /// A limit read, as a row declares it.
    #[must_use]
    pub(crate) const fn Of_Limit(read: LimitRead) -> Self
    {
        return Self(Read::Limit(read));
    }

    /// A case read, as a row declares it.
    #[must_use]
    pub(crate) const fn Of_Case(read: NamingRead) -> Self
    {
        return Self(Read::Case(read));
    }

    /// Every value this read found undeclared in the repository whose facts `facts` reads, for a
    /// rule judged over `judged`: one for a read made repository-wide or for one language, and one
    /// per language `judged` is written in for a read made in each source's own language.
    ///
    /// Nothing when the family could not be read. That is coverage debt, which `OD-ANALYSIS-012`
    /// version 2 decision 2 has reported by its own finding, and not a value nobody declared.
    pub(crate) fn Undeclared(&self, facts: &mut dyn FactReader, judged: &[&SourceFile]) -> Vec<UndeclaredValue>
    {
        use crate::checks::{
            Undeclared_Cases, Undeclared_Goals, Undeclared_Limit_Value, Undeclared_Requirement_Trace,
            Undeclared_Standards_Corpus, Undeclared_Tooling_Language,
        };

        return match &self.0
        {
            Read::Limit(read) => Undeclared_Limit_Value(facts, read).into_iter().collect(),
            Read::Case(read) => Undeclared_Cases(facts, read, &Languages_Read(read, judged)),
            Read::ToolingLanguage => Undeclared_Tooling_Language(facts).into_iter().collect(),
            Read::Goals => Undeclared_Goals(facts).into_iter().collect(),
            Read::StandardsCorpus => Undeclared_Standards_Corpus(facts).into_iter().collect(),
            Read::RequirementTrace => Undeclared_Requirement_Trace(facts).into_iter().collect(),
        };
    }
}

/// The languages `read` looks a case up for over `judged`, each once, in the order first met.
///
/// For a read made in each source's own language, the languages its sources are written in. A
/// source no provider recognizes declares no name a casing rule judges, so no case was read for it.
fn Languages_Read<'judged>(read: &NamingRead, judged: &[&'judged SourceFile]) -> Vec<Option<&'judged str>>
{
    if read.language != ReadLanguage::OfEachSource
    {
        return vec![read.Language_For(None)];
    }

    let mut languages = Vec::new();
    for language in judged.iter().filter_map(|source| return source.language.as_ref().map(nomos_cap_syntax::Language::As_Str))
    {
        if !languages.contains(&Some(language))
        {
            languages.push(Some(language));
        }
    }

    return languages;
}

/// Notes that a rule's body resolved `read`, for the test that holds every body to the reads its
/// row declares.
#[cfg(test)]
pub(crate) fn Note_Read(read: OptionalRead)
{
    recorder::RESOLVED.with(|resolved| resolved.borrow_mut().push(read));
}

/// Outside this crate's own tests a body's reads are not noted: only the test that holds every
/// body to its row reads them.
#[cfg(not(test))]
pub(crate) const fn Note_Read(_read: OptionalRead)
{
}

/// What this thread's rule bodies resolved since it was last taken.
#[cfg(test)]
pub(crate) mod recorder
{
    use super::OptionalRead;
    use std::cell::RefCell;

    thread_local! {
        /// Every read noted on this thread, in the order the bodies made them.
        pub(super) static RESOLVED: RefCell<Vec<OptionalRead>> = const { RefCell::new(Vec::new()) };
    }

    /// Every read noted on this thread since the last call, once each, and forgets them.
    pub(crate) fn Take_Resolved() -> Vec<OptionalRead>
    {
        let noted = RESOLVED.with(|resolved| return resolved.take());
        let mut distinct = Vec::new();
        for read in noted
        {
            if !distinct.contains(&read)
            {
                distinct.push(read);
            }
        }

        return distinct;
    }
}

#[cfg(test)]
mod tests;
