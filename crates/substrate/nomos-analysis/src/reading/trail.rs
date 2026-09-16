//! Every fact a read depended on, in the order it was read.

use crate::Dependency;
use crate::FactKey;
use crate::ReadOutcome;

/// What one reader has read, kept apart from what it reads *through*.
///
/// A [`Reader`](crate::Reader) holds a store, a registry and a build; none of those change
/// while it reads. This is the half that does, and it is the half the invalidation engine
/// consumes afterwards. Separating them is what stops "which offers were admitted" and
/// "what did we end up asking for" from being one thing with two lifetimes.
///
/// A miss is recorded as readily as a hit. "The parser had nothing here" is a real read and
/// a real dependency, and a fact that must be invalidated once the parser *does* have
/// something needs that edge to exist.
#[derive(Default)]
pub(crate) struct Trail
{
    recorded: Vec<Dependency>,
}

impl Trail
{
    /// An empty trail, before anything has been read.
    #[must_use]
    pub(crate) const fn New() -> Self
    {
        return Self {
            recorded: Vec::new(),
        };
    }

    /// Records one read.
    pub(crate) fn Note(&mut self, key: &FactKey, outcome: ReadOutcome)
    {
        self.recorded.push(Dependency {
            key: key.clone(),
            outcome,
        });
    }

    /// Records a miss at the applicability every unanswered candidate is recorded at.
    pub(crate) fn Note_Miss(&mut self, key: &FactKey)
    {
        use nomos_contracts::Applicability;

        self.Note(
            key,
            ReadOutcome::Degraded(Applicability::DependencyUnavailable),
        );
    }

    /// What has been read so far.
    #[must_use]
    pub(crate) fn Recorded(&self) -> &[Dependency]
    {
        return &self.recorded;
    }

    /// The trail, once the reader that produced it is finished with.
    #[must_use]
    pub(crate) fn Into_Dependencies(self) -> Vec<Dependency>
    {
        return self.recorded;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{GuaranteeDigest, InputDigest};
    use nomos_contracts::{
        Applicability, Assurance, BuildVariantId, CapabilityId, ConfigurationId, ContractVersion,
        Digest128, FactVariant, Guarantee, IncrementalGranularity, ProviderId, SubjectId,
    };

    /// The subject of the second key a trail records. The first key's subject is `1`, which
    /// is identity rather than a choice; the two must differ or a trail that recorded one
    /// key twice would read as a trail that recorded two.
    const SECOND_SUBJECT_SEED: u8 = 2;

    /// How many reads the order test records. The two `expect` messages at its assertions
    /// name this same number, so they move together.
    const RECORDED_COUNT: usize = 2;

    /// The variant component of every key in this module.
    const VARIANT_SEED: u8 = 9;

    /// The configuration component of every key in this module; seeded apart from
    /// [`VARIANT_SEED`] so the two components cannot be confused for one another.
    const CONFIGURATION_SEED: u8 = 8;

    #[test]
    fn Test_New_Should_Start_Empty()
    {
        assert!(Trail::New().Recorded().is_empty());
    }

    #[test]
    fn Test_Note_Should_Append_A_Dependency_At_The_Given_Outcome()
    {
        let mut trail = Trail::New();
        let key = Key_For(1);

        trail.Note(&key, ReadOutcome::Materialized);

        assert_eq!(trail.Recorded(), &[Dependency { key, outcome: ReadOutcome::Materialized }]);
    }

    #[test]
    fn Test_Note_Miss_Should_Record_A_Dependency_Unavailable_Degraded_Outcome()
    {
        let mut trail = Trail::New();
        let key = Key_For(SECOND_SUBJECT_SEED);

        trail.Note_Miss(&key);

        assert_eq!(
            trail.Recorded(),
            &[Dependency {
                key,
                outcome: ReadOutcome::Degraded(Applicability::DependencyUnavailable)
            }]
        );
    }

    #[test]
    fn Test_Recorded_Should_Return_Every_Read_In_Order()
    {
        let mut trail = Trail::New();
        trail.Note(&Key_For(1), ReadOutcome::Materialized);
        trail.Note(&Key_For(SECOND_SUBJECT_SEED), ReadOutcome::Absent);

        assert_eq!(trail.Recorded().len(), RECORDED_COUNT);
        assert_eq!(trail.Recorded().first().expect("the assertion above found 2 entries").key, Key_For(1));
        assert_eq!(trail.Recorded().get(1).expect("the assertion above found 2 entries").key, Key_For(SECOND_SUBJECT_SEED));
    }

    #[test]
    fn Test_Into_Dependencies_Should_Return_Every_Read_By_Value()
    {
        let mut trail = Trail::New();
        trail.Note(&Key_For(1), ReadOutcome::Materialized);

        let dependencies = trail.Into_Dependencies();

        assert_eq!(dependencies, vec![Dependency { key: Key_For(1), outcome: ReadOutcome::Materialized }]);
    }

    fn Seeded_Digest(seed: u8) -> Digest128
    {
        return Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);
    }

    fn Key_For(subject_seed: u8) -> FactKey
    {
        let guarantee = Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::File,
        );

        return FactKey {
            contract: CapabilityId::New("nomos.cap.test.trail"),
            contract_version: ContractVersion::New(1, 0),
            subject: SubjectId::From_Digest(Seeded_Digest(subject_seed)),
            semantic_inputs: InputDigest::Of(&[b"fn main() {}"]),
            provider: ProviderId::New("nomos.provider.test"),
            provider_version: ContractVersion::New(1, 0),
            guarantee: GuaranteeDigest::Of(&guarantee),
            variant: BuildVariantId::From_Digest(Seeded_Digest(VARIANT_SEED)),
            configuration: ConfigurationId::From_Digest(Seeded_Digest(CONFIGURATION_SEED)),
        };
    }
}
