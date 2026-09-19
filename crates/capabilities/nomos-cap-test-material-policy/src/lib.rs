//! Zone: Capability Contract — the `nomos.cap.test.material.policy` contract, owned by neither
//! its provider nor any rule that reads it.
//!
//! # Why this earned a crate immediately, like its five siblings
//!
//! `OD-CAPABILITY-002`'s criterion is contention, not principle: a contract lives beside
//! its only provider until a second real party names it. This capability does not wait,
//! for the same reason its five siblings did not: the three security checks and the
//! test-or-example exemption that seven more rules read are a real second party from the
//! day this crate was written.
//!
//! # What `OD-RULES-011` this satisfies
//!
//! A sixth `OD-RULES-011` instance, after naming, numeric limits, scripting, words and
//! goals policy: a repository's own fixture locations, materialized at
//! `EvidenceClass::Verified` by its provider and read, judged and re-emitted as `Finding`s
//! at `EvidenceClass::Derived` by the rules that consume it. A repository declaring nothing
//! keeps each rule's own hardcoded clause list, clause for clause.
//!
//! # A bare list
//!
//! This capability is simpler than every sibling before it: a bare list, because a
//! repository's own fixture location has no scope to qualify and no value to carry — it is
//! one flat set the whole workspace shares, the same way `words.approved_abbreviations` is
//! one flat JSON array. A location is a repository-relative directory prefix: a source
//! whose normalized path is the location, or sits under it, is that repository's test
//! material.
//!
//! # What is here
//!
//! The capability's identity, contract version, ceiling and summary ([`contract`]); and
//! the payload shape and its canonical reader ([`test_material_policy_payload`]).
//!
//! What is not here: the toolchain-fixed clauses (`tests/`, `examples/`, `_test.go`, …),
//! which stay compiled into each rule that reads this capability as the defaults this
//! declaration extends rather than replaces; a provider that actually reads
//! `nomos-test-material.json`; and the rules themselves. All three are this decision's own
//! next increments.

#![forbid(unsafe_code)]

mod contract;
mod test_material_policy_payload;

pub use contract::{Capability, Capability_Contract, Ceiling, CAPABILITY, CONTRACT_VERSION, Payload_Schema, SCHEMA};
pub use test_material_policy_payload::{Encode_Payload, Parse_Payload, Refusal, TestMaterialPolicyPayload};
