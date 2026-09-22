//! The ownership class a materialization intent gives its target.
//!
//! `OD-PACKAGE-004`'s decision: an asset's ownership class decides what regeneration does
//! to it, the class is declared by whatever places the asset rather than by the asset
//! itself, and an asset with no recorded class defaults to `UserOwned` so that the cheap
//! mistake (refusing a write that was allowed) is the one a wrong default produces, never
//! the irrecoverable one (overwriting a person's own file on every machine at once).
//!
//! Mirrors `nomos-tool-package`'s own `Family::Of_Label` pattern exactly: a manual match
//! over the record's exact spellings, not a serde derive mapping this enum's Rust variant
//! names onto the wire form. The three names are transcribed from `OD-PACKAGE-004`'s own
//! table verbatim.

const GENERATED_OWNED_LABEL: &str = "GeneratedOwned";
const COMPOSED_LABEL: &str = "Composed";
const USER_OWNED_LABEL: &str = "UserOwned";

/// One of `OD-PACKAGE-004`'s three ownership classes.
///
/// What each one means for regeneration and for a conflict is that record's own table;
/// this crate carries the classification and nothing that acts on it, because acting on it
/// is the materializer's job (`OD-PACKAGE-003`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OwnershipClass
{
    /// The renderer is the sole author of every byte; regeneration overwrites
    /// unconditionally.
    GeneratedOwned,
    /// Regeneration is scoped to a region tied to a declared source of truth and never
    /// touches the rest.
    Composed,
    /// Nothing ever writes it. A mechanism unable to place a path in either other class
    /// treats it as this one and refuses.
    UserOwned,
}

impl OwnershipClass
{
    /// The class an intent that declares none resolves to.
    ///
    /// `OD-PACKAGE-004`: "An asset with no recorded class defaults to `UserOwned`, and the
    /// write is refused." The direction is fixed by the record's asymmetry argument, not by
    /// convenience -- a `GeneratedOwned` default would let the first installer bug overwrite
    /// a personal file with no diff to recover it from.
    pub const UNDECLARED: Self = Self::UserOwned;

    /// The label a manifest file spells this class with -- `OD-PACKAGE-004`'s own spelling.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::GeneratedOwned => GENERATED_OWNED_LABEL,
            Self::Composed => COMPOSED_LABEL,
            Self::UserOwned => USER_OWNED_LABEL,
        };
    }

    /// The class a label names, or `None` if it names none of the three.
    #[must_use]
    pub fn Of_Label(label: &str) -> Option<Self>
    {
        return match label
        {
            GENERATED_OWNED_LABEL => Some(Self::GeneratedOwned),
            COMPOSED_LABEL => Some(Self::Composed),
            USER_OWNED_LABEL => Some(Self::UserOwned),
            _ => None,
        };
    }
}

impl core::fmt::Display for OwnershipClass
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

    const EVERY_CLASS: [OwnershipClass; 3] = [OwnershipClass::GeneratedOwned, OwnershipClass::Composed, OwnershipClass::UserOwned];

    #[test]
    fn Test_Every_Class_Should_Round_Trip_Through_Its_Label()
    {
        for class in EVERY_CLASS
        {
            assert_eq!(OwnershipClass::Of_Label(class.Label()), Some(class));
        }
    }

    #[test]
    fn Test_A_Label_None_Of_The_Three_Name_Should_Resolve_To_Nothing()
    {
        assert_eq!(OwnershipClass::Of_Label("Owned"), None);
        assert_eq!(OwnershipClass::Of_Label("userowned"), None);
        assert_eq!(OwnershipClass::Of_Label(""), None);
    }

    #[test]
    fn Test_The_Undeclared_Class_Should_Be_User_Owned()
    {
        assert_eq!(OwnershipClass::UNDECLARED, OwnershipClass::UserOwned);
    }

    #[test]
    fn Test_Display_Should_Render_The_Label()
    {
        assert_eq!(OwnershipClass::GeneratedOwned.to_string(), "GeneratedOwned");
    }
}
