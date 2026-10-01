//! One `ToolProvider` a `ToolPackage` registers, and the family it classifies under.

use crate::family::Family;
use nomos_contracts::ProviderId;
use nomos_package::PackageVersion;

/// A registered `ToolProvider`: which one, what version of the underlying tool answers on
/// its behalf, and which of `OD-CAPABILITY-013`'s twelve FAMILY names it classifies under.
///
/// Structurally unlike every sibling manifest's own registration type on purpose: it
/// records nothing about delivery -- whether the provider launches a subprocess or answers
/// in-process. `P47-TOOLPROVIDER-HAS-NO-PACKAGE-2`'s own `done_when` frames family as the
/// axis this crate cares about, "rather than by whether they launch a subprocess," and
/// `OD-CAPABILITY-013` reserves delivery as a second, separate axis on a provider's own
/// registration that this crate does not attempt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolProviderRegistration
{
    /// The provider this registers.
    pub provider: ProviderId,
    /// The version of the tool behind that provider. `PKG-007`'s fourth domain, reused
    /// unchanged from [`nomos_package::ProviderRegistration`].
    pub tool_version: PackageVersion,
    /// Which of `OD-CAPABILITY-013`'s twelve closed FAMILY names this provider
    /// classifies under.
    pub family: Family,
}
