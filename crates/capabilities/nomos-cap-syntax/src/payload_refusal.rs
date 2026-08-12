//! Every way a payload refuses to be read.

use crate::PayloadRefusalKind;

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
