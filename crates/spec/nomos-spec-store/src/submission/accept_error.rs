use crate::StoreError;
use nomos_spec_model::Refusal;

/// Why a submission did not become durable.
///
/// Two arms and not one, because they send the caller to different places. A [`Refusal`] is
/// a fact about the submission and the submitter can act on it; a [`StoreError`] is a fact
/// about the database and they cannot.
#[derive(Debug)]
pub enum AcceptError
{
    /// The submission failed the rule set. Nothing was written.
    Refused(Refusal),
    /// The store could not be used.
    Store(StoreError),
}

impl From<StoreError> for AcceptError
{
    fn from(error: StoreError) -> Self
    {
        return Self::Store(error);
    }
}

impl std::fmt::Display for AcceptError
{
    // `fmt` is the method name `std::fmt::Display` mandates; implementing the trait means
    // matching its signature exactly, so this is not a style choice available to rename.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        return match self
        {
            Self::Refused(refusal) => write!(formatter, "{refusal}"),
            Self::Store(error) => write!(formatter, "{error}"),
        };
    }
}
