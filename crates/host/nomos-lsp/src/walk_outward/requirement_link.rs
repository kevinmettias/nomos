//! Which corpus requirements a finding bears on, read off the assessments that declare its rule.
//!
//! `OD-HOST-015` decided the mechanism and refused two others. A finding does not reach a
//! requirement through the record its rule cites -- that join is empty over seventy of this
//! build's seventy-one rules and false over the remaining one, because a descriptor's `record`
//! is the authority a rule's implementation cites and an assessment's `record` is the reasoning
//! behind a verdict, and the two coincide by accident. Nor through its location: a `site` is
//! where a requirement is *satisfied* and a finding's location is where a rule *fired*.
//!
//! What it reaches through is a `rule` line an assessment declares, read by the reader
//! `nomos-cap-requirement-trace` already has. This module filters those declarations and
//! derives nothing.

use nomos_cap_requirement_trace::Assessment;
use nomos_contracts::RuleId;
use serde::Serialize;

/// One corpus requirement whose committed assessment declares this finding's rule.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RequirementLink
{
    /// The corpus requirement identifier -- `AGT-003`, `CHK-003` -- which is the assessment
    /// file's own stem and appears nowhere inside it.
    pub requirement: String,
    /// What that assessment says about the requirement, in `Verdict::Label`'s own word:
    /// `Met`, `Partial`, `Diverges` or `NotBinding`. Carried because a link into a `Diverges`
    /// entry and a link into a `Met` one are different news, and the entry already says which
    /// -- this is the assessment's word, never a verdict reached here.
    pub verdict: &'static str,
}

impl RequirementLink
{
    /// Every requirement whose assessment declares `rule`, in the order the registry reader
    /// returns them.
    ///
    /// **An empty result means nobody declared a `rule` line. It does not mean no rule bears
    /// on any requirement**, and nothing may read it that way. `OD-TRACE-001` chose that
    /// reading for this whole registry and `OD-HOST-015` kept it for the line: a `rule: none`
    /// would record that somebody looked and found nothing, which is a claim nobody would have
    /// checked. Fifty-nine of this build's seventy-one rules are ported code-standards rules
    /// bearing on no corpus requirement at all, and every repository but this one carries no
    /// registry to read -- so an empty list is the ordinary case and is not a report about
    /// anything.
    #[must_use]
    pub(crate) fn Declared_For(assessments: &[Assessment], rule: &RuleId) -> Vec<Self>
    {
        return assessments
            .iter()
            .filter(|assessment| return assessment.rules.contains(rule))
            .map(|assessment| {
                return Self { requirement: assessment.requirement.clone(), verdict: assessment.verdict.Label() };
            })
            .collect();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_cap_requirement_trace::{Site, Verdict};

    #[test]
    fn Test_Declared_For_Should_Find_The_Requirement_Whose_Entry_Names_The_Rule()
    {
        let assessments = [Declaring("AGT-003", Verdict::Partial, &[nomos_rules::DEPENDENCY_DIRECTION])];

        let links = RequirementLink::Declared_For(&assessments, &RuleId::New(nomos_rules::DEPENDENCY_DIRECTION));

        assert_eq!(links.len(), 1, "{links:?}");
        let only = links.first().expect("asserted one link above");
        assert_eq!(only.requirement, "AGT-003");
        assert_eq!(only.verdict, "Partial", "the entry's own verdict, not one reached here");
    }

    #[test]
    fn Test_Declared_For_Should_Find_Every_Entry_Naming_One_Rule()
    {
        let assessments = [
            Declaring("AGT-003", Verdict::Partial, &[nomos_rules::DEPENDENCY_DIRECTION]),
            Declaring("CHK-003", Verdict::Met, &[nomos_rules::DEPENDENCY_DIRECTION, nomos_rules::COMPLETENESS_MIRROR]),
        ];

        let links = RequirementLink::Declared_For(&assessments, &RuleId::New(nomos_rules::DEPENDENCY_DIRECTION));

        assert_eq!(links.len(), 2, "{links:?}");
        assert_eq!(links.first().expect("asserted two links above").verdict, "Partial");
        assert_eq!(links.get(1).expect("asserted two links above").verdict, "Met");
    }

    /// The absence this module is most able to answer wrongly: an entry that declared no rule
    /// at all must produce no link, and it must produce it for that reason rather than because
    /// its rule failed to match.
    #[test]
    fn Test_Declared_For_Should_Report_Nothing_For_An_Entry_That_Declared_No_Rule()
    {
        let assessments = [Declaring("EVID-001", Verdict::Met, &[])];

        let links = RequirementLink::Declared_For(&assessments, &RuleId::New(nomos_rules::DEPENDENCY_DIRECTION));

        assert!(links.is_empty(), "{links:?}");
    }

    #[test]
    fn Test_Declared_For_Should_Report_Nothing_For_A_Rule_No_Entry_Names()
    {
        let assessments = [Declaring("AGT-003", Verdict::Partial, &[nomos_rules::DEPENDENCY_DIRECTION])];

        let links = RequirementLink::Declared_For(&assessments, &RuleId::New(nomos_rules::NAMING_CONVENTION));

        assert!(links.is_empty(), "{links:?}");
    }

    /// An entry for `requirement` at `verdict` declaring exactly `rules`, carrying the site
    /// every entry owes and whatever else its own verdict owes beside it -- a record for a
    /// verdict somebody decided, a gap for `Partial`.
    ///
    /// None of those three is read by anything under test. They are here so each fixture is an
    /// entry `nomos_cap_requirement_trace::Parse_Assessment` would have accepted, rather than a
    /// shape only this module's own filter could ever see.
    fn Declaring(requirement: &str, verdict: Verdict, rules: &[&str]) -> Assessment
    {
        let record = verdict.Has_A_Record_Obligation().then(|| return "OD-HOST-015".to_owned());
        let gaps = match verdict
        {
            Verdict::Partial => vec![Somewhere("New")],
            Verdict::Met | Verdict::Diverges | Verdict::NotBinding => Vec::new(),
        };

        return Assessment {
            requirement: requirement.to_owned(),
            verdict,
            record,
            sites: vec![Somewhere("DESCRIPTORS")],
            gaps,
            rules: rules.iter().map(|rule| return RuleId::New(*rule)).collect(),
        };
    }

    /// A repo-relative `path#symbol` site naming `symbol`, which is the shape a site owes and
    /// the whole of what these fixtures need one for.
    fn Somewhere(symbol: &str) -> Site
    {
        return Site { path: "crates/rules/nomos-rules/src/lib.rs".to_owned(), symbol: symbol.to_owned() };
    }
}
