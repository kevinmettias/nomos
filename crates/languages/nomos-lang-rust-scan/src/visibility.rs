//! How widely a declaration is visible, taken textually.

/// How visible a declaration says it is.
///
/// There is no `NotApplicable`. A parser knows a trait member declares no visibility of its
/// own; a line-reader cannot see that it is inside a trait, so it would have to guess, and a
/// guess recorded as a fact is worse than a weaker vocabulary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Visibility
{
    Public,
    Restricted
    {
        scope: String,
    },
    Private,
}

impl Visibility
{
    #[must_use]
    pub fn Label(&self) -> String
    {
        return match self
        {
            Self::Public => "Public".to_owned(),
            Self::Restricted { scope } => format!("Restricted({scope})"),
            Self::Private => "Private".to_owned(),
        };
    }
}
