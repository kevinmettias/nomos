//! Which files a `nomos gate run` judges.

/// Which files under [`crate::GateCommand::root`] a run judges.
///
/// Textual path-prefix containment, not a glob engine -- the same choice `OD-LEDGER-013`
/// already made for ledger territory, and for the identical soundness reason: a wrong
/// `Disjoint` there costs an edit that does not come back, and a wrong exclusion here costs
/// a file silently going unjudged. `include`/`exclude` compare against
/// [`nomos_rules::SourceFile::path`] -- repo-relative, forward slashes -- one entry per
/// prefix, no `*`/`**` syntax to get subtly wrong.
///
/// Both lists empty is "select everything," the state every existing caller is in today:
/// `Default` gives that state, so `GateCommand`'s other construction sites and CI's `gate
/// run --root .` are unchanged in behavior by this type existing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScopeSelector
{
    /// Prefixes a file's path must match at least one of, or every file matches when this
    /// is empty.
    pub include: Vec<String>,
    /// Prefixes that remove a file even if `include` matched it.
    pub exclude: Vec<String>,
}

impl ScopeSelector
{
    /// Whether `path` is in scope: included (or nothing was named, so everything is) and
    /// not excluded.
    #[must_use]
    pub fn Is_In_Scope(&self, path: &str) -> bool
    {
        let included = self.include.is_empty() || self.include.iter().any(|prefix| return Is_Under(Prefix(prefix), path));
        let excluded = self.exclude.iter().any(|prefix| return Is_Under(Prefix(prefix), path));

        return included && !excluded;
    }
}

/// One entry from [`ScopeSelector::include`] or [`ScopeSelector::exclude`] -- named so
/// [`Is_Under`] cannot mistake the containing prefix for the path being tested, since both
/// are otherwise identically-shaped `&str`s.
struct Prefix<'a>(&'a str);

/// Whether `path` is `prefix` itself or lives under it, textually -- the same containment
/// `README.md`'s territory rules already use, not a filesystem check.
fn Is_Under(prefix: Prefix<'_>, path: &str) -> bool
{
    return path == prefix.0 || path.starts_with(&format!("{}/", prefix.0));
}

#[cfg(test)]
mod tests
{
    use super::ScopeSelector;

    #[test]
    fn Test_Is_In_Scope_Should_Match_Everything_When_Empty()
    {
        let selector = ScopeSelector::default();

        assert!(selector.Is_In_Scope("crates/rules/nomos-rules/src/lib.rs"));
        assert!(selector.Is_In_Scope("README.md"));
    }

    /// One path inside `crates/rules` and one outside it, against a selector whose only
    /// `include` entry is that directory -- the shape [`Test_Include_Should_Admit_Only_Its_Own_Subtree`]
    /// checks against.
    fn Rules_Subtree_Paths() -> Vec<(&'static str, bool)>
    {
        return vec![
            ("crates/rules/nomos-rules/src/lib.rs", true),
            ("crates/host/nomos-cli/src/gate.rs", false),
        ];
    }

    #[test]
    fn Test_Include_Should_Admit_Only_Its_Own_Subtree()
    {
        let selector = ScopeSelector { include: vec!["crates/rules".to_owned()], exclude: Vec::new() };

        for (path, expected) in Rules_Subtree_Paths()
        {
            assert_eq!(selector.Is_In_Scope(path), expected, "path: {path}");
        }
    }

    /// The exact file an `include` entry names, and a different file that merely shares its
    /// name as a prefix -- the shape [`Test_An_Exact_File_Should_Match_Its_Own_Include_Entry`]
    /// checks against.
    fn Exact_File_Paths() -> Vec<(&'static str, bool)>
    {
        return vec![("README.md", true), ("README.md.bak", false)];
    }

    #[test]
    fn Test_An_Exact_File_Should_Match_Its_Own_Include_Entry()
    {
        let selector = ScopeSelector { include: vec!["README.md".to_owned()], exclude: Vec::new() };

        for (path, expected) in Exact_File_Paths()
        {
            assert_eq!(selector.Is_In_Scope(path), expected, "path: {path}");
        }
    }

    #[test]
    fn Test_Exclude_Should_Win_Over_A_Matching_Include()
    {
        let selector = ScopeSelector {
            include: vec!["crates/rules".to_owned()],
            exclude: vec!["crates/rules/nomos-rules/tests".to_owned()],
        };

        assert!(selector.Is_In_Scope("crates/rules/nomos-rules/src/lib.rs"));
        assert!(!selector.Is_In_Scope("crates/rules/nomos-rules/tests/corpus.rs"));
    }
}
