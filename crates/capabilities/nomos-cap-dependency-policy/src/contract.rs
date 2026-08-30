//! The agreement itself.

use nomos_capability::CapabilityContract;
use nomos_contracts::{
    Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity,
    SchemaId,
};

/// The capability this crate answers.
///
/// Named for what a caller gets — the workspace's own bans/licenses/sources verdict —
/// rather than for the tool that produced it, the same reason `nomos.cap.lint.diagnostics`
/// is named for its answer rather than for `cargo clippy`: a second policy tool offering
/// the same shape of answer must be able to name this capability honestly.
pub const CAPABILITY: &str = "nomos.cap.dependency.policy";

/// The payload schema every answer to this capability is stamped with.
///
/// Versioned separately from the contract for the same reason `nomos-cap-lint` and
/// `nomos-cap-dependency` both give: the shape of the bytes and the meaning of the
/// question change for different reasons.
pub const SCHEMA: &str = "nomos.dependency.policy.v1";

/// The contract version. Not a crate version: a caller reads against the contract.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// The strongest anything may claim for this capability.
///
/// [`FactVariant::SemanticallyResolved`] because a real policy tool's verdict is reached
/// over the fully resolved dependency graph (`Cargo.lock`, not `Cargo.toml`'s own
/// unresolved ranges) — the same tier `nomos.cap.dependency.edges` claims for the
/// identical reason: a tool that only scanned manifest text for suspicious names belongs
/// at `Syntactic`, not here.
///
/// Soundness [`Assurance::Sound`] at the ceiling: every violation a provider reports is a
/// violation the underlying tool actually flagged, never one this capability infers.
/// Completeness [`Assurance::Sound`] as well, deliberately not pinned to what today's one
/// provider achieves — the same "leave room for a stronger future provider" reasoning
/// `nomos-cap-lint`'s own ceiling gives.
///
/// [`IncrementalGranularity::WholeWorkspace`]: this capability's one real provider reasons
/// over the resolved dependency graph as a whole and materializes exactly one fact for it
/// — a duplicate-version or license violation is a property of the graph, not of whichever
/// workspace member happens to pull the offending crate in, so there is no per-member
/// split this ceiling could honestly claim instead.
#[must_use]
pub const fn Ceiling() -> Guarantee
{
    return Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::WholeWorkspace,
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
        summary: "The workspace's own bans/licenses/sources verdict from an external \
                  dependency-policy tool, as that tool itself reported it -- severity, \
                  code and message for each violation it found over the resolved \
                  dependency graph as a whole."
            .to_owned(),
        ceiling: Ceiling(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_capability::contract_testing;

    /// The contract admits an answer weaker than its ceiling, which is what a ceiling is
    /// for.
    #[test]
    fn Test_The_Ceiling_Should_Admit_A_Weaker_Offer()
    {
        contract_testing::Assert_Ceiling_Admits(
            Capability_Contract(),
            Capability(),
            CONTRACT_VERSION,
            Guarantee::New(
                FactVariant::Syntactic,
                Assurance::Unsound,
                Assurance::Unknown,
                IncrementalGranularity::WholeWorkspace,
            ),
        );
    }

    /// And refuses one above it — a claim of `RuntimeObserved` would be a provider
    /// asserting it watched the build actually execute, which reading a policy tool's own
    /// static verdict does not do.
    #[test]
    fn Test_The_Ceiling_Should_Refuse_A_Claim_Of_Runtime_Observation()
    {
        contract_testing::Assert_Ceiling_Refuses(
            Capability_Contract(),
            Guarantee::New(
                FactVariant::RuntimeObserved,
                Assurance::Sound,
                Assurance::Sound,
                IncrementalGranularity::WholeWorkspace,
            ),
            "a provider claiming runtime observation for a capability about a policy \
             tool's own static verdict would satisfy every rule that needs it",
        );
    }

    /// The contract stands without a provider.
    #[test]
    fn Test_The_Contract_Should_Stand_With_No_Provider_At_All()
    {
        contract_testing::Assert_Stands_With_No_Provider(
            Capability_Contract(),
            Capability(),
            CONTRACT_VERSION,
            Ceiling(),
        );
    }

    /// One declaration per capability.
    #[test]
    fn Test_One_Capability_Should_Admit_One_Contract()
    {
        contract_testing::Assert_One_Contract_Per_Capability(Capability_Contract(), "one name, one meaning");
    }

    /// [`Payload_Schema`] is a leaf wrapper with no in-crate caller of its own — the four
    /// tests above drive [`Capability_Contract`], [`Capability`] and [`Ceiling`] but never
    /// this one, so it needs its own direct assertion rather than borrowing theirs.
    #[test]
    fn Test_Payload_Schema_Should_Match_The_Declared_Schema_Constant()
    {
        assert_eq!(Payload_Schema(), SchemaId::New(SCHEMA));
    }

    /// The other four tests each drive [`Capability_Contract`] only to reach a property of
    /// the ceiling or the registry; none asserts on the struct it built. This checks the
    /// construction itself: every field lands the value its own declaration says it should.
    #[test]
    fn Test_Capability_Contract_Should_Assemble_Its_Declared_Identity_And_Ceiling()
    {
        let contract = Capability_Contract();

        assert_eq!(contract.id, Capability());
        assert_eq!(contract.version, CONTRACT_VERSION);
        assert_eq!(contract.ceiling, Ceiling());
    }
}
