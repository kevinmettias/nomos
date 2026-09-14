//! Zone: Capability Contract — the `nomos.cap.words.policy` contract, owned by neither its provider nor any
//! rule that reads it.
//!
//! # Why this earned a crate immediately, unlike a capability with no second party yet
//!
//! `OD-CAPABILITY-002`'s criterion is contention, not principle: a contract lives beside
//! its only provider until a second real party names it. This capability does not wait,
//! for the same reason its three siblings did not: `nomos-rules`' abbreviations rule is a
//! real second party from the day this crate was written.
//!
//! # What `OD-RULES-011` this satisfies
//!
//! A fourth `OD-RULES-011` instance, after naming, numeric limits and scripting policy: a
//! repository's own additional approved abbreviations, materialized at
//! `EvidenceClass::Verified` by its provider and read, judged and re-emitted as `Finding`s
//! at `EvidenceClass::Derived` by the rule that consumes it.
//!
//! # A fourth shape
//!
//! Naming and limits are scope-qualified rows; scripting is a scalar plus a list. This
//! capability is simpler still: a bare list, because a repository's own vocabulary
//! addition has no scope to qualify — it is one flat set the whole workspace shares, the
//! same way `words.approved_abbreviations` is one flat JSON array in `standards.json`
//! rather than a `languages.*` block.
//!
//! # What is here
//!
//! The capability's identity, contract version, ceiling and summary ([`contract`]); and
//! the payload shape and its canonical reader ([`payload`]).
//!
//! What is not here: the default approved/banned vocabulary and the abbreviation
//! judgment itself, which belong to the rule that reads this capability, not to the
//! capability that carries a repository's own *additions* to that vocabulary; a provider
//! that actually reads `standards.json`; and the rule itself. All three are this
//! decision's own next increments.

#![forbid(unsafe_code)]

mod contract;
mod payload;

pub use contract::{Capability, Capability_Contract, Ceiling, CAPABILITY, CONTRACT_VERSION, Payload_Schema, SCHEMA};
pub use payload::{Encode_Payload, Parse_Payload, Refusal, WordsPolicyPayload};
