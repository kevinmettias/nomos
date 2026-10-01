//! Which of the two things being refused was refused.
//!
//! # Why this file sits at the crate root rather than inside `registry/`
//!
//! It was `registry/error_kind.rs`, declaring `ErrorKind`, and `registry.rs` published it
//! as `pub use error_kind::ErrorKind as RegistryErrorKind;`. The same three rules that put
//! [`crate::RegistryError`] here apply one file along: the type's public name forces the
//! file to be `registry_error_kind`, `registry/registry_error_kind.rs` would repeat its
//! parent folder, and keeping `ErrorKind` would need the alias. [`crate::RegistryError`]'s
//! header states the whole of it.

use nomos_contracts::ProviderId;

use crate::OfferRefusal;

/// Which of the two things being refused was refused.
///
/// The three offer refusals are one variant carrying an [`OfferRefusal`] rather than
/// three, because all three are about an offer and an offer has a provider. Declaring a
/// contract has none, which is the whole distinction this level draws.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistryErrorKind
{
    /// A second contract for a capability that already has one.
    AlreadyDeclared,
    /// An offer that will not stand, and who made it.
    Offer
    {
        provider: ProviderId,
        refusal: OfferRefusal,
    },
}
