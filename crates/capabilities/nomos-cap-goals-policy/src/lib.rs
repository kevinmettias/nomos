//! Zone: Capability Contract — the `nomos.cap.goals.policy` contract, owned by neither its provider nor any
//! rule that reads it.
//!
//! # Why this earned a crate immediately, unlike a capability with no second party yet
//!
//! `OD-CAPABILITY-002`'s criterion is contention, not principle: a contract lives beside
//! its only provider until a second real party names it. This capability does not wait, for
//! the same reason `nomos.cap.naming.policy`, `nomos.cap.limits.policy`,
//! `nomos.cap.scripting.policy` and `nomos.cap.words.policy` did not — the rule that holds
//! a repository's declared purposes to its declared parts is a real second party from the
//! day this crate was written, alongside its own provider's one real offer.
//!
//! # Which `OD-RULES-011` this satisfies
//!
//! `OD-RULES-011` decided a rule's configurable parameters are read as a capability fact
//! through the existing `FactReader` seam `OD-RULES-010` already established, not embedded
//! as a Rust constant or a hand-rolled predicate. This crate is that decision's fifth
//! instance, and the clearest case for it yet: code-standards' own `check-goal-traceability`
//! reads *nothing but* the declaration. There is no source text in its judgment at all, so
//! there is no version of the rule that could have been written with its parameters
//! compiled in and still meant anything.
//!
//! # A shape none of the four siblings has
//!
//! `nomos_cap_naming_policy` and `nomos_cap_limits_policy` are scope-qualified rows;
//! `nomos_cap_scripting_policy` is a scalar plus a list; `nomos_cap_words_policy` is a bare
//! list. This one is a small graph — a set of goals, a set of parts, and the edges between
//! them — because the rule's whole question is whether the two sets line up across those
//! edges. That is also why [`payload::SubsystemDeclaration`] gets a line of its own in the
//! encoding rather than being implied by the edges: a part that serves nothing is the
//! defect the rule is looking for, and an encoding that could not say it would hide it.
//!
//! # What is here
//!
//! The capability's identity, contract version, ceiling and summary ([`contract`]); and the
//! payload shape and its canonical reader ([`payload`]).
//!
//! What is not here: a provider that actually reads `standards.json`, and a rule that reads
//! this capability. Both are this decision's own next increments.

#![forbid(unsafe_code)]

mod contract;
mod payload;

pub use contract::{Capability, Capability_Contract, Ceiling, CAPABILITY, CONTRACT_VERSION, Payload_Schema, SCHEMA};
pub use payload::{Encode_Payload, GoalsPolicyPayload, Parse_Payload, Refusal, SubsystemDeclaration};
