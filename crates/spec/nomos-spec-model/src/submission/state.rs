//! Whether a submission has been accepted.

/// Whether a submission has been accepted.
///
/// Not a value with an origin, deliberately. Nobody types `draft` or `accepted`, and a
/// submission able to attribute its own state is a submission able to assert its own
/// acceptance — `OD-SPEC-013` keeps it a column for that reason.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum State
{
    /// A complete submission that has not been accepted.
    Draft,
    /// A draft that additionally satisfies rule 5, has no open blocking gap, and may be cited.
    Accepted,
}

impl State
{
    /// The label this state is stored under.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Draft => "draft",
            Self::Accepted => "accepted",
        };
    }

    /// The state a label names, if it names one.
    #[must_use]
    pub fn Parse(label: &str) -> Option<Self>
    {
        return match label
        {
            "draft" => Some(Self::Draft),
            "accepted" => Some(Self::Accepted),
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
        assert_eq!(State::Draft.Label(), "draft");
        assert_eq!(State::Accepted.Label(), "accepted");
    }

    #[test]
    fn Test_Parse_Should_Round_Trip_Every_Stored_Spelling_And_Refuse_An_Unknown_One()
    {
        for state in All_States()
        {
            assert_eq!(State::Parse(state.Label()), Some(state));
        }
        assert_eq!(State::Parse("unknown"), None);
    }

    fn All_States() -> [State; 2]
    {
        return [State::Draft, State::Accepted];
    }
}
