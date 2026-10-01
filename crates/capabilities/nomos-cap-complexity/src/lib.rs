//! Zone: Capability Contract — the `nomos.cap.metric.complexity` contract, owned by neither its
//! provider nor the rule that reads it.

#![doc = include_str!("../docs/api/lib.md")]
#![forbid(unsafe_code)]

mod contract;
mod payload;

pub use contract::{Capability, Capability_Contract, Ceiling, CAPABILITY, CONTRACT_VERSION, Payload_Schema, SCHEMA};
pub use payload::aggregation::Aggregation;
pub use payload::complexity_payload::ComplexityPayload;
pub use payload::directionality::Directionality;
pub use payload::function_complexity::FunctionComplexity;
pub use payload::metric_descriptor::MetricDescriptor;
pub use payload::payload_refusal::PayloadRefusal;
pub use payload::{Complexity_Descriptor, Encode_Payload, Parse_Payload};
