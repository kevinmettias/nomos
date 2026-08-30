//! What this provider offers, and at what guarantee.

use super::{ContractVersion, CapabilityId, SchemaId, Guarantee, FactVariant, Assurance, IncrementalGranularity, CapabilityContract, ProviderOffer, ProviderId};

/// The capability this module answers.
///
/// Named for what a caller gets rather than for how it is obtained, by the same rule that
/// named `nomos.cap.syntax.items`: a second provider — one that read a module from a
/// compiler's own item table instead of from syntax facts — must be able to offer this
/// honestly.
pub const CAPABILITY: &str = "nomos.cap.module.index";

/// This implementation. The method is in the name because the method is a fact about the
/// answer: this one is a rollup over stored facts and is exactly as good as they were.
pub const PROVIDER: &str = "nomos.lang.rust.rollup";

/// The payload schema every answer to this capability is stamped with.
///
/// Versioned separately from the contract because the shape of the bytes and the meaning
/// of the question change for different reasons.
pub const SCHEMA: &str = "nomos.module.index.v1";

/// The contract version. Not a crate version: a caller reads against the contract.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

#[must_use]
pub fn Capability() -> CapabilityId
{
    return CapabilityId::New(CAPABILITY);
}

#[must_use]
pub fn Payload_Schema() -> SchemaId
{
    return SchemaId::New(SCHEMA);
}

/// The strongest anything may claim for this capability.
///
/// Deliberately above what the provider below achieves. A rollup that reparsed only the
/// member that changed would be file-granular and complete, and a ceiling set to today's
/// implementation would have to be raised to admit it — a ceiling that moves is not a
/// ceiling.
///
/// [`FactVariant::Syntactic`] is the ceiling because an index of what files declare is a
/// statement about what they say on their face. A provider that resolved the names it
/// indexed would be answering a different question.
#[must_use]
pub const fn Ceiling() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::File,
    );
}

/// The contract, to be declared once by whichever composition root builds a registry.
#[must_use]
pub fn Capability_Contract() -> CapabilityContract
{
    return CapabilityContract {
        id: Capability(),
        version: CONTRACT_VERSION,
        summary: "The items a module declares, each attributed to the member file that \
                  declared it, together with which members could be read and which were \
                  answered only approximately."
            .to_owned(),
        ceiling: Ceiling(),
    };
}

/// What this provider claims, on every axis.
///
/// Every axis is at most what its inputs were, which is what
/// [`EvidenceClass::Derived`] says about provenance stated as a guarantee.
///
/// [`Assurance::Sound`] carries over: an entry is here because a member's fact contained
/// it, and there is no step by which this could report an item no member declared.
/// Completeness stays [`Assurance::Unknown`] and cannot be anything else — the syntax facts
/// this reads may have missed macro-generated items, so this has missed them too, and a
/// rollup that is complete over incomplete inputs would be claiming to have seen what its
/// own sources could not.
///
/// [`IncrementalGranularity::Project`] is the module doc's reason: there is no partial
/// refresh to offer.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::Project,
    );
}

/// This provider's offer against [`Capability_Contract`].
#[must_use]
pub fn Provider_Offer() -> ProviderOffer
{
    return ProviderOffer {
        provider: ProviderId::New(PROVIDER),
        capability: Capability(),
        version: CONTRACT_VERSION,
        guarantee: Declared_Guarantee(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_capability::Registry;

    #[test]
    fn Test_Capability_Should_Name_The_Module_Index_Capability()
    {
        assert_eq!(Capability(), CapabilityId::New(CAPABILITY));
    }

    #[test]
    fn Test_Payload_Schema_Should_Name_The_Module_Index_Schema()
    {
        assert_eq!(Payload_Schema(), SchemaId::New(SCHEMA));
    }

    /// The strongest anything may claim for this capability, restated as a value so a
    /// silent weakening of any one axis fails here beside the constants rather than only
    /// wherever a caller happens to compare against it.
    #[test]
    fn Test_Ceiling_Should_State_The_Capabilitys_Strongest_Claimable_Guarantee()
    {
        assert_eq!(
            Ceiling(),
            Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Sound, IncrementalGranularity::File)
        );
    }

    #[test]
    fn Test_Capability_Contract_Should_Carry_The_Capability_Version_And_Ceiling()
    {
        let contract = Capability_Contract();

        assert_eq!(contract.id, Capability());
        assert_eq!(contract.version, CONTRACT_VERSION);
        assert_eq!(contract.ceiling, Ceiling());
    }

    /// What this rollup claims, weaker than the ceiling on completeness alone — the one
    /// axis a rollup over possibly-incomplete leaf facts cannot honestly claim `Sound` for.
    #[test]
    fn Test_Declared_Guarantee_Should_Be_Weaker_Than_The_Ceiling_On_Completeness_Only()
    {
        let ceiling = Ceiling();
        let declared = Declared_Guarantee();

        assert_eq!(declared.variant, ceiling.variant);
        assert_eq!(declared.soundness, ceiling.soundness);
        assert_ne!(declared.completeness, ceiling.completeness);
        assert_eq!(declared.incremental, IncrementalGranularity::Project);
    }

    #[test]
    fn Test_Provider_Offer_Should_Be_Accepted_Under_This_Capabilitys_Own_Contract()
    {
        let mut registry = Registry::New();
        registry
            .Declare(Capability_Contract())
            .expect("the contract is the first declaration in a fresh registry");

        assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
    }
}
