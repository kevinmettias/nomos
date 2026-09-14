//! Zone: Capability Contract — the `nomos.cap.lint.diagnostics` contract, owned by neither its provider nor
//! any rule that reads it.
//!
//! # Why this earned a crate immediately, unlike a capability with no second party yet
//!
//! `OD-CAPABILITY-002`'s criterion is contention, not principle: a contract lives beside
//! its only provider until a second real party names it, the same way
//! `nomos.cap.module.index` still lives inside `nomos-lang-rust`. This capability does not
//! wait, for the same reason `nomos.cap.dependency.edges` and `nomos.cap.controlflow.
//! reachability` both did not: `nomos-rules`' own `Check_Lint_Diagnostics` reads it from
//! the day this crate was written, alongside `nomos-lang-rust-clippy`'s one real offer. A
//! rule that depended on the provider crate directly to reach `Capability()`/
//! `Payload_Schema()` would be naming its provider, which is exactly the decision a
//! registry-mediated capability exists to keep out of a rule's own hands.
//!
//! # What `OD-RULES-010` this satisfies
//!
//! `OD-RULES-010` decided the first real `ToolProvider`'s output is a fact a native rule
//! judges, not a `Finding` the tool emits directly. This crate is that fact's home: what a
//! lint tool reported about one workspace member, materialized at
//! `EvidenceClass::Verified` by its provider and read, judged and re-emitted as `Finding`s
//! at `EvidenceClass::Derived` by the rule that consumes it.
//!
//! # What is here
//!
//! What the parties agree on — the capability's identity, the contract version, the
//! ceiling, the summary, and the schema every answer is stamped with — plus the payload
//! shape and its canonical reader, the same division `nomos-cap-syntax` and
//! `nomos-cap-dependency` both draw between their own `contract` and `payload` modules.

#![forbid(unsafe_code)]

mod contract;
mod payload;

pub use contract::{Capability, Capability_Contract, Ceiling, CAPABILITY, CONTRACT_VERSION, Payload_Schema, SCHEMA};
pub use payload::{Encode_Payload, Parse_Payload};
pub use payload::diagnostics_payload::DiagnosticsPayload;
pub use payload::lint_diagnostic::LintDiagnostic;
pub use payload::lint_level::LintLevel;
pub use payload::refusal::Refusal;
