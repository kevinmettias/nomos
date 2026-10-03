//! `standards-corpus` — the one rule that reads `nomos.cap.standards.corpus`.
//!
//! # A third rule whose subject is not source
//!
//! [`crate::Check_Goals_And_Parts_Line_Up`] and
//! [`crate::Check_Requirement_Trace_Staleness`] are the only other rules in this crate handed
//! nothing but a [`FactReader`], and their own module docs explain why that is not an
//! oversight against `OD-RULES-001`. This rule's subject is the identical shape. A standards
//! corpus is a tree of markdown documents on disk, its declaration lives in a file of the
//! repository's own, and reading it needs a [`nomos_platform::FileSystem`] this crate (Rules
//! zone) may not depend on. So the whole of the acquisition already happened by the time this
//! rule runs, inside `nomos-repo-policy`'s reader, and this rule's job is the
//! [`crate::Check_Lint_Diagnostics`]/[`crate::Check_Review_Findings`] shape: relay an
//! already-reached population as `Finding`s rather than read a second time.
//!
//! # What it reports, and why a declaration is not a violation
//!
//! One `Finding` per document declaring `kind: rule`, carrying what the document declares
//! about itself — its severity, its gate and its mechanical owners — and one per document
//! whose declaration could not be read. Neither is a defect claim. What this rule surfaces is
//! a *population*: the repository's own rules, made visible to the report, which is what
//! makes the corpus judgeable at all (`ARC-CONFORMANCE-003`).
//!
//! The gate a finding carries is the one the document declared, verbatim, and
//! [`Applicability`] is derived from it the same way: a rule whose document claims no
//! mechanical enforcer is [`Applicability::AgentRequired`], because reaching a judgment for it
//! needs a model, and every other declared gate is [`Applicability::Supported`], because the
//! document claims a machine can decide it. Reporting a declared gate as anything but what the
//! document wrote would be this rule supplying a declaration nobody authored — the flattening
//! the capability's own ceiling was written to refuse.
//!
//! A document whose declaration could not be read is [`Applicability::Unparseable`] at
//! [`GateCategory::Review`], since nothing downstream of a declaration that does not parse can
//! be attempted and the remedy is a person's.
//!
//! # What "nothing to report" means here
//!
//! An absent fact, or a repository declaring no corpus, judges nothing. That is the same
//! answer this crate's two other workspace rules give, and the same one the capability's
//! summary states: a repository that declares no corpus judges exactly as it did before this
//! rule existed.

use nomos_analysis::{FactReader, InputDigest};
use nomos_cap_standards_corpus::{DeclarationIssue, DeclaredRule, StandardsCorpusPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// This rule's own identifier.
pub const STANDARDS_CORPUS: &str = "standards-corpus";

/// The record this implementation's contract is written against. `ARC-CONFORMANCE-003`
/// decided that nomos is the ecosystem's shared rule layer and that a corpus is absorbed by
/// reading its declarations rather than by copying its rules; this rule is that decision's
/// enforcement surface, and `tests/contract/tests/rule_contract_citation.rs` reads that
/// record's own front matter on every run and compares it against
/// [`STANDARDS_CORPUS_CONTRACT_RECORD_VERSION`], so an amendment this implementation has not
/// caught up to is a red test rather than silent drift.
pub const STANDARDS_CORPUS_CONTRACT_RECORD: &str = "ARC-CONFORMANCE-003";

/// The version of [`STANDARDS_CORPUS_CONTRACT_RECORD`] this implementation was written
/// against.
///
/// Version 2 re-stated the record's second milestone and corrected two document counts to
/// the rule counts they were; nothing this rule reports moved with it, so the citation
/// follows the record rather than the implementation following a change. Version 3
/// re-stated the third milestone around `OD-CAPABILITY-019` and withdrew the conclusion that
/// the corpus's dominant port is a declared text rule, and that moved nothing here either.
pub const STANDARDS_CORPUS_CONTRACT_RECORD_VERSION: u32 = 3;

/// Reports every document in a repository's declared standards corpus: what each one declares
/// about itself, and which declarations could not be read — one `Finding` per document, never
/// collapsed, since two rules in one corpus are two separate facts a reader needs to see both
/// of.
///
/// Judges nothing when the fact is unavailable or the population is empty. Both are the same
/// answer and neither is itself a `Finding` — see this module's own doc.
#[must_use]
pub fn Check_Standards_Corpus(facts: &mut dyn FactReader) -> Vec<Finding>
{
    let Some(payload) = Declared_Corpus(facts)
    else
    {
        return Vec::new();
    };

    return Findings_For(&payload);
}

fn Declared_Corpus(facts: &mut dyn FactReader) -> Option<StandardsCorpusPayload>
{
    let subject = nomos_model::Subject_Of_Path("");
    let fact = facts
        .Require(&nomos_cap_standards_corpus::Capability(), &subject, InputDigest::Of(&[]), &Standards_Corpus_Requirement())
        .ok()?;

    return nomos_cap_standards_corpus::Parse_Payload(&fact.payload.bytes).ok();
}

/// This crate's own floor for `nomos.cap.standards.corpus` — stated at the capability's own
/// ceiling since there is only one real provider today and no weaker answer this rule could
/// honestly still act on.
fn Standards_Corpus_Requirement() -> nomos_capability::Requirement
{
    return nomos_capability::Requirement::New(
        nomos_cap_standards_corpus::Capability(),
        nomos_cap_standards_corpus::CONTRACT_VERSION,
        nomos_cap_standards_corpus::Ceiling(),
    );
}

/// The audit itself, over a payload already in hand.
///
/// Sorted by `subject_name`, the path each finding is about, so a run's report does not depend
/// on the order a provider happened to walk a directory tree in — the same reason, and the
/// same key, the source-judging rules in this crate share.
fn Findings_For(payload: &StandardsCorpusPayload) -> Vec<Finding>
{
    let mut findings: Vec<Finding> = payload.rules.iter().map(Finding_For_Rule).collect();
    findings.extend(payload.issues.iter().map(Finding_For_Issue));

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Finding_For_Rule(rule: &DeclaredRule) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(STANDARDS_CORPUS),
        subject: nomos_model::Subject_Of_Path(&rule.path),
        subject_name: rule.path.clone(),
        applicability: Applicability_For(rule.gate),
        evidence: EvidenceClass::Derived,
        // The declared gate, verbatim. A document that wrote `review` has said no machine can
        // decide it, and this rule reporting `Blocking` would be reporting a claim nobody made.
        gate: rule.gate,
        summary: format!(
            "{} declares {}: {} at gate {} enforced by {}",
            rule.path,
            rule.id,
            rule.severity.Label(),
            rule.gate.Label(),
            rule.enforced_by.join(", ")
        ),
        locations: vec![rule.path.clone()],
    };
}

/// The state a declared rule's own gate puts a reader in: a document claiming no mechanical
/// enforcer needs a model, and every other declaration claims a machine can decide it.
const fn Applicability_For(gate: GateCategory) -> Applicability
{
    return match gate
    {
        GateCategory::Review => Applicability::AgentRequired,
        GateCategory::Unreachable | GateCategory::Advisory | GateCategory::Blocking => Applicability::Supported,
    };
}

fn Finding_For_Issue(issue: &DeclarationIssue) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(STANDARDS_CORPUS),
        subject: nomos_model::Subject_Of_Path(&issue.path),
        subject_name: issue.path.clone(),
        // A declaration that will not parse stops everything downstream of it, and the remedy
        // is a person editing the document rather than a provider being installed.
        applicability: Applicability::Unparseable,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Review,
        summary: format!("{}: {}", issue.path, issue.reason),
        locations: vec![issue.path.clone()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::checks::test_support::{self, FactToFile, OfferedProvider, Test_Context, TestOffering};
    use nomos_analysis::{MemoryFactStore, Reader};
    use nomos_cap_standards_corpus::{DeclarationKind, DocumentDeclaration, Encode_Payload, Payload_Schema, RuleSeverity};

    const PROVIDER: &str = "nomos.test.standards.corpus.reads";

    #[test]
    fn Test_Check_Standards_Corpus_Should_Judge_Nothing_When_The_Fact_Is_Unavailable()
    {
        let store = MemoryFactStore::New();
        let registry = nomos_capability::Registry::New();
        let mut reader = Reader::On(&store, &registry, Test_Context());

        let findings = Check_Standards_Corpus(&mut reader);

        assert!(findings.is_empty(), "an unmaterialized fact must judge nothing, not report an absence: {findings:?}");
    }

    #[test]
    fn Test_Check_Standards_Corpus_Should_Judge_Nothing_Over_An_Empty_Population()
    {
        let findings = Findings_For(&StandardsCorpusPayload::default());

        assert!(findings.is_empty(), "a repository declaring no corpus must not manufacture a finding: {findings:?}");
    }

    #[test]
    fn Test_Check_Standards_Corpus_Should_Report_One_Finding_Per_Declared_Rule()
    {
        let payload = StandardsCorpusPayload {
            roots: vec!["docs/standards".to_owned()],
            documents: vec![
                Document("docs/standards/naming.md", DeclarationKind::Rule),
                Document("docs/standards/notes.md", DeclarationKind::Undeclared),
            ],
            rules: vec![A_Rule("docs/standards/naming.md", "naming", RuleSeverity::MustNot, GateCategory::Blocking)],
            issues: Vec::new(),
        };

        let findings = Findings_For(&payload);

        assert_eq!(findings.len(), 1, "a document that declared nothing is not a rule: {findings:?}");
        let first = findings.first().expect("asserted a length of one above");
        assert_eq!(first.rule, RuleId::New(STANDARDS_CORPUS));
        assert_eq!(first.evidence, EvidenceClass::Derived);
        assert_eq!(first.subject_name, "docs/standards/naming.md");
        assert_eq!(first.locations, vec!["docs/standards/naming.md".to_owned()]);
    }

    /// The declared gate travels through verbatim in both directions: `gate` is what the
    /// document wrote, and `applicability` is derived from it, so a rule declaring that a
    /// machine decides it is never reported as needing a model and one declaring `review` is
    /// never reported as mechanically decidable.
    #[test]
    fn Test_Check_Standards_Corpus_Should_Carry_Each_Declared_Gate_Verbatim()
    {
        let payload = StandardsCorpusPayload {
            rules: vec![
                A_Rule("docs/standards/a.md", "a", RuleSeverity::MustNot, GateCategory::Blocking),
                A_Rule("docs/standards/b.md", "b", RuleSeverity::Should, GateCategory::Review),
                A_Rule("docs/standards/c.md", "c", RuleSeverity::May, GateCategory::Advisory),
            ],
            ..StandardsCorpusPayload::default()
        };

        let findings = Outcomes(Findings_For(&payload));

        assert_eq!(
            findings,
            vec![
                ("docs/standards/a.md".to_owned(), GateCategory::Blocking, Applicability::Supported),
                ("docs/standards/b.md".to_owned(), GateCategory::Review, Applicability::AgentRequired),
                ("docs/standards/c.md".to_owned(), GateCategory::Advisory, Applicability::Supported),
            ],
            "a declared gate must arrive as declared, with the state it implies"
        );
    }

    /// A document whose declaration cannot be read is a finding of its own, and a different one
    /// from an absence: it is reported at `Unparseable`, and the document is *also* a row in
    /// `documents` — the two are not alternatives.
    #[test]
    fn Test_Check_Standards_Corpus_Should_Report_An_Unreadable_Declaration_As_Unparseable()
    {
        let payload = StandardsCorpusPayload {
            documents: vec![Document("docs/standards/broken.md", DeclarationKind::Undeclared)],
            issues: vec![DeclarationIssue {
                path: "docs/standards/broken.md".to_owned(),
                reason: "the declaration's fence never closes".to_owned(),
            }],
            ..StandardsCorpusPayload::default()
        };

        let findings = Findings_For(&payload);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let first = findings.first().expect("asserted a length of one above");
        assert_eq!(first.applicability, Applicability::Unparseable);
        assert_eq!(first.gate, GateCategory::Review);
        assert_eq!(first.summary, "docs/standards/broken.md: the declaration's fence never closes");
        assert_eq!(first.locations, vec!["docs/standards/broken.md".to_owned()]);
    }

    #[test]
    fn Test_Check_Standards_Corpus_Should_Order_Its_Findings_By_Path()
    {
        let payload = StandardsCorpusPayload {
            rules: vec![
                A_Rule("docs/standards/zeta.md", "zeta", RuleSeverity::Must, GateCategory::Blocking),
                A_Rule("docs/standards/alpha.md", "alpha", RuleSeverity::Must, GateCategory::Blocking),
            ],
            ..StandardsCorpusPayload::default()
        };

        let findings = Findings_For(&payload);

        let paths: Vec<&str> = findings.iter().map(|finding| return finding.subject_name.as_str()).collect();
        assert_eq!(paths, vec!["docs/standards/alpha.md", "docs/standards/zeta.md"], "a report must not read in walk order");
    }

    /// A payload this crate's own encoder wrote is one this rule reads back — the two halves of
    /// the capability agreeing on the wire rather than only on the type.
    #[test]
    fn Test_Check_Standards_Corpus_Should_Report_A_Fact_Its_Own_Reader_Materialized()
    {
        let payload = StandardsCorpusPayload {
            roots: vec!["docs/standards".to_owned()],
            documents: vec![Document("docs/standards/naming.md", DeclarationKind::Rule)],
            rules: vec![A_Rule("docs/standards/naming.md", "naming", RuleSeverity::MustNot, GateCategory::Blocking)],
            ..StandardsCorpusPayload::default()
        };

        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Corpus_Fact(&mut store, &offer, &payload);

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Standards_Corpus(&mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted a length of one above").subject_name, "docs/standards/naming.md");
    }

    /// Each finding reduced to the three things every test here is about, so a test asserting
    /// all three states its whole expectation as one comparable list rather than as three
    /// parallel `Vec`s that could drift out of step.
    fn Outcomes(findings: Vec<Finding>) -> Vec<(String, GateCategory, Applicability)>
    {
        return findings
            .into_iter()
            .map(|finding| return (finding.subject_name, finding.gate, finding.applicability))
            .collect();
    }

    fn Document(path: &str, kind: DeclarationKind) -> DocumentDeclaration
    {
        return DocumentDeclaration {
            path: path.to_owned(),
            kind,
        };
    }

    fn A_Rule(path: &str, id: &str, severity: RuleSeverity, gate: GateCategory) -> DeclaredRule
    {
        return DeclaredRule {
            path: path.to_owned(),
            id: id.to_owned(),
            title: id.replace('-', " "),
            severity,
            gate,
            enforced_by: vec!["review".to_owned()],
        };
    }

    fn Offering() -> TestOffering
    {
        return test_support::Offered_Registry(OfferedProvider {
            contract: nomos_cap_standards_corpus::Capability_Contract(),
            capability: nomos_cap_standards_corpus::Capability(),
            version: nomos_cap_standards_corpus::CONTRACT_VERSION,
            provider: PROVIDER,
            guarantee: nomos_cap_standards_corpus::Ceiling(),
        })
        .expect("a fresh Registry holds neither this contract nor this provider");
    }

    fn Materialize_Corpus_Fact(
        store: &mut MemoryFactStore,
        offer: &nomos_capability::ProviderOffer,
        payload: &StandardsCorpusPayload,
    )
    {
        let bytes = Encode_Payload(payload);
        test_support::Materialize_Fact(
            store,
            FactToFile {
                subject: nomos_model::Subject_Of_Path(""),
                offer,
                semantic_inputs: InputDigest::Of(&[]),
                schema: Payload_Schema(),
                bytes,
            },
        )
        .expect("the fixture's store holds no fact under this key at a newer generation");
    }
}
