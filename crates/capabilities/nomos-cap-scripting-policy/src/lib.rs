//! Zone: Capability Contract — the `nomos.cap.scripting.policy` contract, owned by neither its provider nor
//! any rule that reads it.
//!
//! # Why this earned a crate immediately, unlike a capability with no second party yet
//!
//! `OD-CAPABILITY-002`'s criterion is contention, not principle: a contract lives beside
//! its only provider until a second real party names it. This capability does not wait,
//! for the same reason `nomos.cap.naming.policy` and `nomos.cap.limits.policy` did not:
//! `nomos-rules`' `declared-tooling-language-for-scripts` rule is a real second party from
//! the day this crate was written, alongside its own provider's one real offer.
//!
//! # What `OD-RULES-011` this satisfies
//!
//! `OD-RULES-011` decided a rule's configurable parameters are read as a capability fact
//! through the existing `FactReader` seam `OD-RULES-010` already established, not embedded
//! as a Rust constant or a hand-rolled predicate. This crate is a third instance of that
//! decision, after naming and numeric limits: a repository's own declared tooling language
//! and the script extensions it has decided against, materialized at
//! `EvidenceClass::Verified` by its provider and read, judged and re-emitted as `Finding`s
//! at `EvidenceClass::Derived` by the rule that consumes it.
//!
//! # A shape neither sibling capability has
//!
//! `nomos_cap_naming_policy::NamingPolicyPayload` and `nomos_cap_limits_policy::
//! LimitsPolicyPayload` are both scope-qualified rows: a repository-wide default a
//! language may override. This capability's declared tooling language is not scoped that
//! way — `check-script-discipline`'s own `spec.go` states it explicitly: refining a
//! declaration of "which language is my tooling" *by* language would be circular. So
//! [`scripting_policy_payload::ScriptingPolicyPayload`] is a plain optional scalar plus a list,
//! not a row table, and needs no `Scope` type of its own.
//!
//! # What is here
//!
//! The capability's identity, contract version, ceiling and summary ([`contract`]); and
//! the payload shape and its canonical reader ([`scripting_policy_payload`]).
//!
//! What is not here: a provider that actually reads `standards.json`, and a rule that
//! reads this capability. Both are this decision's own next increments.

#![forbid(unsafe_code)]

mod contract;
mod scripting_policy_payload;

pub use contract::{Capability, Capability_Contract, Ceiling, CAPABILITY, CONTRACT_VERSION, Payload_Schema, SCHEMA};
pub use scripting_policy_payload::{Encode_Payload, Parse_Payload, Refusal, ScriptingPolicyPayload};
