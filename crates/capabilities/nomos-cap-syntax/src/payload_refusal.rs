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

#[cfg(test)]
mod tests
{
    use super::*;

    /// The line each refusal below is attributed to, so the tests agree on one record's line.
    const RECORD_LINE: usize = 4;

    #[test]
    fn Test_Whole_Should_Refuse_With_No_Line()
    {
        let refusal = PayloadRefusal::Whole(PayloadRefusalKind::NotUtf8);

        assert_eq!(refusal.line, None);
        assert_eq!(refusal.kind, PayloadRefusalKind::NotUtf8);
    }

    #[test]
    fn Test_At_Should_Refuse_The_Record_At_That_Line()
    {
        let refusal = PayloadRefusal::At(RECORD_LINE, PayloadRefusalKind::RepeatedHeader);

        assert_eq!(refusal.line, Some(RECORD_LINE));
        assert_eq!(refusal.kind, PayloadRefusalKind::RepeatedHeader);
    }

    #[test]
    fn Test_Describe_Should_Prefix_A_Line_Bound_Refusal_And_Leave_A_Whole_One_Bare()
    {
        let whole = PayloadRefusal::Whole(PayloadRefusalKind::NotUtf8);
        assert_eq!(whole.Describe(), PayloadRefusalKind::NotUtf8.Describe());

        let at_line = PayloadRefusal::At(RECORD_LINE, PayloadRefusalKind::RepeatedHeader);
        assert_eq!(
            at_line.Describe(),
            format!("line 4 {}", PayloadRefusalKind::RepeatedHeader.Describe())
        );
    }
}
