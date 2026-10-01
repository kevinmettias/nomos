//! What a caller needs, and how badly.

use nomos_contracts::ProviderId;
use nomos_contracts::Guarantee;
use nomos_contracts::ContractVersion;
use nomos_contracts::CapabilityId;
/// What a caller needs, stated as a floor rather than a preference.
///
/// `minimum` is what the caller cannot do without; an offer below it is unusable, not
/// merely weaker. `preferred` is a name, not a second guarantee — it is what makes
/// `SupportedWithFallback` a fact about which provider answered rather than a judgement
/// about how good the answer was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Requirement
{
    pub capability: CapabilityId,
    pub version: ContractVersion,
    pub minimum: Guarantee,
    pub preferred: Option<ProviderId>,
}

impl Requirement
{
    #[must_use]
    pub const fn New(capability: CapabilityId, version: ContractVersion, minimum: Guarantee)
    -> Self
    {
        return Self {
            capability,
            version,
            minimum,
            preferred: None,
        };
    }

    #[must_use]
    pub fn Preferring(mut self, provider: ProviderId) -> Self
    {
        self.preferred = Some(provider);
        return self;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{Assurance, FactVariant, IncrementalGranularity};

    #[test]
    fn Test_New_Should_Start_With_No_Preferred_Provider()
    {
        let requirement = Requirement::New(Capability(), Version(), Floor());

        assert_eq!(requirement.capability, Capability());
        assert_eq!(requirement.version, Version());
        assert_eq!(requirement.minimum, Floor());
        assert!(requirement.preferred.is_none());
    }

    #[test]
    fn Test_Preferring_Should_Record_The_Named_Provider()
    {
        let provider = ProviderId::New("nomos.test.preferred");

        let requirement = Requirement::New(Capability(), Version(), Floor()).Preferring(provider.clone());

        assert_eq!(requirement.preferred, Some(provider));
    }

    fn Capability() -> CapabilityId
    {
        return CapabilityId::New("nomos.cap.test.requirement");
    }

    fn Version() -> ContractVersion
    {
        return ContractVersion::New(1, 0);
    }

    fn Floor() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::File,
        );
    }
}
