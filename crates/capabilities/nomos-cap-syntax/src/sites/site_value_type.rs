//! The four types a site's field may hold.

/// What a declared field holds, and so how its value is written.
///
/// Four and no more, because a kind is flat (`OD-CAPABILITY-019`, fourth decision): of the 815
/// fields the corpus's engines declare, 717 are already one of these, and a list of records
/// becomes a kind of its own whose records name the record they belong to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SiteValueType
{
    Text,
    Integer,
    Truth,
    TextList,
}

impl SiteValueType
{
    /// The type as a reader of a refusal names it.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Text => "text",
            Self::Integer => "an integer",
            Self::Truth => "a truth value",
            Self::TextList => "a list of text",
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Label_Should_Name_Every_Type_Differently()
    {
        let labels = [SiteValueType::Text, SiteValueType::Integer, SiteValueType::Truth, SiteValueType::TextList].map(SiteValueType::Label);

        let mut distinct = labels.to_vec();
        distinct.sort_unstable();
        distinct.dedup();
        assert_eq!(distinct.len(), labels.len(), "{labels:?}");
    }
}
