//! Which of the three kinds `OD-SPEC-008` governs.

/// Which of the three kinds `OD-SPEC-008` governs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind
{
    /// What somebody asked for, before anybody decided how or whether to answer it.
    FeatureRequest,
    /// The chosen answer to a request.
    DesignSpec,
    /// What was actually built against a design.
    FeatureResult,
}

impl Kind
{
    /// The label this kind is stored and cited under.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::FeatureRequest => "feature-request",
            Self::DesignSpec => "design-spec",
            Self::FeatureResult => "feature-result",
        };
    }

    /// The kind a label names, if it names one.
    #[must_use]
    pub fn Parse(label: &str) -> Option<Self>
    {
        return match label
        {
            "feature-request" => Some(Self::FeatureRequest),
            "design-spec" => Some(Self::DesignSpec),
            "feature-result" => Some(Self::FeatureResult),
            _ => None,
        };
    }

    /// The fields this kind must carry, beyond the universal ones.
    ///
    /// `OD-SPEC-010`'s per-kind tables, which are its requiredness test applied rather than
    /// an independent authority. A field added here argues from that test.
    #[must_use]
    pub const fn Required_Fields(self) -> &'static [&'static str]
    {
        return match self
        {
            Self::FeatureRequest => &["goal", "behaviour", "acceptance", "invariants"],
            Self::DesignSpec =>
            {
                &["answers", "alternatives", "selected", "architecture_delta", "acceptance"]
            }
            Self::FeatureResult => &["implements", "evidence", "deviations", "owed"],
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
        assert_eq!(Kind::FeatureRequest.Label(), "feature-request");
        assert_eq!(Kind::DesignSpec.Label(), "design-spec");
        assert_eq!(Kind::FeatureResult.Label(), "feature-result");
    }

    #[test]
    fn Test_Parse_Should_Round_Trip_Every_Stored_Spelling_And_Refuse_An_Unknown_One()
    {
        for kind in All_Kinds()
        {
            assert_eq!(Kind::Parse(kind.Label()), Some(kind));
        }
        assert_eq!(Kind::Parse("unknown"), None);
    }

    fn All_Kinds() -> [Kind; 3]
    {
        return [Kind::FeatureRequest, Kind::DesignSpec, Kind::FeatureResult];
    }

    #[test]
    fn Test_Required_Fields_Should_Differ_By_Kind()
    {
        assert_eq!(
            Kind::FeatureRequest.Required_Fields(),
            &["goal", "behaviour", "acceptance", "invariants"]
        );
        assert_eq!(
            Kind::FeatureResult.Required_Fields(),
            &["implements", "evidence", "deviations", "owed"]
        );
    }
}
