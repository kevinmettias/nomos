//! How widely a declaration is visible, as written.

/// The visibility an item declares.
///
/// Four values, not a `bool`. A trait method and a private function are both "not
/// public" and they are not the same fact: one has no visibility to declare, and
/// recording it as `Private` would be this provider inventing a declaration the source
/// does not contain. [`Visibility::NotApplicable`] is what a sound provider says there.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Visibility
{
    /// `pub`.
    Public,
    /// `pub(crate)`, `pub(super)`, `pub(in path)` — with the scope as written.
    Restricted
    {
        scope: String,
    },
    /// No visibility keyword, on an item form that permits one.
    Private,
    /// An item form that declares no visibility: an `impl` block, a trait member.
    NotApplicable,
}

impl Visibility
{
    /// The stable label used in an encoded payload.
    #[must_use]
    pub fn Label(&self) -> String
    {
        return match self
        {
            Self::Public => "Public".to_owned(),
            Self::Restricted { scope } => format!("Restricted({scope})"),
            Self::Private => "Private".to_owned(),
            Self::NotApplicable => "NotApplicable".to_owned(),
        };
    }

    pub(crate) fn Of(visibility: &syn::Visibility) -> Self
    {
        use crate::syntax::Path_As_Written;

        return match visibility
        {
            syn::Visibility::Public(_) => Self::Public,
            syn::Visibility::Inherited => Self::Private,
            syn::Visibility::Restricted(restricted) =>
            {
                let scope = if restricted.in_token.is_some()
                {
                    format!("in {}", Path_As_Written(&restricted.path))
                }
                else
                {
                    Path_As_Written(&restricted.path)
                };

                Self::Restricted { scope }
            }
        };
    }
}

impl core::fmt::Display for Visibility
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(&self.Label());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Label_Should_Match_The_Shared_Vocabulary()
    {
        assert_eq!(Visibility::Public.Label(), "Public");
        assert_eq!(Visibility::Private.Label(), "Private");
        assert_eq!(Visibility::NotApplicable.Label(), "NotApplicable");
        assert_eq!(Visibility::Restricted { scope: "crate".to_owned() }.Label(), "Restricted(crate)");
    }

    #[test]
    fn Test_Of_Should_Read_A_Restricted_Visibilitys_Scope_As_Written()
    {
        let restricted: syn::Visibility = syn::parse_str("pub(crate)").expect("a valid visibility fixture parses");

        assert_eq!(Visibility::Of(&restricted), Visibility::Restricted { scope: "crate".to_owned() });
    }

    #[test]
    fn Test_Of_Should_Read_No_Keyword_As_Private()
    {
        let inherited: syn::Visibility = syn::Visibility::Inherited;

        assert_eq!(Visibility::Of(&inherited), Visibility::Private);
    }
}
