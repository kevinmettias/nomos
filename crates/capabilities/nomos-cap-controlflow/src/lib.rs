//! Zone: Capability Contract — the `nomos.cap.controlflow.reachability` contract, owned by neither its
//! provider nor any rule that reads it.
#![doc = include_str!("../docs/api/lib.md")]
#![forbid(unsafe_code)]

mod contract;
mod arm_shape;

pub use contract::{
    Capability, Capability_Contract, Ceiling, CAPABILITY, CONTRACT_VERSION, Payload_Schema, SCHEMA,
};
pub use arm_shape::{ArmShape, Encode_Payload, Parse_Payload};
pub use arm_shape::payload_refusal::PayloadRefusal;
pub use arm_shape::reachability_payload::ReachabilityPayload;
pub use arm_shape::reachability_site::ReachabilitySite;
