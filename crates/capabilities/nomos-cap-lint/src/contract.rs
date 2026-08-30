//! The agreement itself.

use nomos_capability::CapabilityContract;
use nomos_contracts::{
    Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity,
    SchemaId,
};

/// The capability this crate answers.
///
/// Named for what a caller gets — one workspace member's own lint diagnostics — rather
/// than for the tool that produced them, the same reason `nomos.cap.dependency.edges` is
/// named for its answer rather than for `cargo metadata`: a second lint tool offering the
/// same shape of answer must be able to name this capability honestly.
pub const CAPABILITY: &str = "nomos.cap.lint.diagnostics";

/// The payload schema every answer to this capability is stamped with.
///
/// Versioned separately from the contract for the same reason `nomos-cap-syntax` and
/// `nomos-cap-dependency` both give: the shape of the bytes and the meaning of the
/// question change for different reasons.
pub const SCHEMA: &str = "nomos.lint.diagnostics.v1";

/// The contract version. Not a crate version: a caller reads against the contract.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// The strongest anything may claim for this capability.
///
/// [`FactVariant::SemanticallyResolved`] because a real lint tool's diagnostics are
/// established over resolved names and types, not text alone — the same tier
/// `nomos.cap.dependency.edges` claims for the same reason: a tool that only scanned
/// source text for suspicious patterns belongs at `Syntactic`, not here.
///
/// Soundness [`Assurance::Sound`] at the ceiling: every diagnostic a provider reports is
/// a diagnostic the underlying tool actually emitted, never one this capability infers.
/// Completeness [`Assurance::Sound`] as well, deliberately not pinned to what today's one
/// provider achieves — the same "leave room for a stronger future provider" reasoning
/// `nomos-cap-syntax`'s own ceiling gives, though `OD-RULES-010`'s own text already notes
/// no real tool honestly claims that today.
///
/// [`IncrementalGranularity::Project`]: this capability's one real provider materializes
/// one fact per workspace member, bundling every diagnostic that member's own files
/// produced, the same granularity `nomos.cap.dependency.edges` already chose for a
/// per-package fact.
#[must_use]
pub const fn Ceiling() -> Guarantee
{
    return Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::Project,
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
        summary: "One workspace member's own diagnostics from an external lint tool, as \
                  that tool itself reported them -- level, message, lint identity where \
                  the tool names one, and the file and line each diagnostic's primary span \
                  names."
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
    /// asserting it watched the build actually execute, which reading a lint tool's own
    /// static diagnostics does not do.
    #[test]
    fn Test_The_Ceiling_Should_Refuse_A_Claim_Of_Runtime_Observation()
    {
        contract_testing::Assert_Ceiling_Refuses(
            Capability_Contract(),
            Guarantee::New(
                FactVariant::RuntimeObserved,
                Assurance::Sound,
                Assurance::Sound,
                IncrementalGranularity::Project,
            ),
            "a provider claiming runtime observation for a capability about a lint tool's \
             own static diagnostics would satisfy every rule that needs it",
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
}
