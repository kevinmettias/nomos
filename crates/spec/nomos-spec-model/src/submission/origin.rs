//! Where a value came from.

/// Where a value came from.
///
/// Not interchangeable, and `OD-SPEC-010` turns that into a rule rather than a convention:
/// an `Inferred` value is readable and never sufficient for acceptance. A guess written down
/// as a guess is worth having; a guess that can satisfy acceptance is the system accepting
/// its own inferences as what somebody wanted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origin
{
    /// Typed by the submitter.
    Submitted,
    /// Supplied by the submitter later, on being asked.
    Clarified,
    /// Supplied by machinery — a CLI default, a form's pre-populated field, an agent's guess.
    Inferred,
    /// Closed by a governing record or a recorded decision.
    Decided,
}

impl Origin
{
    /// The label this origin is stored under.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Submitted => "submitted",
            Self::Clarified => "clarified",
            Self::Inferred => "inferred",
            Self::Decided => "decided",
        };
    }

    /// The origin a label names, if it names one.
    #[must_use]
    pub fn Parse(label: &str) -> Option<Self>
    {
        return match label
        {
            "submitted" => Some(Self::Submitted),
            "clarified" => Some(Self::Clarified),
            "inferred" => Some(Self::Inferred),
            "decided" => Some(Self::Decided),
            _ => None,
        };
    }

    /// Whether a value of this origin can satisfy acceptance.
    ///
    /// `OD-SPEC-010`: a submission is accepted only if every required field's current value
    /// has origin `submitted`, `clarified` or `decided`. `Decided` qualifies because a
    /// decision is answerable to something; an inference is not.
    #[must_use]
    pub const fn Can_Satisfy_Acceptance(self) -> bool
    {
        return !matches!(self, Self::Inferred);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Label_Should_Match_The_Stored_Spelling()
    {
        assert_eq!(Origin::Submitted.Label(), "submitted");
        assert_eq!(Origin::Clarified.Label(), "clarified");
        assert_eq!(Origin::Inferred.Label(), "inferred");
        assert_eq!(Origin::Decided.Label(), "decided");
    }

    #[test]
    fn Test_Parse_Should_Round_Trip_Every_Stored_Spelling_And_Refuse_An_Unknown_One()
    {
        for origin in All_Origins()
        {
            assert_eq!(Origin::Parse(origin.Label()), Some(origin));
        }
        assert_eq!(Origin::Parse("unknown"), None);
    }

    #[test]
    fn Test_Can_Satisfy_Acceptance_Should_Be_False_Only_For_An_Inferred_Value()
    {
        for origin in All_Origins()
        {
            assert_eq!(origin.Can_Satisfy_Acceptance(), origin != Origin::Inferred);
        }
    }

    fn All_Origins() -> [Origin; 4]
    {
        return [Origin::Submitted, Origin::Clarified, Origin::Inferred, Origin::Decided];
    }
}
