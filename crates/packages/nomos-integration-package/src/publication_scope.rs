//! The publication scope a materialization intent gives its target.
//!
//! `OD-PACKAGE-005`'s decision: a materialized asset's publication scope decides whether it
//! may enter repository-distributed state, the scope is declared beside the ownership class
//! by whatever places the asset, and an asset with no recorded scope defaults to `Local` --
//! an asset wrongly kept local costs a rerun once somebody notices, while an asset wrongly
//! published is in a commit, possibly on a remote nobody here controls, with no diff to
//! recover the world where it never went.
//!
//! Orthogonal to [`crate::OwnershipClass`] on purpose: that record's own composition table
//! shows `GeneratedOwned` against both `Shared` (the rendered diagram) and `Ephemeral` (a
//! scratch build root), so folding scope into ownership would name combinations rather than
//! properties. The three names are transcribed from `OD-PACKAGE-005`'s own table verbatim,
//! by the same manual-match pattern `OwnershipClass` and `nomos-tool-package`'s `Family` use.

const EPHEMERAL_LABEL: &str = "Ephemeral";
const LOCAL_LABEL: &str = "Local";
const SHARED_LABEL: &str = "Shared";

/// One of `OD-PACKAGE-005`'s three publication scopes.
///
/// The axis is about repository-distributed state specifically -- a clone, a push, a
/// review -- and not every channel bytes could leave a machine through; `OD-PACKAGE-005`'s
/// version-2 amendment narrowed `Shared` to exactly that, and this enum inherits the
/// narrowing rather than widening it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PublicationScope
{
    /// Should not survive the session that produced it.
    Ephemeral,
    /// Persists on the machine that wrote it, and is never published.
    Local,
    /// Enters repository-distributed state: committed, reviewed, handed to every clone.
    Shared,
}

impl PublicationScope
{
    /// The scope an intent that declares none resolves to.
    ///
    /// `OD-PACKAGE-005`: "An asset with no recorded scope defaults to `Local`, and publishing
    /// it is refused." The same asymmetry `OD-PACKAGE-004` argues for ownership, pointed the
    /// same direction for the same reason.
    pub const UNDECLARED: Self = Self::Local;

    /// The label a manifest file spells this scope with -- `OD-PACKAGE-005`'s own spelling.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Ephemeral => EPHEMERAL_LABEL,
            Self::Local => LOCAL_LABEL,
            Self::Shared => SHARED_LABEL,
        };
    }

    /// The scope a label names, or `None` if it names none of the three.
    #[must_use]
    pub fn Of_Label(label: &str) -> Option<Self>
    {
        return match label
        {
            EPHEMERAL_LABEL => Some(Self::Ephemeral),
            LOCAL_LABEL => Some(Self::Local),
            SHARED_LABEL => Some(Self::Shared),
            _ => None,
        };
    }
}

impl core::fmt::Display for PublicationScope
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    const EVERY_SCOPE: [PublicationScope; 3] = [PublicationScope::Ephemeral, PublicationScope::Local, PublicationScope::Shared];

    #[test]
    fn Test_Every_Scope_Should_Round_Trip_Through_Its_Label()
    {
        for scope in EVERY_SCOPE
        {
            assert_eq!(PublicationScope::Of_Label(scope.Label()), Some(scope));
        }
    }

    #[test]
    fn Test_A_Label_None_Of_The_Three_Name_Should_Resolve_To_Nothing()
    {
        assert_eq!(PublicationScope::Of_Label("Public"), None);
        assert_eq!(PublicationScope::Of_Label("local"), None);
        assert_eq!(PublicationScope::Of_Label(""), None);
    }

    #[test]
    fn Test_The_Undeclared_Scope_Should_Be_Local()
    {
        assert_eq!(PublicationScope::UNDECLARED, PublicationScope::Local);
    }

    #[test]
    fn Test_Display_Should_Render_The_Label()
    {
        assert_eq!(PublicationScope::Shared.to_string(), "Shared");
    }
}
