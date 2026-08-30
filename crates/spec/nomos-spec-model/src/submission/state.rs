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
