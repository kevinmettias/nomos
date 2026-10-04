//! What the repository a ledger serves declares, handed to the verbs that judge an item
//! against it.

use crate::PredicateCoverage;
use crate::Territory;

/// What the repository a ledger serves declares, as its caller read it.
///
/// Read by the caller and handed over as a value, never discovered here. The store owns the
/// *decision* — `OD-LEDGER-021` put `add` behind one lock for that reason — and enumerating a
/// repository is not a thing a general exclusion ledger should learn to do. That was already
/// the arrangement for the published records, and `OD-GATE-036` reads the coverage
/// declaration at the same place, so the two arrive together.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryDeclarations
{
    /// The record files this repository has already published, which `add` refuses an
    /// undeclared reservation of.
    pub published: Territory,
    /// What it declares an item's predicate must carry when the item reaches certain paths,
    /// which `add` and `widen` both judge.
    pub coverage: PredicateCoverage,
}

impl RepositoryDeclarations
{
    /// A repository that has published no record and declares no coverage rule.
    ///
    /// What a caller with nothing to declare passes. It says nothing is known rather than that
    /// nothing exists, and the open-item half of every check still holds.
    #[must_use]
    pub fn Undeclared() -> Self
    {
        return Self {
            published: Territory::Empty(),
            coverage: PredicateCoverage::Undeclared(),
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Undeclared_Should_Publish_Nothing_And_Declare_No_Rule()
    {
        let declared = RepositoryDeclarations::Undeclared();

        assert_eq!(declared.published, Territory::Empty());
        assert_eq!(declared.coverage, PredicateCoverage::Undeclared());
    }
}
