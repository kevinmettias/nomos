//! One field's value in a site record.

use super::SiteValueType;

/// A field's value, of one of the four types a kind may declare.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SiteValue
{
    Text(String),
    Integer(i64),
    Truth(bool),
    TextList(Vec<String>),
}

impl SiteValue
{
    /// The type this value is, which the reader compares against the field's declaration.
    #[must_use]
    pub const fn Type(&self) -> SiteValueType
    {
        return match self
        {
            Self::Text(_) => SiteValueType::Text,
            Self::Integer(_) => SiteValueType::Integer,
            Self::Truth(_) => SiteValueType::Truth,
            Self::TextList(_) => SiteValueType::TextList,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Type_Should_Name_The_Variant()
    {
        assert_eq!(SiteValue::Text(String::new()).Type(), SiteValueType::Text);
        assert_eq!(SiteValue::Integer(0).Type(), SiteValueType::Integer);
        assert_eq!(SiteValue::Truth(false).Type(), SiteValueType::Truth);
        assert_eq!(SiteValue::TextList(Vec::new()).Type(), SiteValueType::TextList);
    }
}
