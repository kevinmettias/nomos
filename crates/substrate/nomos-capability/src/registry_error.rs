//! Every way a composition is refused before anything is resolved.
//!
//! # Why this file sits at the crate root rather than inside `registry/`
//!
//! It was `registry/error.rs`, declaring `Error`, and `registry.rs` published it as
//! `pub use error::Error as RegistryError;`. Three rules pull against that shape, and
//! moving the file here is the only arrangement that satisfies all three at once:
//!
//! - `file-name-matches-declared-type` requires a file to be named after the public type it
//!   declares, so a type called `RegistryError` forces the file to be `registry_error`.
//! - `check-tree-legibility` forbids a file repeating its parent folder, so
//!   `registry/registry_error.rs` is out.
//! - `check-facade-surface` counts `pub use x::Y as Z;` a second public name for one item, so
//!   keeping `Error` and qualifying it at the facade is out too.
//!
//! The public path is unchanged: `lib.rs` re-exports this, so
//! `nomos_capability::RegistryError` is what it always was.

use nomos_contracts::CapabilityId;

use crate::RegistryErrorKind;

/// Why a declaration or an offer was refused, always naming the capability it was about.
///
/// The capability is the type's and not the kind's. Every refusal here is a refusal
/// *about one capability*, so a caller reads which one without matching on a reason it
/// does not otherwise care about, and a new reason cannot forget to say which capability
/// it concerns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryError
{
    pub capability: CapabilityId,
    pub kind: RegistryErrorKind,
}

impl core::fmt::Display for RegistryError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", Refusal_Message(&self.capability, &self.kind));
    }
}

/// The one human-readable sentence for `capability`'s refusal, gathered here rather than
/// assembled across several `write!` calls sharing one `Formatter` — a `RegistryError` is
/// one sentence, not several fragments.
fn Refusal_Message(capability: &CapabilityId, kind: &RegistryErrorKind) -> String
{
    use crate::OfferRefusal;

    return match kind
    {
        RegistryErrorKind::AlreadyDeclared => format!("{capability} is already declared"),
        RegistryErrorKind::Offer { provider, refusal } => match refusal
        {
            OfferRefusal::ForUndeclared => format!(
                "{provider} offers {capability}, which no contract declares. An offer \
                 against nothing is a capability with no agreed meaning"
            ),
            OfferRefusal::ExceedsCeiling => format!(
                "{provider} claims more for {capability} than its contract permits. A \
                 provider grading its own work is how a syntactic answer comes to satisfy \
                 a rule that needs resolution"
            ),
            OfferRefusal::Duplicate => format!("{provider} already offers {capability}"),
        },
    };
}

impl std::error::Error for RegistryError
{}
