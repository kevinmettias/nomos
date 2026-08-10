//! What an actor is permitted to do, and what an operation does to the world.
//!
//! The two are separated because an actor's grants are not a function of what an
//! operation touches: approving somebody else's change touches nothing and requires more
//! authority than making one. [`MutationClass::Required_Authority`] is the only place the
//! two meet, and it is a default rather than a definition.

mod authority_class;
mod mutation_class;

pub use authority_class::AuthorityClass;
pub use mutation_class::MutationClass;
