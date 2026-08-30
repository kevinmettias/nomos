//! Which of the seven refusals it was.

/// Which of the seven refusals it was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PayloadRefusalKind
{
    /// The bytes are not UTF-8, so they are not this schema.
    NotUtf8,
    /// The first record is not an `unexpanded` header.
    ///
    /// Includes the empty payload. A fact carrying no bytes is not a file that declared
    /// nothing — that is a header and no items — and reading one as the other makes a
    /// subject that was never read indistinguishable from a subject with nothing in it.
    NoHeader,
    /// A second `unexpanded` header, which would leave two answers to one question.
    RepeatedHeader,
    /// A record tag this build does not understand.
    ///
    /// The likeliest cause is a payload from a newer schema, which is precisely the case
    /// where guessing is worst: the unknown record is where the missing information is.
    UnknownRecord
    {
        tag: String,
    },
    /// A record with the right tag and the wrong shape.
    WrongFieldCount
    {
        tag: String,
        expected: usize,
        found: usize,
    },
    /// A field that must be a number and is not.
    ///
    /// `cause` is the parser's own words for why. It is carried rather than dropped because
    /// "too large for the field" and "not digits at all" are two different defects in the
    /// writer, and a reader holding only the rejected text has to guess which one it met.
    UnreadableNumber
    {
        field: &'static str,
        value: String,
        cause: String,
    },
    /// A field that must be an observation and is not one of its three spellings.
    UnreadableObservation
    {
        field: &'static str,
        value: String,
    },
}

impl PayloadRefusalKind
{
    /// What went wrong, as a clause the line prefix completes.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::NotUtf8 => "the payload is not UTF-8, so it is not this schema".to_owned(),
            Self::NoHeader => "the payload does not begin with an `unexpanded` header, so it \
                               is either empty or not this schema"
                .to_owned(),
            Self::RepeatedHeader =>
            {
                "is a second `unexpanded` header, and a payload has one".to_owned()
            }
            Self::UnknownRecord { tag } => format!(
                "carries the record tag `{tag}`, which this build does not understand"
            ),
            Self::WrongFieldCount {
                tag,
                expected,
                found,
            } => format!("is a `{tag}` record with {found} field(s) where this build expects {expected}"),
            Self::UnreadableNumber {
                field,
                value,
                cause,
            } => format!("carries `{value}` where `{field}` must be a number: {cause}"),
            Self::UnreadableObservation { field, value } =>
            {
                format!("carries `{value}` where `{field}` must be `-`, `.` or `+` and a value")
            }
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Describe_Should_Name_What_Went_Wrong_For_Every_Kind()
    {
        assert_eq!(PayloadRefusalKind::NotUtf8.Describe(), "the payload is not UTF-8, so it is not this schema");

        let unknown = PayloadRefusalKind::UnknownRecord {
            tag: "region".to_owned(),
        };
        assert!(unknown.Describe().contains("region"));

        let wrong_count = PayloadRefusalKind::WrongFieldCount {
            tag: "item".to_owned(),
            expected: 7,
            found: 5,
        };
        assert!(wrong_count.Describe().contains("item"));
        assert!(wrong_count.Describe().contains('7'));
        assert!(wrong_count.Describe().contains('5'));
    }
}
