//! What an invalidation reached, and where it had to widen.

use nomos_contracts::GenerationId;
use crate::Broadening;
use crate::FactKey;
use crate::GenerationCause;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report
{
    pub cause: GenerationCause,
    pub from: GenerationId,
    pub direct: Vec<FactKey>,
    pub dependent: Vec<FactKey>,
    pub broadened: Vec<Broadening>,
    pub retained: u32,
}

impl Report
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

    fn Key_For(subject_seed: u8) -> FactKey
    {
        return FactKey {
            contract: CapabilityId::New("nomos.cap.test.report"),
            contract_version: ContractVersion::New(1, 0),
            subject: SubjectId::From_Digest(Seeded(subject_seed)),
            semantic_inputs: InputDigest::Of(&[b"fn main() {}"]),
            provider: ProviderId::New("nomos.provider.test"),
            provider_version: ContractVersion::New(1, 0),
            guarantee: GuaranteeDigest::Of(&Guarantee::New(
                FactVariant::Syntactic,
                Assurance::Sound,
                Assurance::Sound,
                IncrementalGranularity::File,
            )),
            variant: BuildVariantId::From_Digest(Seeded(9)),
            configuration: ConfigurationId::From_Digest(Seeded(8)),
        };
    }

    #[test]
    fn Test_Invalidated_Should_Add_Direct_And_Dependent_Counts()
    {
        let mut report = Empty_Report();
        report.direct = vec![Key_For(1), Key_For(2)];
        report.dependent = vec![Key_For(3)];

        assert_eq!(report.Invalidated(), 3);
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

    fn Empty_Report() -> Report
    {
        return Report {
            cause: GenerationCause::VariantChanged { variant: BuildVariantId::From_Digest(Seeded(1)) },
            from: GenerationId::From_Raw(2),
            direct: Vec::new(),
            dependent: Vec::new(),
            broadened: Vec::new(),
            retained: 3,
        };
    }
}
