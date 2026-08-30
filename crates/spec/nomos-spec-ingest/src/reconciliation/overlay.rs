pub(crate) mod artifact;
pub(crate) mod identifiers;
pub(crate) mod report;
pub(crate) mod overlaid;
pub(crate) mod pair_change;
pub(crate) mod reconcile;

/// Prose that says a section exists without saying what it says.
///
/// The first two are v14's own blocklist, ported from `validate_bundle.py`. They match
/// nothing in v15.0 — v15's filler is differently worded — and are kept because the
/// recorded blocklist is what the corpus declared, not what one revision happened to
/// need. The rest were measured in v15.0 and account for 405 blocks across 99 files.
pub const FILLER_PATTERNS: &[&str] = &[
    "This section groups related specification material",
    "Detailed statements appear below with stable IDs and graph and projection tooling",
    "This section preserves the reference or explanatory material for",
    "See the normative content above; this schema heading makes the retained architecture record",
    "The requirements below are the normative detail; the owning domain source supplies",
    "This decision preserves the product/domain boundary and behavior described by the owning",
    "Alternatives include retaining the preceding architecture, duplicating the mechanism locally",
    "The accepted decision is the normative direction stated in this record",
    "Implementations and projections shall conform to the accepted decision and expose incompatibility",
];

/// The pattern a block matches, or `None`.
///
/// Returns which pattern rather than a bool, so a lineage row can name why a block was
/// judged filler instead of asserting it.
#[must_use]
pub fn Get_Filler_Pattern(text: &str) -> Option<&'static str>
{
    return FILLER_PATTERNS
        .iter()
        .find(|pattern| text.contains(**pattern))
        .copied();
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Filler_Should_Name_The_Pattern_That_Judged_It()
    {
        let judged = Get_Filler_Pattern(
            "This section preserves the reference or explanatory material for Contents.",
        );

        assert_eq!(
            judged,
            Some("This section preserves the reference or explanatory material for")
        );
        assert_eq!(Get_Filler_Pattern("Nomos uses a small identity kernel."), None);
    }
}
