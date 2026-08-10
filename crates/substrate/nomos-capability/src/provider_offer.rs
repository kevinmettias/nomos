//! What one provider says it can do for one capability.

use nomos_contracts::Guarantee;
use nomos_contracts::ContractVersion;
use nomos_contracts::CapabilityId;
use nomos_contracts::ProviderId;
/// A provider's claim that it can satisfy a capability, and how well.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderOffer
{
    pub provider: ProviderId,
    pub capability: CapabilityId,
    pub version: ContractVersion,
    pub guarantee: Guarantee,
}
