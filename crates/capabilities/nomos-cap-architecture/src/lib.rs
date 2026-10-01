//! Zone: Capability Contract — the `nomos.cap.architecture.declaration` contract, owned by
//! neither its provider nor any party that reads it.
//!
//! # What this is for
//!
//! `OD-RULES-003` decided that a declared architecture is data and the observed dependency
//! graph is a fact a capability establishes, and named three prerequisites. Two were built:
//! `nomos-cap-dependency` establishes the graph, and `Check_Dependency_Direction` reads it
//! through a `FactReader`. The third — "a place to author a declared architecture as data
//! rather than a Rust `const` table" — was unowned, and `OD-RULES-029` measured what its
//! absence cost: `ZONES` named this workspace's own crates as string literals, so on any
//! other repository the whole family reported that every member had declared nothing. This
//! crate is that third prerequisite.
//!
//! # What travels, and why all of it
//!
//! `OD-RULES-029` is explicit that the declaration is a **triple** and not a membership map:
//! "a finite set of named components, an order over them, and named exceptions the order
//! alone cannot express." Shipping the membership and leaving the order compiled "would move
//! one-third of the triple and fix nothing for the repository it was moved for: a foreign
//! workspace would author `crate -> zone` in *this* workspace's twelve-zone vocabulary and
//! still be judged by *this* workspace's permitted-edge matrix."
//!
//! So [`ArchitecturePayload`] carries the vocabulary itself. There is no component named
//! anywhere in this crate. `OD-RULES-023`'s write authorities travel with it for the same
//! reason and by the same argument: a declared allow-list over the same observed fact, named
//! by the repository rather than by whichever rule judges it.
//!
//! # Why this is not an `OD-RULES-011` family
//!
//! Because `OD-RULES-029` decided it is not, and said why: an `OD-RULES-011` family is a
//! repository's *policy parameters*, parameterizing a judgment the rule keeps making, and
//! this is a description the rule judges *against*. That the two happen to arrive through the
//! same `FactReader` seam is a fact about the seam, not about the content — the seam is
//! `OD-RULES-010`'s and it predates both.
//!
//! # What is here
//!
//! The capability's identity, contract version, ceiling and summary ([`contract`]); and the
//! declaration's shape, its canonical encoding, and the generic relation queries every party
//! asks of it ([`architecture_payload`]). The queries are here rather than in a rule because they
//! are lookups over a declaration and not judgments — see [`architecture_payload`]'s own doc.
//!
//! What is not here: a provider that reads any particular configuration surface, and any
//! rule. `nomos-repo-policy`'s `architecture` module is the first provider and `nomos-rules`'
//! three dependency checks are the first rules, and neither belongs to this contract.

#![forbid(unsafe_code)]

mod contract;
mod architecture_payload;

pub use contract::{Capability, Capability_Contract, Ceiling, CAPABILITY, CONTRACT_VERSION, Payload_Schema, SCHEMA};
pub use architecture_payload::authority::Authority;
pub use architecture_payload::depended::Depended;
pub use architecture_payload::depending::Depending;
pub use architecture_payload::exception::Exception;
pub use architecture_payload::membership::Membership;
pub use architecture_payload::permission::Permission;
pub use architecture_payload::refusal::Refusal;
pub use architecture_payload::ArchitecturePayload;
pub use architecture_payload::{Encode_Payload, Parse_Payload};
