//! [`Population`], the sources a rule judges, as its descriptor declares them.
//!
//! `OD-ANALYSIS-012` version 2's first decision. A rule's population is a value fixed when the rule
//! is written and never computed from run state, it is declared on the rule's descriptor, and the
//! rule's body filters the sources it is handed by that very value -- one statement, so the
//! descriptor and the body cannot state two different populations. The composition root reads it
//! only to count: every body still receives exactly the sources it always received, and the root
//! partitions nothing, which `OD-RULES-014` declined.
//!
//! The grain is the language or kind a rule's norm is about, and nothing finer. What a body leaves
//! out within that kind -- its own implementation file, test material, a Go file that is not a
//! test -- is judgment, and a population wholly left out that way still counts as judged.

use crate::SourceFile;

/// Which of the sources a rule is handed it judges.
#[derive(Clone, Copy, Debug)]
pub enum Population
{
    /// Every source the rule is handed: the walk, or the capability family's slice a rule reads
    /// instead of it. The default, and right for every rule whose norm is about any file.
    Every,
    /// Every source written in one language, as the composition root recognized it.
    Language(&'static str),
    /// Every source written in any of these languages.
    Languages(&'static [&'static str]),
    /// Every source of a kind no language names, decided by a predicate the rule's own module
    /// declares -- a script recognized by its first line, say.
    Kind
    {
        /// What the kind is called, as a report names it.
        name: &'static str,
        /// Whether a source is of the kind.
        holds: fn(&SourceFile) -> bool,
    },
}

impl Population
{
    /// Whether `source` is in this population.
    #[must_use]
    pub fn Holds(&self, source: &SourceFile) -> bool
    {
        return match self
        {
            Self::Every => true,
            Self::Language(language) => source.Is_Written_In(language),
            Self::Languages(languages) => languages.iter().any(|language| return source.Is_Written_In(language)),
            Self::Kind { holds, .. } => holds(source),
        };
    }

    /// The population as a report names it: "every source", "rust sources", "rust or go
    /// sources", or a kind's own name.
    #[must_use]
    pub fn Name(&self) -> String
    {
        return match self
        {
            Self::Every => "every source".to_owned(),
            Self::Language(language) => format!("{language} sources"),
            Self::Languages(languages) => format!("{} sources", languages.join(" or ")),
            Self::Kind { name, .. } => (*name).to_owned(),
        };
    }
}

#[cfg(test)]
mod tests;
