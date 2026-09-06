//! Which crate and zone a finding's location falls under, read from `nomos_rules::ZONES` --
//! the one declared architecture this workspace has, per `OD-RULES-020`.
//!
//! This is real and mechanical only when a location is a source path under `crates/`: every
//! crate in this workspace lives at `crates/<zone-directory>/<crate-name>/...`, so the crate
//! name is the path's third segment. A location that is not shaped that way -- a package
//! name (`dependency/violations.rs`'s own `subject_name`), a doc or test path, or no
//! location at all -- has no architectural component this function can name, and it says so
//! by returning `None` rather than guessing. `docs/records/OD-HOST-010-*.md` names this as
//! one of the three walk-outward targets this crate answers honestly rather than richly.

use nomos_rules::{Zone, Zone_Of};
use serde::Serialize;

/// The crate and zone a finding's location resolves to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ArchitecturalComponent
{
    /// The crate name read off the path's `crates/<zone-directory>/<crate-name>/...`
    /// convention.
    pub crate_name: String,
    /// That crate's own declared zone, from `nomos_rules::ZONES` -- the same table
    /// `tests/contract/tests/boundaries` checks every real workspace member against.
    pub zone: String,
}

impl ArchitecturalComponent
{
    /// The architectural component `location` falls under, or `None` when `location` is
    /// not a `crates/.../...` path, or names a crate `ZONES` does not declare (which
    /// `Test_Every_Member_Should_Declare_A_Band` in `tests/contract` already refuses to let
    /// happen for a real workspace member, so `None` here means "not a crate path" in
    /// practice, not "an undeclared crate").
    #[must_use]
    pub(crate) fn Of(location: &str) -> Option<Self>
    {
        let mut segments = location.split('/');
        if segments.next() != Some("crates")
        {
            return None;
        }
        let _zone_directory = segments.next()?;
        let crate_name = segments.next()?;

        let zone: Zone = Zone_Of(crate_name)?;

        return Some(Self { crate_name: crate_name.to_owned(), zone: zone.to_string() });
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Of_Should_Resolve_A_Real_Crate_Path_To_Its_Declared_Zone()
    {
        let component = ArchitecturalComponent::Of("crates/rules/nomos-rules/src/lib.rs").expect("nomos-rules is a declared zone member");

        assert_eq!(component.crate_name, "nomos-rules");
        assert_eq!(component.zone, "Rules");
    }

    #[test]
    fn Test_Of_Should_Resolve_A_Host_Crate_To_The_Host_Zone()
    {
        let component = ArchitecturalComponent::Of("crates/host/nomos-cli/src/main.rs").expect("nomos-cli is a declared zone member");

        assert_eq!(component.crate_name, "nomos-cli");
        assert_eq!(component.zone, "Host");
    }

    #[test]
    fn Test_Of_Should_Be_None_For_A_Non_Crate_Path()
    {
        assert!(ArchitecturalComponent::Of("docs/records/OD-RULES-020.md").is_none());
        assert!(ArchitecturalComponent::Of("README.md").is_none());
    }

    #[test]
    fn Test_Of_Should_Be_None_For_A_Bare_Package_Name()
    {
        assert!(ArchitecturalComponent::Of("nomos-rules").is_none());
    }
}
