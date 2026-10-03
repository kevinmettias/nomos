//! Which way a sites payload refused to be read.

use super::SiteValueType;

/// Which refusal it was.
///
/// The three a kind's declaration exists to make possible are [`Self::UndeclaredKind`],
/// [`Self::UndeclaredField`] and [`Self::MissingField`]: without them a record naming a kind
/// nobody declared, or carrying a field its kind does not have, would be passed on to a rule that
/// then reads whatever it finds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SitesRefusalKind
{
    /// The bytes are not UTF-8, so they are not this schema.
    NotUtf8,
    /// A record tag this build does not understand.
    UnknownRecord
    {
        tag: String,
    },
    /// A stance with the wrong number of fields.
    WrongFieldCount
    {
        tag: String,
        expected: usize,
        found: usize,
    },
    /// A stance or a site naming a kind this family does not declare.
    UndeclaredKind
    {
        kind: String,
    },
    /// A second stance on one kind, which would leave two answers to one question.
    RepeatedStance
    {
        kind: String,
    },
    /// A decline that gives no reason.
    UnexplainedDecline
    {
        kind: String,
    },
    /// A site of a kind the payload does not offer: it declines the kind, or never names it.
    UnofferedSite
    {
        kind: String,
    },
    /// A site whose line is not a number.
    UnreadableLine
    {
        value: String,
        cause: String,
    },
    /// A site field that is not `name=value`.
    MalformedField
    {
        field: String,
    },
    /// A field the site's kind does not declare.
    UndeclaredField
    {
        kind: String,
        field: String,
    },
    /// A field the site's kind declares and the site does not carry.
    MissingField
    {
        kind: String,
        field: String,
    },
    /// A field carried twice by one site.
    RepeatedField
    {
        kind: String,
        field: String,
    },
    /// A value that is not the type its field declares.
    UnreadableValue
    {
        field: String,
        value: String,
        expected: SiteValueType,
    },
}

impl SitesRefusalKind
{
    /// What went wrong, as a clause a line prefix completes.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::NotUtf8 => "the payload is not UTF-8, so it is not this schema".to_owned(),
            Self::UnknownRecord { tag } => format!("carries the record tag `{tag}`, which this build does not understand"),
            Self::WrongFieldCount { tag, expected, found } => format!("is a `{tag}` record with {found} field(s) where this build expects {expected}"),
            Self::UndeclaredKind { kind } => format!("names the kind `{kind}`, which this family does not declare"),
            Self::RepeatedStance { kind } => format!("is a second stance on the kind `{kind}`, and a payload states one"),
            Self::UnexplainedDecline { kind } => format!("declines the kind `{kind}` without a reason, and a decline must say why"),
            Self::UnofferedSite { kind } => format!("is a site of the kind `{kind}`, which this payload does not offer"),
            Self::UnreadableLine { value, cause } => format!("carries `{value}` where a site's line must be a number: {cause}"),
            Self::MalformedField { field } => format!("carries the field `{field}`, which is not `name=value`"),
            Self::UndeclaredField { kind, field } => format!("carries the field `{field}`, which the kind `{kind}` does not declare"),
            Self::MissingField { kind, field } => format!("lacks the field `{field}`, which the kind `{kind}` declares"),
            Self::RepeatedField { kind, field } => format!("carries the field `{field}` of the kind `{kind}` twice"),
            Self::UnreadableValue { field, value, expected } => format!("carries `{value}` where `{field}` must be {}", expected.Label()),
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Describe_Should_Name_What_It_Refused()
    {
        let undeclared = SitesRefusalKind::UndeclaredField { kind: "labeled-jump".to_owned(), field: "depth".to_owned() };
        assert!(undeclared.Describe().contains("`depth`") && undeclared.Describe().contains("`labeled-jump`"));

        let unreadable = SitesRefusalKind::UnreadableValue { field: "label_line".to_owned(), value: "two".to_owned(), expected: SiteValueType::Integer };
        assert!(unreadable.Describe().contains("an integer"), "{}", unreadable.Describe());
    }
}
