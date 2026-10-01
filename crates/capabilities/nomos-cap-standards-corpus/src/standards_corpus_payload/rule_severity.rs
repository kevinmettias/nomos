//! The severity a standards document declares for itself.

/// The severity a standards document declares for itself, in the corpus's own vocabulary.
///
/// The five variants are `rule.schema.json`'s severity enum, and the spelling [`Self::Label`]
/// returns is the spelling the document declares -- `MUST NOT` with its space, not a
/// normalized identifier. That is the whole point of the type: the census this capability is
/// measured against counts the declared words, so a round trip that rewrote them could not
/// be checked against it.
///
/// [`Self::MAY`] is a severity rather than an absence of one. A document declaring `MAY`
/// states something about what a repository is permitted to do, and dropping it from the
/// population because it cannot fail a build would be the flattening this capability exists
/// to avoid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuleSeverity
{
    /// The document declares `MUST`.
    Must,
    /// The document declares `MUST NOT`.
    MustNot,
    /// The document declares `SHOULD`.
    Should,
    /// The document declares `SHOULD NOT`.
    ShouldNot,
    /// The document declares `MAY`.
    May,
}

impl RuleSeverity
{
    /// The severity as the document declared it.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Must => "MUST",
            Self::MustNot => "MUST NOT",
            Self::Should => "SHOULD",
            Self::ShouldNot => "SHOULD NOT",
            Self::May => "MAY",
        };
    }

    /// The severity a document declares by writing `declared`, or `None` where the corpus's
    /// own vocabulary has no such severity.
    ///
    /// `None` is a refusal rather than a default: a document declaring a severity this
    /// workspace does not know is a declaration that did not parse, and the reader reports it
    /// as an issue rather than filing it under a severity it never wrote.
    #[must_use]
    pub fn Declared_By(declared: &str) -> Option<Self>
    {
        return match declared.trim()
        {
            "MUST" => Some(Self::Must),
            "MUST NOT" => Some(Self::MustNot),
            "SHOULD" => Some(Self::Should),
            "SHOULD NOT" => Some(Self::ShouldNot),
            "MAY" => Some(Self::May),
            _ => None,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// Every variant, so a test that walks the vocabulary cannot miss one added later.
    const EVERY: [RuleSeverity; 5] = [
        RuleSeverity::Must,
        RuleSeverity::MustNot,
        RuleSeverity::Should,
        RuleSeverity::ShouldNot,
        RuleSeverity::May,
    ];

    #[test]
    fn Test_Every_Severity_Should_Round_Trip_Through_Its_Own_Label()
    {
        for severity in EVERY
        {
            assert_eq!(RuleSeverity::Declared_By(severity.Label()), Some(severity), "{severity:?}");
        }
    }

    #[test]
    fn Test_A_Declared_Severity_Should_Keep_The_Spelling_The_Document_Wrote()
    {
        assert_eq!(RuleSeverity::MustNot.Label(), "MUST NOT", "the census counts the declared words");
    }

    #[test]
    fn Test_An_Unknown_Severity_Should_Be_Refused_Rather_Than_Defaulted()
    {
        assert_eq!(RuleSeverity::Declared_By("SHALL"), None);
        assert_eq!(RuleSeverity::Declared_By("must"), None, "the vocabulary is the corpus's, not a case-folded one");
    }
}
