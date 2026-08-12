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
                let path = Path_As_Written(&restricted.path);
                let scope = if restricted.in_token.is_some()
                {
                    format!("in {path}")
                }
                else
                {
                    path
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
