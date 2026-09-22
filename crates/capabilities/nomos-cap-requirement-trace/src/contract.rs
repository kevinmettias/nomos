//! The agreement itself, and why this crate carries its own one provider rather than
//! waiting beside it the way its `nomos-cap-*` siblings did.
//!
//! `OD-CAPABILITY-002`'s real criterion is contention: a capability contract extracts into
//! its own crate once a second real *provider* names it, not merely once a rule reads it.
//! `nomos.cap.requirement.trace` has exactly one provider, this crate's own, so a split
//! crate the way `nomos.cap.naming.policy` and its four siblings are split (contract here,
//! provider in `nomos-repo-policy`) would buy independence nothing spends.
//! `nomos-connector-coderabbit` held the identical bundled shape for
//! `nomos.cap.review.finding` and no longer does: `OD-ROADMAP-005`'s fifth decision split
//! that one on the owner's own sequencing rather than on contention, which leaves this
//! crate the only bundled contract in the workspace and `OD-CAPABILITY-002`'s criterion
//! exactly as it was. Nothing in that decision reaches this capability, whose rule reads
//! no vendor's name.
//!
//! # Why this is Capability Contract zone rather than Provider zone
//!
//! `nomos-rules` (Rules zone) may depend on Capability Contract zone but not Provider zone
//! at all — `crates/rules/nomos-rules/src/checks/dependency/zones.rs`'s own `Permits` says
//! so. Splitting this crate's provider into Provider zone would make
//! `nomos.cap.requirement.trace` structurally unreachable by any rule, not merely
//! undesirable, which is why this crate is classified Capability Contract zone even though
//! its provider (unlike every split sibling's) lives beside its contract in the same crate.

use nomos_capability::CapabilityContract;
use nomos_contracts::{
    Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity,
    SchemaId,
};

/// The capability this crate answers.
///
/// Named for what a caller gets — whether this repository's own committed requirement
/// assessments still resolve — rather than for the corpus or the directory they are read
/// from, the same "name the answer, not the source" reason `nomos.cap.goals.policy` is
/// named for its resolved answer rather than for `standards.json`.
pub const CAPABILITY: &str = "nomos.cap.requirement.trace";

/// The payload schema every answer to this capability is stamped with.
pub const SCHEMA: &str = "nomos.requirement.trace.v1";

/// The contract version. Not a crate version: a caller reads against the contract.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// The strongest anything may claim for this capability.
///
/// [`FactVariant::Syntactic`] — every judgment this capability answers is either "does this
/// exact symbol occur in this exact file's text" or "does this exact filename exist," with
/// no name resolution and no inference over either. The four `nomos.cap.*.policy`
/// capabilities share this ceiling for the identical reason.
///
/// Both [`Assurance`] axes [`Assurance::Sound`]: a provider that read every committed
/// assessment and checked every site, gap and record it names against the real tree has
/// read everything there is to read, and reports nothing it did not find there.
///
/// [`IncrementalGranularity::WholeWorkspace`]: the corpus this capability answers about is
/// the whole committed set under `tests/contract/requirements/`, not one assessment at a
/// time — the same granularity `nomos.cap.goals.policy` states for the identical reason,
/// a repository's own declared corpus is one thing, not a population of independent parts.
#[must_use]
pub const fn Ceiling() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
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
        summary: "Whether this repository's own committed requirement assessments, under \
                  tests/contract/requirements/, still resolve against the real tree -- each \
                  named site, gap and governing record either found or reported stale, and \
                  each Diverges/NotBinding/Partial entry checked for the record or gap it \
                  owes. OD-TRACE-001."
            .to_owned(),
        ceiling: Ceiling(),
    };
}
