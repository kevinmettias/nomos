//! Composite identity: what makes two observations the same thing.
//!
//! [`CompositeIdentity`] is the answer, and the three types beside it are its components:
//! the shape that survives a rename, where the declaration came from, and how two
//! declarations that legitimately share a name are told apart. Each is a separate file
//! because each is separately replaceable — a language package may state a different
//! [`IdentityPolicy`] without touching what an identity is made of.

// How an identity moves from one form to another, beneath the identity it is of.
mod transition;
mod transition_kind;

pub use transition::IdentityTransition;
pub use transition_kind::IdentityTransitionKind;

mod composite_identity;
mod identity_policy;
mod source_provenance;
mod structural_fingerprint;

pub use composite_identity::CompositeIdentity;
pub use identity_policy::IdentityPolicy;
pub use source_provenance::SourceProvenance;
pub use structural_fingerprint::StructuralFingerprint;
