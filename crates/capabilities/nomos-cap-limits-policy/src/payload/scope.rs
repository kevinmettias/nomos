//! Which part of a repository's declaration one [`super::policy_row::PolicyRow`] came
//! from.

const REPOSITORY_LABEL: &str = "*";

/// A row's own scope: the repository-wide default, or one language's own override.
///
/// Mirrors `standards.json`'s own shape — a top-level block and a `languages.<name>.*`
/// block that refines it — the same split `nomos_cap_naming_policy::Scope` already draws
/// for the sibling capability, rather than flattening the two into one namespace a reader
/// could not tell apart again.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Scope
{
    /// From the repository-wide default.
    Repository,
    /// From a per-language override.
    Language(String),
}

impl Scope
{
    /// This scope's wire spelling: `*` for [`Scope::Repository`], the language's own name
    /// otherwise. A language is never named `*` by any provider this crate ships, so the
    /// two can never collide.
    #[must_use]
    pub fn Label(&self) -> &str
    {
        return match self
        {
            Self::Repository => REPOSITORY_LABEL,
            Self::Language(name) => name,
        };
    }

    /// The scope `label` names: [`Scope::Repository`] for `*`, [`Scope::Language`]
    /// otherwise. Never refuses — an unrecognized language name is still a language, just
    /// one no provider currently overrides for.
    #[must_use]
    pub fn From_Label(label: &str) -> Self
    {
        if label == REPOSITORY_LABEL
        {
            return Self::Repository;
        }
        return Self::Language(label.to_owned());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Label_Should_Round_Trip_Through_From_Label_For_Repository()
    {
        assert_eq!(Scope::From_Label(Scope::Repository.Label()), Scope::Repository);
    }

    #[test]
    fn Test_Label_Should_Round_Trip_Through_From_Label_For_A_Language()
    {
        let scope = Scope::Language("go".to_owned());
        assert_eq!(Scope::From_Label(scope.Label()), scope);
    }
}
