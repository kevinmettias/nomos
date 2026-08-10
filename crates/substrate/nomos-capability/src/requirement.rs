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
