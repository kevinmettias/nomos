//! Which crate and architectural component a finding's location falls under, read from the
//! architecture the repository under check declares.
//!
//! This is real and mechanical only when a location is a source path under `crates/`: every
//! crate in this workspace lives at `crates/<group>/<crate-name>/...`, so the crate name is
//! the path's third segment. The directory a crate sits in is a filing convention and not the
//! declaration -- `architecture` is what says which component a crate belongs to, and the two
//! need not agree. A location that is not shaped that way -- a package
//! name (`dependency/violations.rs`'s own `subject_name`), a doc or test path, or no
//! location at all -- has no architectural component this function can name, and it says so
//! by returning `None` rather than guessing. `docs/records/OD-HOST-010-*.md` names this as
//! one of the three walk-outward targets this crate answers honestly rather than richly.

use nomos_cap_architecture::ArchitecturePayload;
use serde::Serialize;

/// The crate a finding's location resolves to, and the component its repository places it in.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ArchitecturalComponent
{
    /// The crate name read off the path's `crates/<group>/<crate-name>/...` convention.
    pub crate_name: String,
    /// The component that crate is declared in, in the repository's own words. A `String`
    /// rather than a variant of anything: the vocabulary is the repository's, so this crate
    /// has no set to draw it from and inventing one would put the ontology back.
    pub component: String,
}

impl ArchitecturalComponent
{
    /// The architectural component `location` falls under, or `None` when `location` is not a
    /// `crates/.../...` path, or names a crate `architecture` does not place (which
    /// `Test_Every_Member_Should_Declare_A_Band` in `tests/contract` already refuses to let
    /// happen for a real workspace member of *this* repository, so `None` here means "not a
    /// crate path" in practice rather than "an unplaced crate" -- for any other repository it
    /// means whichever of the two is true there).
    #[must_use]
    pub(crate) fn Of(architecture: &ArchitecturePayload, location: &str) -> Option<Self>
    {
        let mut segments = location.split('/');
        if segments.next() != Some("crates")
        {
            return None;
        }
        let _group_directory = segments.next()?;
        let crate_name = segments.next()?;

        let component = architecture.Component_Of(crate_name)?;

        return Some(Self { crate_name: crate_name.to_owned(), component: component.to_owned() });
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_cap_architecture::Membership;

    #[test]
    fn Test_Of_Should_Resolve_A_Crate_Path_To_Its_Declared_Component()
    {
        let component = ArchitecturalComponent::Of(&Declaration(), "crates/domain/billing/src/lib.rs").expect("billing is placed");

        assert_eq!(component.crate_name, "billing");
        assert_eq!(component.component, "Domain");
    }

    #[test]
    fn Test_Of_Should_Resolve_A_Second_Crate_To_Its_Own_Component()
    {
        let component = ArchitecturalComponent::Of(&Declaration(), "crates/api/http/src/main.rs").expect("http is placed");

        assert_eq!(component.crate_name, "http");
        assert_eq!(component.component, "Api");
    }

    #[test]
    fn Test_Of_Should_Be_None_For_A_Non_Crate_Path()
    {
        assert!(ArchitecturalComponent::Of(&Declaration(), "docs/records/OD-RULES-020.md").is_none());
        assert!(ArchitecturalComponent::Of(&Declaration(), "README.md").is_none());
    }

    #[test]
    fn Test_Of_Should_Be_None_For_A_Bare_Package_Name()
    {
        assert!(ArchitecturalComponent::Of(&Declaration(), "billing").is_none());
    }

    #[test]
    fn Test_Of_Should_Be_None_For_A_Crate_The_Declaration_Does_Not_Place()
    {
        assert!(ArchitecturalComponent::Of(&Declaration(), "crates/somewhere/unplaced/src/lib.rs").is_none());
    }

    /// A repository that declared nothing resolves nothing, rather than resolving against
    /// whatever nomos happens to know about itself.
    #[test]
    fn Test_Of_Should_Be_None_When_The_Repository_Declared_Nothing()
    {
        assert!(ArchitecturalComponent::Of(&ArchitecturePayload::default(), "crates/domain/billing/src/lib.rs").is_none());
    }

    /// A declaration in a vocabulary this workspace does not use -- the point being that this
    /// module resolves whatever the repository under check declares, not what nomos knows.
    fn Declaration() -> ArchitecturePayload
    {
        return ArchitecturePayload {
            components: vec!["Domain".to_owned(), "Api".to_owned()],
            membership: vec![
                Membership { package: "billing".to_owned(), component: "Domain".to_owned() },
                Membership { package: "http".to_owned(), component: "Api".to_owned() },
            ],
            ..ArchitecturePayload::default()
        };
    }
}
