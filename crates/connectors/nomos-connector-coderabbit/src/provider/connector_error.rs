//! Why this connector could not produce a fact: the vendor could not be reached, or its
//! answer could not be translated.

use crate::fetching::FetchError;
use crate::translation::TranslationError;

/// This connector could not produce a fact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectorError
{
    Fetch(FetchError),
    Translation(TranslationError),
}

impl core::fmt::Display for ConnectorError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Fetch(error) => write!(formatter, "{error}"),
            Self::Translation(error) => write!(formatter, "{error}"),
        };
    }
}

impl From<FetchError> for ConnectorError
{
    fn from(error: FetchError) -> Self
    {
        return Self::Fetch(error);
    }
}

impl From<TranslationError> for ConnectorError
{
    fn from(error: TranslationError) -> Self
    {
        return Self::Translation(error);
    }
}
