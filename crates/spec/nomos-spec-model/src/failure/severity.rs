//! How badly an open decision gap bites.

/// How badly an open decision gap bites.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity
{
    /// Prevents acceptance.
    Blocking,
    /// Does not prevent acceptance, and survives into the design and the result.
    NonBlocking,
}

impl Severity
{
    /// The label this severity is stored under.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Blocking => "blocking",
            Self::NonBlocking => "non-blocking",
        };
    }

    /// The severity a label names, if it names one.
    #[must_use]
    pub fn Parse(label: &str) -> Option<Self>
    {
        return match label
        {
            "blocking" => Some(Self::Blocking),
            "non-blocking" => Some(Self::NonBlocking),
            _ => None,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Label_Should_Match_The_Stored_Spelling()
    {
        assert_eq!(Severity::Blocking.Label(), "blocking");
        assert_eq!(Severity::NonBlocking.Label(), "non-blocking");
    }

    /// Every severity there is, so a case cannot quietly go unchecked.
    fn All_Severities() -> [Severity; 2]
    {
        return [Severity::Blocking, Severity::NonBlocking];
    }

    #[test]
    fn Test_Parse_Should_Round_Trip_Every_Stored_Spelling_And_Refuse_An_Unknown_One()
    {
        for severity in All_Severities()
        {
            assert_eq!(Severity::Parse(severity.Label()), Some(severity));
        }
        assert_eq!(Severity::Parse("unknown"), None);
    }
}
