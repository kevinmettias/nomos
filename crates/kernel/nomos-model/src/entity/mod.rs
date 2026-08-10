//! The three canonical entity kinds, and the identity all three carry.
//!
//! Each kind is paired with an enum saying which sort of it this one is. The pairing is
//! deliberate and stays split across two files apiece: the kind enum is a closed
//! vocabulary that grows when a language or a deployment target introduces a new sort of
//! thing, and the entity struct is a shape that grows when Nomos learns to record
//! something new about one. The two change for different reasons and on different
//! schedules.

mod artifact;
mod artifact_kind;
mod entity_id;
mod resource;
mod resource_kind;
mod symbol;
mod symbol_kind;

pub use artifact::Artifact;
pub use artifact_kind::ArtifactKind;
pub use entity_id::EntityId;
pub use resource::Resource;
pub use resource_kind::ResourceKind;
pub use symbol::Symbol;
pub use symbol_kind::SymbolKind;
