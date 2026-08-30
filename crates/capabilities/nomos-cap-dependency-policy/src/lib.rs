//! Band 23 — the `nomos.cap.dependency.policy` contract, owned by neither its provider nor
//! any rule that reads it.
//!
//! # Why this earned a crate immediately, unlike a capability with no second party yet
//!
//! `OD-CAPABILITY-002`'s criterion is contention, not principle: a contract lives beside
//! its only provider until a second real party names it, the same way `nomos.cap.module.
//! index` still lives inside `nomos-lang-rust`. This capability does not wait, for the
//! same reason `nomos.cap.lint.diagnostics` did not: `nomos-rules`' own `Check_Dependency_
//! Policy` reads it from the day this crate was written, alongside `nomos-lang-rust-deny`'s
//! one real offer. A rule that depended on the provider crate directly to reach
//! `Capability()`/`Payload_Schema()` would be naming its provider, which is exactly the
//! decision a registry-mediated capability exists to keep out of a rule's own hands.
//!
//! # What `OD-RULES-010` this satisfies
//!
//! `OD-RULES-010` decided the first real `ToolProvider`'s output is a fact a native rule
//! judges, not a `Finding` the tool emits directly, and named that decision's own scope as
//! the shape any `ToolProvider` takes, not a ruling scoped to `cargo clippy` alone. This
//! crate is that decision's second real instance rather than a second decision: `cargo
//! deny check bans licenses sources`'s own verdict about the workspace's resolved
//! dependency graph, materialized at `EvidenceClass::Verified` by its provider and read,
//! judged and re-emitted as `Finding`s at `EvidenceClass::Derived` by the rule that
//! consumes it. Nothing here widens `Finding`'s attribution scheme or gives
//! `EvidenceClass::Authoritative` a consumer — `cargo deny`'s own verdict is represented
//! honestly through the same judging-rule shape `nomos.cap.lint.diagnostics` already uses.
//!
//! # What is here
//!
//! What the parties agree on — the capability's identity, the contract version, the
//! ceiling, the summary, and the schema every answer is stamped with — plus the payload
//! shape and its canonical reader, the same division `nomos-cap-lint` and `nomos-cap-
//! dependency` both draw between their own `contract` and `payload` modules.
//!
//! # Why `IncrementalGranularity::WholeWorkspace`, unlike `nomos-cap-lint`'s `Project`
//!
//! `cargo clippy` diagnoses one workspace member's own code at a time, so `nomos.cap.
//! lint.diagnostics` materializes one fact per member. `cargo deny check bans licenses
//! sources` reasons over the whole resolved dependency graph at once — a duplicate-version
//! or license violation is a property of the graph, not attributable to whichever member
//! happens to pull the offending crate in. This capability's one real provider
//! materializes exactly one fact, addressed to the workspace as a whole, and this ceiling
//! states that honestly rather than inventing a per-member split the tool's own answer
//! does not support.

#![forbid(unsafe_code)]

mod contract;
mod payload;

pub use contract::{Capability, Capability_Contract, Ceiling, CAPABILITY, CONTRACT_VERSION, Payload_Schema, SCHEMA};
pub use payload::policy_payload::PolicyPayload;
pub use payload::policy_severity::PolicySeverity;
pub use payload::policy_violation::PolicyViolation;
pub use payload::refusal::Refusal;
pub use payload::{Encode_Payload, Parse_Payload};
