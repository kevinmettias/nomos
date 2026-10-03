//! Why a sites payload could not be read, and where.

use super::SitesRefusalKind;

/// A refusal and the line of the record it refused, when there was one -- the split
/// [`crate::PayloadRefusal`] draws for the items payload, for the same reason: a payload that is
/// not UTF-8 is refused as a whole, and giving that a line would claim a record was read there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SitesRefusal
{
    pub kind: SitesRefusalKind,
    pub line: Option<usize>,
}

impl SitesRefusal
{
    /// A refusal of the payload as a whole.
    #[must_use]
    pub const fn Whole(kind: SitesRefusalKind) -> Self
    {
        return Self { kind, line: None };
    }

    /// A refusal of the record at one line.
    #[must_use]
    pub const fn At(line: usize, kind: SitesRefusalKind) -> Self
    {
        return Self { kind, line: Some(line) };
    }

    /// What went wrong, in terms somebody can act on.
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

impl core::fmt::Display for SitesRefusal
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(&self.Describe());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The line the line-bound fixture refusal is attributed to.
    const RECORD_LINE: usize = 3;

    #[test]
    fn Test_Describe_Should_Prefix_A_Line_Bound_Refusal_And_Leave_A_Whole_One_Bare()
    {
        assert_eq!(SitesRefusal::Whole(SitesRefusalKind::NotUtf8).Describe(), SitesRefusalKind::NotUtf8.Describe());

        let at = SitesRefusal::At(RECORD_LINE, SitesRefusalKind::NotUtf8);
        assert_eq!(at.line, Some(RECORD_LINE));
        assert_eq!(at.to_string(), format!("line 3 {}", SitesRefusalKind::NotUtf8.Describe()));
    }
}
