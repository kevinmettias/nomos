//! Zone: Capability Contract — the `nomos.cap.csharp.conditional_compilation` contract, owned by
//! neither its provider nor any rule that reads it.

#![doc = include_str!("../docs/api/lib.md")]
#![forbid(unsafe_code)]

mod contract;
mod payload;

pub use contract::{Capability, Capability_Contract, Ceiling, CAPABILITY, CONTRACT_VERSION, Payload_Schema, SCHEMA};
pub use payload::branch::Branch;
pub use payload::branch_state::BranchState;
pub use payload::build_selection::BuildSelection;
pub use payload::conditional_payload::ConditionalPayload;
pub use payload::conditional_region::ConditionalRegion;
pub use payload::definition_effect::DefinitionEffect;
pub use payload::file_definition::FileDefinition;
pub use payload::payload_refusal::PayloadRefusal;
pub use payload::{Encode_Payload, Parse_Payload};
