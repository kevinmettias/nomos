//! Band 23 — the `nomos.cap.controlflow.reachability` contract, owned by neither its
//! provider nor any rule that reads it.
//!
//! # Why this earned a crate immediately
//!
//! `OD-CAPABILITY-002`'s criterion is contention: a contract earns its own crate the
//! moment a real second party is expected to offer against or read it, not only once one
//! actually has. `crates/rules/nomos-rules` reads this capability from the day it is
//! written (`Check_Unread_Reaches_A_Finding`), the same real-second-party shape
//! `nomos.cap.dependency.edges` was written under — so the contract lives here, below both
//! its provider (`nomos-lang-rust`, band 25) and the rule that reads it (`nomos-rules`,
//! band 30), named by neither.
//!
//! # What is here
//!
//! What the parties agree on — the capability's identity, the contract version, the
//! ceiling, the summary, and the schema every answer is stamped with — plus the payload
//! shape and its canonical reader, the same division `nomos-cap-dependency` draws between
//! its `contract` and its `payload` modules.

#![forbid(unsafe_code)]

mod contract;
mod payload;

pub use contract::{
    Capability, Capability_Contract, Ceiling, CAPABILITY, CONTRACT_VERSION, Payload_Schema, SCHEMA,
};
pub use payload::{ArmShape, Encode_Payload, Parse_Payload, PayloadRefusal, ReachabilityPayload, ReachabilitySite};
