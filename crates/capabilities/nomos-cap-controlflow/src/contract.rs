//! The agreement itself.

use nomos_capability::CapabilityContract;
use nomos_contracts::{
    Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity,
    SchemaId,
};

/// The capability this crate answers.
///
/// Named for what a caller gets — whether a control-flow path forward from a fact-read
/// failure reaches a `Finding` — rather than for how it is obtained, the same reason
/// `nomos.cap.dependency.edges` is named for its answer rather than for Cargo.
pub const CAPABILITY: &str = "nomos.cap.controlflow.reachability";

/// The payload schema every answer to this capability is stamped with.
pub const SCHEMA: &str = "nomos.controlflow.reachability.v1";

/// The contract version. Not a crate version: a caller reads against the contract.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// The strongest anything may claim for this capability.
///
/// [`FactVariant::SemanticallyResolved`] because `OD-RULES-008` already settled what a
/// *sound* answer to this question needs: every call an `Err` arm reaches, including into
/// helper functions elsewhere in the crate, resolved to confirm it actually constructs or
/// propagates a `Finding` rather than merely being named as though it does. Pattern-matching
/// the arm's own tail expression — what any provider offering below this ceiling does —
/// cannot see past a call site, which is exactly the gap a sound provider closes. The
/// ceiling states the capability's true tier; it is not moved down to match what the first
/// provider offering against it actually delivers, the same choice `nomos.cap.dependency.
/// edges`' own ceiling makes for the same reason.
///
/// [`IncrementalGranularity::File`]: this capability's canonical subject is a control-flow
/// edge inside one function body, and a function body lives in one file — there is no
/// coarser unit a change here could force a re-derivation across, the same granularity
/// `nomos.cap.syntax.items` already claims for the same reason.
///
/// Both `Assurance`s `Sound`, not pinned to what any current provider achieves: this
/// ceiling leaves room for a provider whose soundness and completeness are both established,
/// the same way `nomos.cap.dependency.edges`' ceiling leaves room for one that also resolves
/// generated dependency declarations.
#[must_use]
pub const fn Ceiling() -> Guarantee
{
    return Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::File,
    );
}

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

/// The contract, to be declared once by whichever composition root builds a registry.
#[must_use]
pub fn Capability_Contract() -> CapabilityContract
{
    return CapabilityContract {
        id: Capability(),
        version: CONTRACT_VERSION,
        summary: "Whether a control-flow path forward from a fact-read failure — a match \
                  arm binding an `Err` from a capability read — reaches a `Finding` \
                  construction before the enclosing function returns, the property \
                  `Applicability`'s own module doc names as this product's first \
                  principle: unknown is not pass."
            .to_owned(),
        ceiling: Ceiling(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_capability::{ProviderOffer, Registry, Requirement, Resolution, Unmet};
    use nomos_contracts::ProviderId;

    /// The contract admits an answer weaker than its ceiling, which is what a ceiling is
    /// for — a `Syntactic`, pattern-matching-only provider is exactly the first offer this
    /// capability expects.
    #[test]
    fn Test_The_Ceiling_Should_Admit_A_Weaker_Offer()
    {
        let mut registry = Registry::New();
        registry.Declare(Capability_Contract()).expect("declared once");

        let heuristic = ProviderOffer {
            provider: ProviderId::New("nomos.test.heuristic"),
            capability: Capability(),
            version: CONTRACT_VERSION,
            guarantee: Guarantee::New(
                FactVariant::Syntactic,
                Assurance::Sound,
                Assurance::Unsound,
                IncrementalGranularity::File,
            ),
        };

        assert_eq!(registry.Offer(heuristic), Ok(()));
    }

    /// And refuses one above it — a claim of `RuntimeObserved` would be a provider
    /// asserting it watched every path actually execute, which no static reader does.
    #[test]
    fn Test_The_Ceiling_Should_Refuse_A_Claim_Above_Semantically_Resolved()
    {
        let mut registry = Registry::New();
        registry.Declare(Capability_Contract()).expect("declared once");

        let observed = ProviderOffer {
            provider: ProviderId::New("nomos.test.optimistic"),
            capability: Capability(),
            version: CONTRACT_VERSION,
            guarantee: Guarantee::New(
                FactVariant::RuntimeObserved,
                Assurance::Sound,
                Assurance::Sound,
                IncrementalGranularity::File,
            ),
        };

        assert!(
            registry.Offer(observed).is_err(),
            "a provider claiming runtime observation for a capability about static \
             reachability would satisfy every rule that needs it"
        );
    }

    /// The contract stands without a provider.
    #[test]
    fn Test_The_Contract_Should_Stand_With_No_Provider_At_All()
    {
        let mut registry = Registry::New();
        registry.Declare(Capability_Contract()).expect("declared once");

        let need = Requirement::New(Capability(), CONTRACT_VERSION, Ceiling());
        let resolution = registry.Resolve(&need);

        assert!(
            matches!(
                resolution,
                Resolution::Unsatisfied {
                    reason: Unmet::NoProvider,
                    ..
                }
            ),
            "the contract is declared and unoffered, which is coverage debt rather than an \
             undeclared capability: {resolution:?}"
        );
    }

    /// One declaration per capability.
    #[test]
    fn Test_One_Capability_Should_Admit_One_Contract()
    {
        let mut registry = Registry::New();
        registry.Declare(Capability_Contract()).expect("declared once");

        assert!(
            registry.Declare(Capability_Contract()).is_err(),
            "one name, one meaning"
        );
    }
}
