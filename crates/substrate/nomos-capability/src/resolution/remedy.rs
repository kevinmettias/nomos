//! What closes the gap a matching [`Unmet`](crate::Unmet) variant names.

use nomos_contracts::ContractVersion;
use nomos_contracts::ProviderId;

/// What closes the gap a matching [`Unmet`](crate::Unmet) variant names -- the only half of
/// the pair a caller can act on, structured so it survives being rendered more than one way
/// rather than fixed as one sentence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Remedy
{
    /// Author a capability contract naming this capability, then declare it against a
    /// [`crate::Registry`].
    DeclareTheContract,
    /// The contract is declared; register a provider offer against it.
    RegisterAProvider,
    /// Offered at a version this caller cannot read; either side moving to a version the
    /// other can read closes the gap.
    AgreeOnAVersion
    {
        offered: ContractVersion,
    },
    /// The closest offer does not reach the required guarantee; strengthen it, or register
    /// a new offer that does.
    StrengthenTheClosestOffer
    {
        closest: ProviderId,
    },
}
