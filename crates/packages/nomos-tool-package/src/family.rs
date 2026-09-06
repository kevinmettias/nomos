//! The closed FAMILY vocabulary a `ToolProvider`'s own registration classifies itself
//! under.
//!
//! `OD-CAPABILITY-013`'s decision: this workspace adopts `toolspec`'s twelve names
//! verbatim rather than inventing a second taxonomy for the same idea, and a family
//! classifies a provider's own registration -- never something a rule names, since a rule
//! already names only a capability contract. Mirrors `nomos-lang-rust-package`'s own
//! `RustEdition::Of_Label` pattern exactly: a manual match over the record's exact
//! `SCREAMING_SNAKE_CASE` spellings, not a serde derive mapping this enum's Rust variant
//! names onto that wire form.

const PARSER_LABEL: &str = "PARSER";
const SEMANTIC_MODEL_LABEL: &str = "SEMANTIC_MODEL";
const COMPILER_LABEL: &str = "COMPILER";
const LINTER_LABEL: &str = "LINTER";
const TYPE_CHECKER_LABEL: &str = "TYPE_CHECKER";
const FORMATTER_LABEL: &str = "FORMATTER";
const LANGUAGE_SERVER_LABEL: &str = "LANGUAGE_SERVER";
const REFACTORING_ENGINE_LABEL: &str = "REFACTORING_ENGINE";
const INDEXER_LABEL: &str = "INDEXER";
const TEST_RUNNER_LABEL: &str = "TEST_RUNNER";
const DOCUMENTATION_GENERATOR_LABEL: &str = "DOCUMENTATION_GENERATOR";
const PACKAGE_MANAGER_LABEL: &str = "PACKAGE_MANAGER";

/// One of `OD-CAPABILITY-013`'s twelve closed FAMILY names.
///
/// Applied to this workspace's own two real `ToolProvider`s: `nomos-lang-rust-clippy` is
/// [`Family::Linter`], and `nomos-lang-rust-deny` is [`Family::PackageManager`] -- a
/// dependency-graph auditor beside `nomos-lang-rust-cargo`'s dependency-graph reader,
/// judged there rather than as a source-code rule engine beside clippy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Family
{
    Parser,
    SemanticModel,
    Compiler,
    Linter,
    TypeChecker,
    Formatter,
    LanguageServer,
    RefactoringEngine,
    Indexer,
    TestRunner,
    DocumentationGenerator,
    PackageManager,
}

impl Family
{
    /// The label a manifest file spells this family with -- `OD-CAPABILITY-013`'s own
    /// `SCREAMING_SNAKE_CASE` spelling, transcribed from `toolspec` verbatim.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Parser => PARSER_LABEL,
            Self::SemanticModel => SEMANTIC_MODEL_LABEL,
            Self::Compiler => COMPILER_LABEL,
            Self::Linter => LINTER_LABEL,
            Self::TypeChecker => TYPE_CHECKER_LABEL,
            Self::Formatter => FORMATTER_LABEL,
            Self::LanguageServer => LANGUAGE_SERVER_LABEL,
            Self::RefactoringEngine => REFACTORING_ENGINE_LABEL,
            Self::Indexer => INDEXER_LABEL,
            Self::TestRunner => TEST_RUNNER_LABEL,
            Self::DocumentationGenerator => DOCUMENTATION_GENERATOR_LABEL,
            Self::PackageManager => PACKAGE_MANAGER_LABEL,
        };
    }

    /// The family a label names, or `None` if it names none of the twelve
    /// `OD-CAPABILITY-013` closes the vocabulary at.
    #[must_use]
    pub fn Of_Label(label: &str) -> Option<Self>
    {
        return match label
        {
            PARSER_LABEL => Some(Self::Parser),
            SEMANTIC_MODEL_LABEL => Some(Self::SemanticModel),
            COMPILER_LABEL => Some(Self::Compiler),
            LINTER_LABEL => Some(Self::Linter),
            TYPE_CHECKER_LABEL => Some(Self::TypeChecker),
            FORMATTER_LABEL => Some(Self::Formatter),
            LANGUAGE_SERVER_LABEL => Some(Self::LanguageServer),
            REFACTORING_ENGINE_LABEL => Some(Self::RefactoringEngine),
            INDEXER_LABEL => Some(Self::Indexer),
            TEST_RUNNER_LABEL => Some(Self::TestRunner),
            DOCUMENTATION_GENERATOR_LABEL => Some(Self::DocumentationGenerator),
            PACKAGE_MANAGER_LABEL => Some(Self::PackageManager),
            _ => None,
        };
    }
}

impl core::fmt::Display for Family
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    const EVERY_FAMILY: [Family; 12] = [
        Family::Parser,
        Family::SemanticModel,
        Family::Compiler,
        Family::Linter,
        Family::TypeChecker,
        Family::Formatter,
        Family::LanguageServer,
        Family::RefactoringEngine,
        Family::Indexer,
        Family::TestRunner,
        Family::DocumentationGenerator,
        Family::PackageManager,
    ];

    #[test]
    fn Test_Every_Family_Should_Round_Trip_Through_Its_Label()
    {
        for family in EVERY_FAMILY
        {
            assert_eq!(Family::Of_Label(family.Label()), Some(family));
        }
    }

    #[test]
    fn Test_A_Label_None_Of_The_Twelve_Name_Should_Resolve_To_Nothing()
    {
        assert_eq!(Family::Of_Label("REVIEWER"), None);
        assert_eq!(Family::Of_Label("linter"), None);
        assert_eq!(Family::Of_Label(""), None);
    }

    #[test]
    fn Test_Display_Should_Render_The_Label()
    {
        assert_eq!(Family::PackageManager.to_string(), "PACKAGE_MANAGER");
    }
}
