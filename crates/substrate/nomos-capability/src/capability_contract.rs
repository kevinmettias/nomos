use nomos_contracts::{CapabilityId, ContractVersion, Guarantee};

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
