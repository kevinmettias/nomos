use nomos_contracts::{CapabilityId, ContractVersion, Guarantee, ProviderId};

/// What a capability promises to answer, and the strongest anyone may claim about it.
///
/// The ceiling exists because a provider declaring its own guarantee is a provider
/// grading its own work. A syntactic parser that claims semantic resolution makes every
/// rule that requires resolution silently accept an answer that cannot support it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityContract
{
    pub id: CapabilityId,
    pub version: ContractVersion,
    pub summary: String,
    pub ceiling: Guarantee,
}

/// A provider's claim that it can satisfy a capability, and how well.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderOffer
{
    pub provider: ProviderId,
    pub capability: CapabilityId,
    pub version: ContractVersion,
    pub guarantee: Guarantee,
}

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
