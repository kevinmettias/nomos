//! Every way a payload refuses to be read.

/// Why a payload could not be read.
///
/// Every variant carries where. A caller told only that something failed has been handed a
/// number nobody can act on, and this text reaches a finding somebody has to answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PayloadRefusal
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
    RepeatedHeader
    {
        line: usize,
    },
    /// A record tag this build does not understand.
    ///
    /// The likeliest cause is a payload from a newer schema, which is precisely the case
    /// where guessing is worst: the unknown record is where the missing information is.
    UnknownRecord
    {
        tag: String,
        line: usize,
    },
    /// A record with the right tag and the wrong shape.
    WrongFieldCount
    {
        tag: String,
        expected: usize,
        found: usize,
        line: usize,
    },
    /// A field that must be a number and is not.
    UnreadableNumber
    {
        field: &'static str,
        value: String,
        line: usize,
    },
    /// A field that must be an observation and is not one of its three spellings.
    UnreadableObservation
    {
        field: &'static str,
        value: String,
        line: usize,
    },
}

impl PayloadRefusal
{
    /// What went wrong, in terms somebody can act on.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::NotUtf8 => "the payload is not UTF-8, so it is not this schema".to_owned(),
            Self::NoHeader => "the payload does not begin with an `unexpanded` header, so it \
                               is either empty or not this schema"
                .to_owned(),
            Self::RepeatedHeader { line } => {
                format!("line {line} is a second `unexpanded` header, and a payload has one")
            }
            Self::UnknownRecord { tag, line } => format!(
                "line {line} carries the record tag `{tag}`, which this build does not \
                 understand"
            ),
            Self::WrongFieldCount {
                tag,
                expected,
                found,
                line,
            } => format!(
                "line {line} is a `{tag}` record with {found} field(s) where this build \
                 expects {expected}"
            ),
            Self::UnreadableNumber { field, value, line } => {
                format!("line {line} carries `{value}` where `{field}` must be a number")
            }
            Self::UnreadableObservation { field, value, line } => format!(
                "line {line} carries `{value}` where `{field}` must be `-`, `.` or `+` and a \
                 value"
            ),
        };
    }
}
