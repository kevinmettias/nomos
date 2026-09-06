//! The agreement itself, and why it is a new capability rather than a second offer against
//! a hypothetical `nomos.cap.connector.artifact`.
//!
//! Lives here, beside this crate's one provider, rather than in its own crate.
//! `OD-CAPABILITY-002`'s real criterion -- worked out against
//! `crates/languages/nomos-lang-rust-scan/src/guarantee.rs`'s own cautionary module doc --
//! is that a capability contract extracts once a second real *provider* names it, not
//! merely once a rule reads it. `nomos.cap.review.finding` has exactly one provider today,
//! this crate's own, so a second crate would buy independence nothing spends -- the same
//! lives-beside-its-only-provider state `nomos.cap.module.surface` is in.
//!
//! # Why this is Capability Contract zone rather than Provider zone
//!
//! Every other `ToolProvider` in this workspace (`nomos-lang-rust-clippy`,
//! `nomos-lang-rust-deny`) splits its contract into its own `nomos-cap-*` crate,
//! specifically so `nomos-rules` (Rules zone) can depend on the contract without
//! depending on the Provider-zone crate that performs the subprocess I/O --
//! `crates/rules/nomos-rules/src/checks/dependency/zones.rs`'s own `Permits` forbids
//! `Rules` from naming `Provider` at all. This crate bundles contract and provider
//! together in one crate instead (the shape `OD-CAPABILITY-002` licenses for a
//! single-provider capability), so it is classified Capability Contract zone rather than
//! Provider zone: Provider zone would make this fact family structurally unreachable by
//! any rule, not merely undesirable.
//!
//! # Why this is not a second offer against an artifact-shaped contract
//!
//! `ARC-CONNECTOR-001` names Jira, Confluence and `SharePoint` as the expected second and
//! third connectors -- every one of them an issue tracker or a document store, answering
//! one question: what is this external record's own title, state, locator and kind. A
//! `CodeRabbit` review comment is not that question answered by a different vendor; it is a
//! different question entirely:
//!
//! - `title` and `state` presuppose one external record with an evolving lifecycle status
//!   (an issue is `OPEN` or `CLOSED`). A review comment has neither. It has a severity
//!   (`Critical`, `Major`, `Minor`, ...) and a category (`Security & Privacy`,
//!   `Functional Correctness`, ...) -- axes an issue does not carry and a title/state pair
//!   cannot honestly hold without inventing a mapping neither vendor's schema states.
//! - An issue's own fields describe the record itself, independent of any other artifact.
//!   A review comment is inseparable from a file and a line in a diff -- `path` and `line`
//!   are not optional context, they are what makes the finding a finding at all.
//! - Forcing a review comment through a single `kind` word would either lose the severity
//!   and category that are this fact's entire reason for existing, or overload one field
//!   with data a shared schema was never agreed to carry -- silently changing the
//!   agreement any future artifact-shaped provider (Jira, Confluence) would also be bound
//!   by, without either of them having been asked.
//!
//! Two facts arriving through a connector, both `Observed` evidence about something
//! external, is the resemblance `OD-CAPABILITY-002` warns against taking as the criterion
//! on its own. Two providers "contend" for one capability when they answer the same
//! question well enough that a caller could honestly be indifferent to which one
//! answered it. Nothing here could stand in for an artifact contract's answer, and
//! nothing there could stand in for this one.

use nomos_capability::CapabilityContract;
use nomos_contracts::{
    Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity,
    SchemaId,
};

/// The capability this crate answers.
///
/// Named for what a caller gets -- one automated code review's own observed finding, tied
/// to a diff -- rather than for the vendor that produced it: a second review tool (a
/// second CodeRabbit-shaped source, or an unrelated static-analysis bot posting the
/// identical shape of comment) offering the same shape of answer must be able to name this
/// capability honestly.
pub const CAPABILITY: &str = "nomos.cap.review.finding";

/// The payload schema every answer to this capability is stamped with.
pub const SCHEMA: &str = "nomos.review.finding.v1";

/// The contract version. Not a crate version: a caller reads against the contract.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// The strongest anything may claim for this capability.
///
/// [`FactVariant::RuntimeObserved`]: reading one already-posted review comment by its own
/// stable id is a live read of GitHub's current record for it, not a cached or derived one
/// -- whether the comment still exists, and exactly what it currently says, can change
/// between two reads the same way an issue's state can.
///
/// Soundness [`Assurance::Sound`] at the ceiling: every canonical field a provider reports
/// is a field the vendor's own response actually carried, or a value read from a portion
/// of the response whose structure and version the vendor's own data explicitly marks --
/// never a field this capability infers or a structural guess about an unversioned
/// convention.
///
/// Completeness [`Assurance::Sound`] as well, deliberately not pinned to what today's one
/// provider achieves: a provider that canonically mapped a review tool's full comment
/// record -- effort classification, thread resolution state, the diff hunk itself --
/// rather than this crate's curated subset, could honestly claim it.
///
/// [`IncrementalGranularity::None`]: no provider of this capability implements any
/// incremental or cached refresh yet -- every materialize call is a full, fresh
/// observation of one already-identified comment. Nothing here forecloses a future
/// provider that tracked a vendor's own change token; nothing here invents a claim no
/// provider has earned.
#[must_use]
pub const fn Ceiling() -> Guarantee
{
    return Guarantee::New(
        FactVariant::RuntimeObserved,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::None,
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
        summary: "One automated code review's own observed finding -- category, severity, \
                  the file and line it concerns, and the reviewer's own message -- as its \
                  own tool reported them, kept apart from any judgment about whether the \
                  finding is valid or what it bears on."
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
    /// for.
    #[test]
    fn Test_The_Ceiling_Should_Admit_A_Weaker_Offer()
    {
        let mut registry = Registry::New();
        registry.Declare(Capability_Contract()).expect("declared once");

        let approximate = ProviderOffer {
            provider: ProviderId::New("nomos.test.weak"),
            capability: Capability(),
            version: CONTRACT_VERSION,
            guarantee: Guarantee::New(
                FactVariant::Predicted,
                Assurance::Unsound,
                Assurance::Unknown,
                IncrementalGranularity::None,
            ),
        };

        assert_eq!(registry.Offer(approximate), Ok(()));
    }

    /// And refuses one above it -- a claim of `IncrementalGranularity::Symbol` would be a
    /// provider asserting a cached, sub-finding incremental refresh no honest connector has
    /// built.
    #[test]
    fn Test_The_Ceiling_Should_Refuse_A_Claim_Of_Symbol_Incrementality()
    {
        let mut registry = Registry::New();
        registry.Declare(Capability_Contract()).expect("declared once");

        let overclaimed = ProviderOffer {
            provider: ProviderId::New("nomos.test.optimistic"),
            capability: Capability(),
            version: CONTRACT_VERSION,
            guarantee: Guarantee::New(
                FactVariant::RuntimeObserved,
                Assurance::Sound,
                Assurance::Sound,
                IncrementalGranularity::Symbol,
            ),
        };

        assert!(
            registry.Offer(overclaimed).is_err(),
            "a provider claiming symbol-level incrementality for a capability whose ceiling \
             states none would satisfy every rule that needs it"
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

    // This capability's own id must never be able to satisfy a Requirement for any other
    // capability. Not asserted here by spelling a sibling's literal id:
    // `tests/contract/tests/boundaries/capabilities.rs::
    // Test_A_Capability_Id_Should_Be_Written_In_One_Crate` already refuses any capability
    // id written in more than one crate's `src`, and writing that string here to check it
    // would be exactly the collision that test exists to catch.
}
