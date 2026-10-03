//! One located construct.

use super::SiteValue;
use std::collections::BTreeMap;

/// One construct of a declared kind: the kind, the line it starts on, and its fields by name.
///
/// The fields are a map rather than a positional list so a record and its kind's declaration
/// agree by name: the writer puts them in the declaration's order, and the reader finds a field
/// the kind does not declare, or one it declares and the record lacks, by looking it up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SiteRecord
{
    pub kind: String,
    pub line: usize,
    pub values: BTreeMap<String, SiteValue>,
}

impl SiteRecord
{
    /// The text field named `name`, if the record carries one.
    #[must_use]
    pub fn Text(&self, name: &str) -> Option<&str>
    {
        return match self.values.get(name)
        {
            Some(SiteValue::Text(text)) => Some(text.as_str()),
            _ => None,
        };
    }

    /// The integer field named `name`, if the record carries one.
    #[must_use]
    pub fn Integer(&self, name: &str) -> Option<i64>
    {
        return match self.values.get(name)
        {
            Some(SiteValue::Integer(integer)) => Some(*integer),
            _ => None,
        };
    }

    /// The truth-value field named `name`, if the record carries one.
    #[must_use]
    pub fn Truth(&self, name: &str) -> Option<bool>
    {
        return match self.values.get(name)
        {
            Some(SiteValue::Truth(truth)) => Some(*truth),
            _ => None,
        };
    }

    /// The list-of-text field named `name`, if the record carries one.
    #[must_use]
    pub fn Text_List(&self, name: &str) -> Option<&[String]>
    {
        return match self.values.get(name)
        {
            Some(SiteValue::TextList(items)) => Some(items.as_slice()),
            _ => None,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Record() -> SiteRecord
    {
        let values = BTreeMap::from([
            ("words".to_owned(), SiteValue::Text("break".to_owned())),
            ("count".to_owned(), SiteValue::Integer(2)),
            ("flag".to_owned(), SiteValue::Truth(true)),
            ("scopes".to_owned(), SiteValue::TextList(vec!["a".to_owned()])),
        ]);
        return SiteRecord { kind: "test-kind".to_owned(), line: 1, values };
    }

    #[test]
    fn Test_Each_Accessor_Should_Read_Its_Own_Type_And_No_Other()
    {
        let record = Record();

        assert_eq!(record.Text("words"), Some("break"));
        assert_eq!(record.Integer("count"), Some(2));
        assert_eq!(record.Truth("flag"), Some(true));
        assert_eq!(record.Text_List("scopes"), Some(["a".to_owned()].as_slice()));
        assert_eq!(record.Text("count"), None);
        assert_eq!(record.Integer("flag"), None);
        assert_eq!(record.Truth("words"), None);
        assert_eq!(record.Text_List("missing"), None);
    }
}
