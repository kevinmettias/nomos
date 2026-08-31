use crate::FactKey;
use nomos_contracts::{
    BuildVariantId, ConfigurationId, IncrementalGranularity, ProviderId,
    SnapshotId, SubjectId,
};
use std::collections::BTreeSet;

// The condensation walk (`RematerializationGroup` and the Tarjan condensation it is built
// from) is its own public type and its own responsibility, so it keeps its own file rather
// than sharing this one with `GenerationCause`.
#[path = "store/rematerialization_group.rs"] mod condensation;

pub use condensation::{Condensation_Of, RematerializationGroup};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenerationCause
{
    SubjectChanged
    {
        subject: SubjectId,
        granularity: IncrementalGranularity,
    },
    ConfigurationChanged
    {
        configuration: ConfigurationId,
    },
    ProviderChanged
    {
        provider: ProviderId,
    },
    /// The workspace was replaced wholesale — a checkout, a reopened store, a tree that
    /// moved under a running process.
    ///
    /// # Why it names the members that differ
    ///
    /// It used to name only the new snapshot and invalidate every fact whose key carried
    /// the old one. That worked because a fact key carried a snapshot, and it stopped
    /// working for a good reason: a key that names a workspace state changes for every fact
    /// in the corpus when one file is edited. The component is gone, so there is nothing on
    /// a key left to match a snapshot against.
    ///
    /// What replaces it is what the caller doing the replacing actually has. Something
    /// swapped one workspace state for another, and both are content-addressed maps of
    /// path to digest — the difference between them is a set of paths, computable without
    /// consulting the store at all. Invalidating by that set is also *narrower* than the
    /// old behaviour: a checkout that touched four files no longer discards a corpus.
    ///
    /// An empty `differing` set is not refused. Two snapshots that hold identical members
    /// and differ in variant or configuration are a real thing, and those have causes of
    /// their own. [`GenerationCause::Describe`] says how many members differed, so a
    /// replacement that invalidated nothing reads as a replacement that invalidated
    /// nothing rather than as a clean result.
    SnapshotReplaced
    {
        from: SnapshotId,
        to: SnapshotId,
        /// The subjects whose content is not the same in both states.
        differing: BTreeSet<SubjectId>,
    },
    VariantChanged
    {
        variant: BuildVariantId,
    },
}

impl GenerationCause
{
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::SubjectChanged {
                subject,
                granularity,
            } => format!("{subject} changed at {granularity:?} granularity"),
            Self::ConfigurationChanged { configuration } => {
                format!("configuration {configuration} was resolved differently")
            }
            Self::ProviderChanged { provider } => format!("provider {provider} changed"),
            Self::SnapshotReplaced {
                from,
                to,
                differing,
            } => format!(
                "snapshot {from} was replaced by {to}, in which {} member(s) differ",
                differing.len()
            ),
            Self::VariantChanged { variant } => format!("build variant {variant} changed"),
        };
    }

    #[must_use]
    pub const fn Granularity(&self) -> IncrementalGranularity
    {
        return match self
        {
            Self::SubjectChanged { granularity, .. } => *granularity,
            // A replacement that names its differing members is a statement about files,
            // the same as an edit is. It was `WholeWorkspace` while the cause could only
            // say "the snapshot is different" — and a cause reported coarser than what
            // happened makes every provider's broadening record read as unavoidable.
            Self::SnapshotReplaced { .. } => IncrementalGranularity::File,
            Self::ConfigurationChanged { .. }
            | Self::ProviderChanged { .. }
            | Self::VariantChanged { .. } => IncrementalGranularity::WholeWorkspace,
        };
    }

    /// Whether this cause reaches the fact `key` names.
    ///
    /// Crate-visible rather than public: the store asks it while spreading an
    /// invalidation, and a caller outside that walk asking it would be deciding for
    /// itself what a change reaches.
    pub(crate) fn Is_Naming(&self, key: &FactKey) -> bool
    {
        return match self
        {
            Self::SubjectChanged { subject, .. } => key.subject == *subject,
            Self::ConfigurationChanged { configuration } => key.configuration == *configuration,
            Self::ProviderChanged { provider } => key.provider == *provider,
            Self::SnapshotReplaced { differing, .. } => differing.contains(&key.subject),
            Self::VariantChanged { variant } => key.variant == *variant,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::GuaranteeDigest;
    use crate::InputDigest;
    use nomos_contracts::{Assurance, CapabilityId, ContractVersion, Digest128, FactVariant, Guarantee};

    #[test]
    fn Test_Describe_Should_Name_What_Changed_For_Every_Variant()
    {
        assert!(
            GenerationCause::ConfigurationChanged { configuration: ConfigurationId::From_Digest(Seeded(1)) }
                .Describe()
                .contains("configuration")
        );
        assert!(
            GenerationCause::ProviderChanged { provider: ProviderId::New("nomos.test.provider") }
                .Describe()
                .contains("provider")
        );
        assert!(
            GenerationCause::VariantChanged { variant: BuildVariantId::From_Digest(Seeded(2)) }
                .Describe()
                .contains("build variant")
        );
    }

    #[test]
    fn Test_Granularity_Should_Read_A_Subject_Change_Own_Stated_Granularity()
    {
        let cause = GenerationCause::SubjectChanged {
            subject: SubjectId::From_Digest(Seeded(1)),
            granularity: IncrementalGranularity::Symbol,
        };

        assert_eq!(cause.Granularity(), IncrementalGranularity::Symbol);
        assert_eq!(
            GenerationCause::ProviderChanged { provider: ProviderId::New("nomos.test.provider") }.Granularity(),
            IncrementalGranularity::WholeWorkspace
        );
    }

    #[test]
    fn Test_Is_Naming_Should_Match_Only_The_Subject_A_Change_Names()
    {
        let subject = SubjectId::From_Digest(Seeded(1));
        let other_subject = SubjectId::From_Digest(Seeded(2));
        let cause = GenerationCause::SubjectChanged { subject, granularity: IncrementalGranularity::File };

        let key = Key_With(
            subject,
            ConfigurationId::From_Digest(Seeded(3)),
            ProviderId::New("nomos.test.provider"),
            BuildVariantId::From_Digest(Seeded(4)),
        );
        let other_key = Key_With(
            other_subject,
            ConfigurationId::From_Digest(Seeded(3)),
            ProviderId::New("nomos.test.provider"),
            BuildVariantId::From_Digest(Seeded(4)),
        );

        assert!(cause.Is_Naming(&key));
        assert!(!cause.Is_Naming(&other_key));
    }

    fn Key_With(
        subject: SubjectId,
        configuration: ConfigurationId,
        provider: ProviderId,
        variant: BuildVariantId,
    ) -> FactKey
    {
        return FactKey {
            contract: CapabilityId::New("nomos.cap.test.generation_cause"),
            contract_version: ContractVersion::New(1, 0),
            subject,
            semantic_inputs: InputDigest::Of(&[b"fn main() {}"]),
            provider,
            provider_version: ContractVersion::New(1, 0),
            guarantee: GuaranteeDigest::Of(&Guarantee::New(
                FactVariant::Syntactic,
                Assurance::Sound,
                Assurance::Sound,
                IncrementalGranularity::File,
            )),
            variant,
            configuration,
        };
    }

    fn Seeded(seed: u8) -> Digest128
    {
        return Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);
    }
}
