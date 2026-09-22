//! One distinct provenance tuple a rule's reads reduced to.

use nomos_analysis::{Dependency, FactStore, MaterializedFact, MemoryFactStore, ReadOutcome};
use nomos_contracts::{CapabilityId, ContractVersion, EvidenceClass, GenerationId, Guarantee, ProviderId};

/// One distinct provenance tuple behind a rule's reads: which capability was asked, which
/// provider answered it at which contract version, how the read came back, and -- for a read
/// that was answered -- the guarantee and evidence class of the fact that answered it.
///
/// Provenance, not identity. `OD-HOST-016` decided that the `nomos_analysis::FactKey`'s own
/// digest components -- its subject, its `semantic_inputs`, its `GuaranteeDigest`, its build
/// variant and its configuration -- are deliberately not carried here: they address a fact
/// and answer no question a reader can ask. Dropping the subject in particular is what makes
/// this honest at rule grain, because a finding's subject is not always the subject its rule
/// read, and joining the two on their shared type would claim a precision this grain does not
/// have.
///
/// [`Self::guarantee`] and [`Self::evidence`] are read off the `nomos_analysis::
/// MaterializedFact` that answered, inside the run, while the store is still in hand -- so
/// the answer that leaves carries them by value and no caller ever needs a key or a store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactRead
{
    /// The capability family the rule asked for.
    pub capability: CapabilityId,
    /// The provider whose offer this read was addressed to.
    pub provider: ProviderId,
    /// The contract version that provider's offer names.
    pub provider_version: ContractVersion,
    /// How the read came back. A miss is recorded as readily as a hit: a rule whose optional
    /// policy read missed was judged against a compiled-in default, and this is the only
    /// place that says so.
    pub outcome: ReadOutcome,
    /// The declared completeness of the fact that answered, or `None` for a read that did
    /// not answer.
    pub guarantee: Option<Guarantee>,
    /// How the fact that answered was come by, or `None` for a read that did not answer.
    pub evidence: Option<EvidenceClass>,
    /// How many reads stood behind this tuple. A rule reading one family over a thousand
    /// sources reduces to one tuple carrying a thousand, which is the whole reason the
    /// reduction exists.
    pub reads: usize,
}

/// `dependencies` reduced to the distinct provenance tuples they hold, each carrying the
/// count of the reads behind it.
///
/// Linear rather than keyed, because the reduced population is bounded by the number of
/// distinct offers that actually answered -- one or two for most rules -- and not by the
/// number of files read.
pub(crate) fn Reduced(dependencies: &[Dependency], store: &MemoryFactStore, at: GenerationId) -> Vec<FactRead>
{
    let mut reduced: Vec<FactRead> = Vec::new();
    for dependency in dependencies
    {
        match reduced.iter_mut().find(|read| return Is_The_Same_Tuple(read, dependency))
        {
            Some(read) => read.reads = read.reads.saturating_add(1),
            None => reduced.push(Read_Of(dependency, store, at)),
        }
    }

    return reduced;
}

/// Whether `read` is the tuple `dependency` reduces to.
fn Is_The_Same_Tuple(read: &FactRead, dependency: &Dependency) -> bool
{
    return read.capability == dependency.key.contract
        && read.provider == dependency.key.provider
        && read.provider_version == dependency.key.provider_version
        && read.outcome == dependency.outcome;
}

/// `dependency` as a fresh tuple, resolved against the store while it is still in hand.
fn Read_Of(dependency: &Dependency, store: &MemoryFactStore, at: GenerationId) -> FactRead
{
    let answering = Answering(dependency, store, at);

    return FactRead {
        capability: dependency.key.contract.clone(),
        provider: dependency.key.provider.clone(),
        provider_version: dependency.key.provider_version,
        outcome: dependency.outcome,
        guarantee: answering.as_ref().map(|fact| return fact.guarantee),
        evidence: answering.as_ref().map(|fact| return fact.evidence),
        reads: 1,
    };
}

/// The fact that answered `dependency`, or `None` when the read did not answer.
fn Answering(dependency: &Dependency, store: &MemoryFactStore, at: GenerationId) -> Option<MaterializedFact>
{
    if !dependency.outcome.Is_Answered()
    {
        return None;
    }

    return store.Current(&dependency.key.clone().At(at), at);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_analysis::{FactKey, GuaranteeDigest, InputDigest};
    use nomos_contracts::{
        Applicability, Assurance, BuildVariantId, ConfigurationId, Digest128, FactVariant,
        IncrementalGranularity, SubjectId,
    };

    /// The generation every key in this module is read at.
    const GENERATION: u64 = 1;

    /// The subject of the second key the count test records, distinct from the first so a
    /// reduction that kept subjects apart would produce two tuples rather than one.
    const SECOND_SUBJECT_SEED: u8 = 2;

    /// How many reads the count test performs over one tuple.
    const REDUCED_READS: usize = 2;

    #[test]
    fn Test_Reduced_Should_Fold_Two_Subjects_Of_One_Offer_Into_One_Tuple()
    {
        let store = MemoryFactStore::New();
        let dependencies = [Miss_On(1), Miss_On(SECOND_SUBJECT_SEED)];

        let reduced = Reduced(&dependencies, &store, GenerationId::From_Raw(GENERATION));

        assert_eq!(reduced.len(), 1, "{reduced:?}");
        assert_eq!(reduced.first().expect("the assertion above found one tuple").reads, REDUCED_READS);
    }

    #[test]
    fn Test_Reduced_Should_Keep_Two_Outcomes_Of_One_Offer_Apart()
    {
        let store = MemoryFactStore::New();
        let dependencies = [Miss_On(1), Dependency { key: Key_For(1), outcome: ReadOutcome::Absent }];

        let reduced = Reduced(&dependencies, &store, GenerationId::From_Raw(GENERATION));

        assert_eq!(reduced.len(), REDUCED_READS, "{reduced:?}");
    }

    #[test]
    fn Test_Reduced_Should_Carry_No_Guarantee_For_A_Read_That_Did_Not_Answer()
    {
        let store = MemoryFactStore::New();

        let reduced = Reduced(&[Miss_On(1)], &store, GenerationId::From_Raw(GENERATION));

        let read = reduced.first().expect("one dependency reduces to one tuple");
        assert_eq!(read.guarantee, None);
        assert_eq!(read.evidence, None);
    }

    fn Miss_On(subject_seed: u8) -> Dependency
    {
        return Dependency {
            key: Key_For(subject_seed),
            outcome: ReadOutcome::Degraded(Applicability::DependencyUnavailable),
        };
    }

    fn Key_For(subject_seed: u8) -> FactKey
    {
        let guarantee = Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Sound, IncrementalGranularity::File);

        return FactKey {
            contract: CapabilityId::New("nomos.cap.test.reduction"),
            contract_version: ContractVersion::New(1, 0),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([subject_seed; Digest128::BYTE_LENGTH])),
            semantic_inputs: InputDigest::Of(&[b"fn main() {}"]),
            provider: ProviderId::New("nomos.provider.test.reduction"),
            provider_version: ContractVersion::New(1, 0),
            guarantee: GuaranteeDigest::Of(&guarantee),
            variant: BuildVariantId::From_Digest(Digest128::From_Bytes([9; Digest128::BYTE_LENGTH])),
            configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([8; Digest128::BYTE_LENGTH])),
        };
    }
}
