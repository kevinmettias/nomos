//! [`UndeclaredValue`], one value a rule read that the repository never declared.

use super::UndeclaredOutcome;

/// One value a rule read from a family a repository may leave undeclared, that the repository did
/// not declare: the family, where it would be declared, the key, the language the rule read it for,
/// and what the rule did with it.
///
/// `OD-RULES-011` version 3 decision 1, at the grain a repository declares a value. A run names
/// these beside its claim and never in a finding, so nothing here moves a finding, its identity or
/// the claim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UndeclaredValue
{
    /// The family: `limits`, `naming`, `scripting`, `goals`, `standards-corpus` or
    /// `requirement-trace`.
    pub family: &'static str,
    /// Where a repository declares it: the file, or for the requirement trace the directory.
    pub declared_in: &'static str,
    /// The key it is declared under.
    pub key: &'static str,
    /// The language the rule read it for, or `None` for a value read repository-wide.
    pub language: Option<String>,
    /// What the rule did with it.
    pub outcome: UndeclaredOutcome,
}
