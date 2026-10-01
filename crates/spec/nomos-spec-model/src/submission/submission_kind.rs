//! Which of the three kinds `OD-SPEC-008` governs.

/// Which of the three kinds `OD-SPEC-008` governs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubmissionKind
{
    /// What somebody asked for, before anybody decided how or whether to answer it.
    FeatureRequest,
    /// The chosen answer to a request.
    DesignSpec,
    /// What was actually built against a design.
    FeatureResult,
}

impl SubmissionKind
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
        assert_eq!(SubmissionKind::FeatureRequest.Label(), "feature-request");
        assert_eq!(SubmissionKind::DesignSpec.Label(), "design-spec");
        assert_eq!(SubmissionKind::FeatureResult.Label(), "feature-result");
    }

    #[test]
    fn Test_Parse_Should_Round_Trip_Every_Stored_Spelling_And_Refuse_An_Unknown_One()
    {
        for kind in All_Kinds()
        {
            assert_eq!(SubmissionKind::Parse(kind.Label()), Some(kind));
        }
        assert_eq!(SubmissionKind::Parse("unknown"), None);
    }

    fn All_Kinds() -> [SubmissionKind; 3]
    {
        return [SubmissionKind::FeatureRequest, SubmissionKind::DesignSpec, SubmissionKind::FeatureResult];
    }

    #[test]
    fn Test_Required_Fields_Should_Differ_By_Kind()
    {
        assert_eq!(
            SubmissionKind::FeatureRequest.Required_Fields(),
            &["goal", "behaviour", "acceptance", "invariants"]
        );
        assert_eq!(
            SubmissionKind::FeatureResult.Required_Fields(),
            &["implements", "evidence", "deviations", "owed"]
        );
    }
}
