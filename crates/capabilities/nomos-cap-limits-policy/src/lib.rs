//! Zone: Capability Contract — the `nomos.cap.limits.policy` contract, owned by neither its provider nor
//! any rule that reads it.
//!
//! # Why this earned a crate immediately, unlike a capability with no second party yet
//!
//! `OD-CAPABILITY-002`'s criterion is contention, not principle: a contract lives beside
//! its only provider until a second real party names it. This capability does not wait,
//! for the same reason `nomos.cap.naming.policy` did not: `nomos-rules`' threshold rules
//! (the file-size triggers, the parameter-count caps) are a real second party from the
//! day this crate was written, alongside its own provider's one real offer. A rule that
//! depended on the provider crate directly to reach `Capability()`/`Payload_Schema()`
//! would be naming its provider, which is exactly the decision a registry-mediated
//! capability exists to keep out of a rule's own hands.
//!
//! # What `OD-RULES-011` this satisfies
//!
//! `OD-RULES-011` decided a rule's configurable parameters are read as a capability fact
//! through the existing `FactReader` seam `OD-RULES-010` already established for tool-
//! provider output, not embedded as a Rust constant or a hand-rolled predicate, and named
//! "a threshold family (the file-size triggers)" as its own future instance of that same
//! decision — this crate is that instance's contract. A repository's own declared
//! numeric thresholds, materialized at `EvidenceClass::Verified` by its provider and
//! read, judged and re-emitted as `Finding`s at `EvidenceClass::Derived` by the rules
//! that consume it — the same `Require`-then-judge-then-emit shape every existing
//! capability in this workspace already uses, and the same shape `nomos-cap-naming-
//! policy` already draws for the sibling naming family.
//!
//! # What is here
//!
//! The capability's identity, contract version, ceiling and summary ([`contract`]); and
//! the payload shape and its canonical reader ([`limits_policy_payload`]) — scope-qualified numeric
//! rows rather than `nomos-cap-naming-policy`'s closed `Case` vocabulary, because a
//! threshold is a bare number a repository states, not a value drawn from a fixed style
//! set. The same division `nomos-cap-naming-policy`, `nomos-cap-lint`, `nomos-cap-
//! dependency` and `nomos-cap-dependency-policy` each draw between their own `contract`
//! and payload modules.
//!
//! What is not here: a provider that actually reads `standards.json`, and a rule that
//! reads this capability. Both are this decision's own next increments, per `OD-RULES-
//! 011`'s "What This Does Not Do".

#![forbid(unsafe_code)]

mod contract;
mod limits_policy_payload;

pub use contract::{Capability, Capability_Contract, Ceiling, CAPABILITY, CONTRACT_VERSION, Payload_Schema, SCHEMA};
pub use limits_policy_payload::policy_row::PolicyRow;
pub use limits_policy_payload::scope::Scope;
pub use limits_policy_payload::LimitsPolicyPayload;
pub use limits_policy_payload::{Encode_Payload, Parse_Payload};
pub use limits_policy_payload::refusal::Refusal;
