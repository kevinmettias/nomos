//! What an invalidation reached, and where it had to widen.
//!
//! Declared at the crate root rather than in `invalidation/`, and `lib.rs` says why where it
//! declares it: the name it is published under already carries the invalidation, so a file
//! named for it cannot also sit inside the folder that name would otherwise group it with.

use nomos_contracts::GenerationId;
use crate::Broadening;
use crate::FactKey;
use crate::GenerationCause;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvalidationReport
{
    pub cause: GenerationCause,
    pub from: GenerationId,
    pub direct: Vec<FactKey>,
    pub dependent: Vec<FactKey>,
    pub broadened: Vec<Broadening>,
    pub retained: u32,
}

impl InvalidationReport
{
    #[must_use]
    pub fn Invalidated(&self) -> usize
    {
        return self.direct.len().saturating_add(self.dependent.len());
    }

    #[must_use]
    pub fn Report(&self) -> String
    {
        return format!(
            "{} invalidated {} directly and {} through dependency edges at {}, retaining {}",
            self.cause.Describe(),
            self.direct.len(),
            self.dependent.len(),
            self.from,
            self.retained
        );
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{GuaranteeDigest, InputDigest};
    use nomos_contracts::{
        Assurance, BuildVariantId, CapabilityId, ConfigurationId, ContractVersion, Digest128,
        FactVariant, Guarantee, IncrementalGranularity, ProviderId, SubjectId,
    };

    /// The variant component every key in this module shares.
    const VARIANT_SEED: u8 = 9;

    /// The configuration component every key in this module shares; seeded apart from
    /// [`VARIANT_SEED`] so a key built with the wrong component asserts unequal.
    const CONFIGURATION_SEED: u8 = 8;

    /// The subject of the second key on the direct half. The three keys get three distinct
    /// seeds so that a fixture which lost one of them cannot still add up to
    /// [`INVALIDATED_COUNT`].
    const SECOND_DIRECT_SUBJECT_SEED: u8 = 2;

    /// The subject of the key that stands for the dependent half of a reach.
    const DEPENDENT_SUBJECT_SEED: u8 = 3;

    /// One per key the report below holds: two direct and one dependent.
    const INVALIDATED_COUNT: usize = 3;

    /// The generation the empty report describes itself as widening from.
    const REPORT_GENERATION: u64 = 2;

    /// How many facts the empty report claims to have kept.
    const RETAINED_COUNT: u32 = 3;

    fn Key_For(subject_seed: u8) -> FactKey
    {
        let guarantee = Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::File,
        );

        return FactKey {
            contract: CapabilityId::New("nomos.cap.test.report"),
            contract_version: ContractVersion::New(1, 0),
            subject: SubjectId::From_Digest(Seeded(subject_seed)),
            semantic_inputs: InputDigest::Of(&[b"fn main() {}"]),
            provider: ProviderId::New("nomos.provider.test"),
            provider_version: ContractVersion::New(1, 0),
            guarantee: GuaranteeDigest::Of(&guarantee),
            variant: BuildVariantId::From_Digest(Seeded(VARIANT_SEED)),
            configuration: ConfigurationId::From_Digest(Seeded(CONFIGURATION_SEED)),
        };
    }

    #[test]
    fn Test_Invalidated_Should_Add_Direct_And_Dependent_Counts()
    {
        let mut report = Empty_Report();
        report.direct = vec![Key_For(1), Key_For(SECOND_DIRECT_SUBJECT_SEED)];
        report.dependent = vec![Key_For(DEPENDENT_SUBJECT_SEED)];

        assert_eq!(report.Invalidated(), INVALIDATED_COUNT);
    }

    #[test]
    fn Test_Report_Should_Mention_The_Cause_And_The_Retained_Count()
    {
        let text = Empty_Report().Report();

        assert!(text.contains("build variant"), "{text}");
        assert!(text.contains("retaining 3"), "{text}");
    }

    fn Seeded(seed: u8) -> Digest128
    {
        return Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);
    }

    fn Empty_Report() -> InvalidationReport
    {
        return InvalidationReport {
            cause: GenerationCause::VariantChanged { variant: BuildVariantId::From_Digest(Seeded(1)) },
            from: GenerationId::From_Raw(REPORT_GENERATION),
            direct: Vec::new(),
            dependent: Vec::new(),
            broadened: Vec::new(),
            retained: RETAINED_COUNT,
        };
    }
}
