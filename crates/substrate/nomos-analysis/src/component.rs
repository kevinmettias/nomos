//! The parts a fact identity is assembled from.

/// The parts of a fact's identity.
///
/// # Why there is no `Snapshot`
///
/// There was one, and it defeated the two components either side of it.
///
/// A workspace snapshot identity is a digest over *every* member of the workspace, so a key
/// carrying one changes for every fact in the corpus whenever any one file is edited. The
/// vertical slice measured it: over a six-file corpus, editing one file recomputed eight
/// facts where two had changed.
///
/// It was also redundant twice over. [`Component::SemanticInputs`] already says what a fact
/// was computed from — for a leaf, the file's content, exactly and no more coarsely — and
/// the store's generation interval already says which analysis state a fact is current at.
/// The snapshot added every other file in the workspace to the first and, being the
/// coarsest of the three, overrode the second.
///
/// A fact's relation to a workspace state is now provenance on
/// [`crate::MaterializedFact::snapshot`]: what it was measured against, not part of what it
/// is. `docs/records/OD-ANALYSIS-001` carries the finding and what closed it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Component
{
    Contract,
    ContractVersion,
    Subject,
    SemanticInputs,
    Provider,
    ProviderVersion,
    Guarantee,
    Variant,
    Configuration,
}

impl Component
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Contract => "contract",
            Self::ContractVersion => "contract_version",
            Self::Subject => "subject",
            Self::SemanticInputs => "semantic_inputs",
            Self::Provider => "provider",
            Self::ProviderVersion => "provider_version",
            Self::Guarantee => "guarantee",
            Self::Variant => "variant",
            Self::Configuration => "configuration",
        };
    }

    /// Every component of a fact's key.
    ///
    /// Mirrored by `Test_Every_Component_Should_Be_Matched_Exhaustively`, an exhaustive
    /// match over every variant with no wildcard arm, in
    /// `crates/substrate/nomos-analysis/tests/fact_identity/key_identity.rs`. It fails to
    /// compile, not merely to pass, if a variant is added here without being added there.
    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[
            Self::Contract,
            Self::ContractVersion,
            Self::Subject,
            Self::SemanticInputs,
            Self::Provider,
            Self::ProviderVersion,
            Self::Guarantee,
            Self::Variant,
            Self::Configuration,
        ];
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// One per variant `Component::All` lists. `Label`'s exhaustive match over every
    /// variant is what keeps this list and the enum in step, so this pins the count.
    const COMPONENT_COUNT: usize = 9;

    #[test]
    fn Test_Label_Should_Give_Each_Variant_A_Distinct_Lowercase_Name()
    {
        let mut labels: Vec<&str> = Component::All().iter().map(|component| component.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two components share a label");
        assert_eq!(Component::Contract.Label(), "contract");
    }

    #[test]
    fn Test_All_Should_List_Every_Variant_Exactly_Once()
    {
        let all = Component::All();

        assert_eq!(all.len(), COMPONENT_COUNT);
        assert!(all.contains(&Component::Configuration));
    }
}
