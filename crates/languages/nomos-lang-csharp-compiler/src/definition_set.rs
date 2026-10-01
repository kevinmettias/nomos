//! [`DefinitionSet`], the preprocessor symbols one build defines.

use nomos_cap_csharp_semantics::BuildSelection;
use std::collections::BTreeSet;

/// The preprocessor symbols one build hands its compiler, before any file's own `#define` or
/// `#undef`, and the build they are for.
///
/// [`crate::Evaluate_Build`] is where one comes from in a real run: `MSBuild`'s own answer for the
/// project. [`DefinitionSet::New`] exists for a caller that already holds one -- a test,
/// a determinism harness -- and a fact produced from a set stated that way still claims what
/// [`crate::Declared_Guarantee`] claims, so the caller stating it owns that the set is real.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefinitionSet
{
    /// The build the symbols are for.
    pub selection: BuildSelection,
    /// The symbols, sorted and without duplicates.
    pub symbols: BTreeSet<String>,
}

impl DefinitionSet
{
    /// A definition set for `selection` holding `symbols`.
    #[must_use]
    pub fn New(selection: BuildSelection, symbols: BTreeSet<String>) -> Self
    {
        return DefinitionSet { selection, symbols };
    }
}
