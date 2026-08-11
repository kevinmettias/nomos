//! Two restored members claiming one identifier.

/// Two members that would take one identifier.
///
/// Refused rather than merged. Merging would give one node two origins and make "which row
/// did this come from" unanswerable, which is the question the restoration is for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Collision
{
    pub id: String,
    pub first: String,
    pub second: String,
}

impl core::fmt::Display for Collision
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(
            formatter,
            "{} would identify both \"{}\" and \"{}\". Refusing to mint one node for two \
             members, because the second's origin would be unrecoverable",
            self.id, self.first, self.second
        );
    }
}
