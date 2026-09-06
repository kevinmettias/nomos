//! Which rule produced a finding, and what `nomos_rules::DESCRIPTORS` already says it reads.

use nomos_contracts::RuleId;
use nomos_rules::{RequiredFact, SubjectKind, DESCRIPTORS};
use serde::Serialize;

/// The rule that governs one finding, as its own descriptor already states it -- nothing
/// computed here that `nomos_rules::DESCRIPTORS` did not already declare.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GoverningRule
{
    /// The rule identifier, the same string `Finding::rule` carries.
    pub rule: String,
    /// What this rule reads: the text of each walked source
    /// (`nomos_rules::SubjectKind::SourceText`), facts filed under the source's own subject
    /// (`SourceFacts`), or one whole-workspace fact (`Workspace`).
    pub subject_kind: &'static str,
    /// The capability contracts this rule's own `RuleDescriptor::requires` names, as their
    /// real wire identifiers (`nomos.cap.syntax.items`, and so on) -- never invented labels.
    pub required_facts: Vec<String>,
    /// The governing record this rule's contract cites, or `nomos_rules::PORTED_STANDARD` /
    /// `WORKSPACE_CONVENTIONS` for a rule with no versioned record. Carried through exactly
    /// as `RuleDescriptor::contract_record` names it -- not the same population as a
    /// corpus requirement id (`AGT-007`, `CHK-003`); see this crate's own module doc for why
    /// the two are not conflated here.
    pub contract_record: &'static str,
    /// `RuleDescriptor::contract_record_version`, or `nomos_rules::NO_VERSIONED_RECORD`
    /// when `contract_record` is prose rather than a versioned record.
    pub contract_record_version: u32,
}

impl GoverningRule
{
    /// The governing rule for `rule`, read from `nomos_rules::DESCRIPTORS` -- `None` when
    /// `rule` names a rule this build's `nomos-rules` never described, which the caller
    /// treats as "no walk-outward data for this rule" rather than an error: a `Finding`
    /// and the descriptor table it is checked against can only disagree if one build's
    /// findings are read against a different build's rules, which is not this crate's
    /// failure to explain.
    #[must_use]
    pub(crate) fn Of(rule: &RuleId) -> Option<Self>
    {
        let descriptor = DESCRIPTORS.iter().find(|descriptor| return descriptor.Rule() == *rule)?;

        return Some(Self {
            rule: rule.As_Str().to_owned(),
            subject_kind: Subject_Kind_Label(descriptor.subject),
            required_facts: descriptor.requires.iter().map(|fact| return Required_Fact_Label(*fact)).collect(),
            contract_record: descriptor.contract_record,
            contract_record_version: descriptor.contract_record_version,
        });
    }
}

/// `kind`'s own name, since [`SubjectKind`] carries no `Display` of its own.
fn Subject_Kind_Label(kind: SubjectKind) -> &'static str
{
    return match kind
    {
        SubjectKind::SourceText => "SourceText",
        SubjectKind::SourceFacts => "SourceFacts",
        SubjectKind::Workspace => "Workspace",
    };
}

/// `fact`'s real capability identifier -- the same string a `Reader::Require` call would
/// resolve against, not a label invented for this crate alone.
fn Required_Fact_Label(fact: RequiredFact) -> String
{
    return fact.Capability().As_Str().to_owned();
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Of_Should_Find_A_Real_Composed_Rule()
    {
        let rule = RuleId::New(nomos_rules::COMPLETENESS_MIRROR);

        let governing = GoverningRule::Of(&rule).expect("completeness-mirror is a real descriptor");

        let syntax_capability = RequiredFact::SyntaxItems.Capability().As_Str().to_owned();

        assert_eq!(governing.rule, nomos_rules::COMPLETENESS_MIRROR);
        assert_eq!(governing.subject_kind, "SourceFacts");
        assert!(governing.required_facts.contains(&syntax_capability), "{:?}", governing.required_facts);
    }

    #[test]
    fn Test_Of_Should_Be_None_For_An_Unknown_Rule()
    {
        let rule = RuleId::New("nomos-lsp-test-does-not-exist");

        assert!(GoverningRule::Of(&rule).is_none());
    }

    #[test]
    fn Test_Of_Should_Report_No_Required_Facts_For_A_Source_Text_Rule()
    {
        let rule = RuleId::New(nomos_rules::NO_TRAILING_WHITESPACE);

        let governing = GoverningRule::Of(&rule).expect("no-trailing-whitespace is a real descriptor");

        assert_eq!(governing.subject_kind, "SourceText");
        assert!(governing.required_facts.is_empty(), "{:?}", governing.required_facts);
    }
}
