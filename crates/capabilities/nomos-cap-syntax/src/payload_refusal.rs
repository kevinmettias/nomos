//! Every way a payload refuses to be read.

/// Why a payload could not be read, and where.
///
/// A caller told only that something failed has been handed a number nobody can act on,
/// and this text reaches a finding somebody has to answer.
///
/// The line is the type's and not the kind's, because five of the seven refusals are
/// refusals *of a record at a line* and repeated it. It is optional rather than invented
/// for the other two: [`PayloadRefusalKind::NotUtf8`] and
/// [`PayloadRefusalKind::NoHeader`] are statements about the whole payload, and giving
/// them line 1 would claim a record was read there when none was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PayloadRefusal
{
    pub kind: PayloadRefusalKind,
    pub line: Option<usize>,
}

impl PayloadRefusal
{
    /// A refusal of the payload as a whole, which happened at no line.
    #[must_use]
    pub const fn Whole(kind: PayloadRefusalKind) -> Self
    {
        return Self { kind, line: None };
    }

    /// A refusal of the record at one line.
    #[must_use]
    pub const fn At(line: usize, kind: PayloadRefusalKind) -> Self
    {
        return Self {
            kind,
            line: Some(line),
        };
    }

    /// What went wrong, in terms somebody can act on.
    ///
    /// The line is prefixed here rather than repeated inside each kind's sentence, which
    /// is what keeps a kind from being able to describe itself without saying where.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self.line
        {
            Some(line) => format!("line {line} {}", self.kind.Describe()),
            None => self.kind.Describe(),
        };
    }
}

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
    UnreadableNumber
    {
        field: &'static str,
        value: String,
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
            Self::UnreadableNumber { field, value } =>
            {
                format!("carries `{value}` where `{field}` must be a number")
            }
            Self::UnreadableObservation { field, value } =>
            {
                format!("carries `{value}` where `{field}` must be `-`, `.` or `+` and a value")
            }
        };
    }
}
