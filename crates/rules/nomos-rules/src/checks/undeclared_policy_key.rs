//! `undeclared-policy-key` — a key a repository declares in a policy file this workspace owns,
//! which no axis names and so no rule reads.
//!
//! # Why this is a finding rather than a no-op
//!
//! The limits provider validates a declared value and accepts any key, so a repository that
//! writes `nesting-depth-maxx: 5` used to get a row nothing read, no finding, and the compiled
//! default -- its preference silently not applied. code-standards decided the opposite for its
//! own keys, in its own words: "DisallowUnknownFields turns a typo'd key into a loud error
//! rather than a no-op." `OD-RULES-035` decided this workspace holds its own files to the same
//! standard, and this rule is that decision: every key in `nomos-limits.json` that no axis in
//! `rule_descriptor::policy_axis::LIMITS_AXES` names is one `Blocking` finding, because the
//! remedy is a person correcting the declaration and until they do the repository is judged
//! against a limit it did not choose.
//!
//! # Why only a file this workspace owns
//!
//! The families this rule judges are declared on its descriptor rather than inferred, and today
//! they are exactly limits. Naming is read from `standards.json`, a file another tool decodes and
//! owns; a key this workspace does not read there may be one that tool does -- `standards.json`'s
//! `tiers` is read by code-standards and by nothing here -- so "unread here" is not "unread", and
//! reporting it would be this rule inventing a defect in somebody else's file.
//!
//! # Where the rows come from
//!
//! From the limits family's own payload read, `checks::structure::Materialized_Limits_Payload`,
//! rather than a second requirement written here: `OD-RULES-035` decided a row-shaped family has
//! one resolver and one requirement, and a rule that needs the rows rather than one resolved value
//! still reads them through that one read.

use super::structure::{Materialized_Limits_Payload, LIMITS_JSON};
use crate::rule_descriptor::policy_axis::LIMITS_AXES;
use crate::rule_descriptor::RequiredFact;
use nomos_analysis::FactReader;
use nomos_cap_limits_policy::{PolicyRow, Scope};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// This rule's own identifier.
pub const UNDECLARED_POLICY_KEY: &str = "undeclared-policy-key";

/// The record this implementation's contract is written against. `OD-RULES-035` decided a key
/// no axis declares is a `Blocking` finding in a file this workspace owns and never in a block
/// of a shared file; `tests/contract/tests/rule_contract_citation.rs` reads that record's own
/// front matter on every run and compares it against
/// [`UNDECLARED_POLICY_KEY_CONTRACT_RECORD_VERSION`].
pub const UNDECLARED_POLICY_KEY_CONTRACT_RECORD: &str = "OD-RULES-035";

/// The version of [`UNDECLARED_POLICY_KEY_CONTRACT_RECORD`] this implementation was written
/// against.
pub const UNDECLARED_POLICY_KEY_CONTRACT_RECORD_VERSION: u32 = 1;

/// Reports every key a repository declares in `nomos-limits.json`, repository-wide or under a
/// language, that no limits axis names -- one `Blocking` finding per key.
///
/// Judges nothing when no limits fact is available: a repository with no file declared nothing,
/// and nothing it did not declare can be misspelled.
#[must_use]
pub fn Check_Undeclared_Policy_Key(facts: &mut dyn FactReader) -> Vec<Finding>
{
    let Some(payload) = Materialized_Limits_Payload(facts, RequiredFact::LimitsPolicy)
    else
    {
        return Vec::new();
    };

    let mut findings: Vec<Finding> = payload.rows.iter().filter(|row| return !Is_A_Declared_Axis(&row.key)).map(Finding_For_Row).collect();

    findings.sort_by(|left, right| return left.address.cmp(&right.address));
    return findings;
}

/// Whether some limits axis is declared under `key`.
fn Is_A_Declared_Axis(key: &str) -> bool
{
    return LIMITS_AXES.iter().any(|axis| return axis.key == key);
}

fn Finding_For_Row(row: &PolicyRow) -> Finding
{
    let (where_declared, scope) = match &row.scope
    {
        Scope::Repository => ("repository-wide".to_owned(), "*".to_owned()),
        Scope::Language(language) => (format!("under languages.{language}"), language.clone()),
    };

    return Finding {
        address: Some(format!("{LIMITS_JSON}#{scope}.{}", row.key)),
        rule: RuleId::New(UNDECLARED_POLICY_KEY),
        subject: nomos_model::Subject_Of_Path(LIMITS_JSON),
        subject_name: LIMITS_JSON.to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!(
            "{LIMITS_JSON} declares `{}` {where_declared}, and no limits axis is named that, so no rule reads it and the limit it was meant to set is judged against its default",
            row.key
        ),
        locations: vec![LIMITS_JSON.to_owned()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::checks::test_support::{self, FactToFile, OfferedProvider, Test_Context, TestOffering};
    use nomos_analysis::{InputDigest, MemoryFactStore, Reader};
    use nomos_capability::ProviderOffer;

    /// A misspelling of `nesting-depth-max`, the case this rule exists for.
    const MISSPELLED: &str = "nesting-depth-maxx";

    #[test]
    fn Test_Check_Undeclared_Policy_Key_Should_Report_A_Repository_Wide_Key_No_Axis_Names()
    {
        let findings = Findings_Over(vec![Row(Scope::Repository, MISSPELLED)]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New(UNDECLARED_POLICY_KEY));
        assert_eq!(found.gate, GateCategory::Blocking);
        assert_eq!(found.subject_name, "nomos-limits.json");
        assert!(found.summary.contains("`nesting-depth-maxx` repository-wide"), "{}", found.summary);
    }

    #[test]
    fn Test_Check_Undeclared_Policy_Key_Should_Name_The_Language_A_Key_Was_Declared_Under()
    {
        let findings = Findings_Over(vec![Row(Scope::Language("go".to_owned()), "file-size-hardlines")]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert!(found.summary.contains("`file-size-hardlines` under languages.go"), "{}", found.summary);
        assert_eq!(found.address.as_deref(), Some("nomos-limits.json#go.file-size-hardlines"));
    }

    /// The four keys a repository writes today, spelled here as a repository spells them rather
    /// than read back from `LIMITS_AXES`: a test that took its keys from the declaration would
    /// stop declaring an axis the moment the declaration dropped it, and pass. Spelled out, an
    /// axis dropped from `LIMITS_AXES` is reported here, which is what makes this the declaration's
    /// falsifier.
    const LIMITS_A_REPOSITORY_WRITES: [&str; 4] = ["file-size-review-lines", "file-size-hard-lines", "parameter-count-max", "nesting-depth-max"];

    /// Every key a repository may write, repository-wide and for a language, reports nothing.
    #[test]
    fn Test_Check_Undeclared_Policy_Key_Should_Accept_Every_Declared_Axis_At_Either_Scope()
    {
        let rows = LIMITS_A_REPOSITORY_WRITES
            .iter()
            .flat_map(|key| return [Row(Scope::Repository, key), Row(Scope::Language("go".to_owned()), key)])
            .collect();

        let findings = Findings_Over(rows);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// One finding per key, never collapsed: two misspellings are two things to fix.
    #[test]
    fn Test_Check_Undeclared_Policy_Key_Should_Report_Each_Undeclared_Key_Once()
    {
        let findings = Findings_Over(vec![Row(Scope::Repository, MISSPELLED), Row(Scope::Repository, "parameter-count")]);

        let addresses: Vec<Option<&str>> = findings.iter().map(|finding| return finding.address.as_deref()).collect();
        assert_eq!(addresses, vec![Some("nomos-limits.json#*.nesting-depth-maxx"), Some("nomos-limits.json#*.parameter-count")]);
    }

    #[test]
    fn Test_Check_Undeclared_Policy_Key_Should_Judge_Nothing_Without_A_Limits_Fact()
    {
        let store = MemoryFactStore::New();
        let registry = nomos_capability::Registry::New();
        let mut facts = Reader::On(&store, &registry, Test_Context());

        assert!(Check_Undeclared_Policy_Key(&mut facts).is_empty());
    }

    /// `OD-RULES-035`'s exclusion, pinned: a naming key no axis names, read from `standards.json`,
    /// is not this rule's to report, because that file's other owner may read it.
    #[test]
    fn Test_Check_Undeclared_Policy_Key_Should_Not_Judge_A_Naming_Key()
    {
        let TestOffering { mut store, registry, offer } = test_support::Offered_Registry(OfferedProvider {
            contract: nomos_cap_naming_policy::Capability_Contract(),
            capability: nomos_cap_naming_policy::Capability(),
            version: nomos_cap_naming_policy::CONTRACT_VERSION,
            provider: "nomos.test.naming.provides",
            guarantee: nomos_cap_naming_policy::Ceiling(),
        })
        .expect("a fresh Registry holds neither this contract nor this provider");
        let payload = nomos_cap_naming_policy::NamingPolicyPayload {
            rows: vec![nomos_cap_naming_policy::PolicyRow {
                scope: nomos_cap_naming_policy::Scope::Repository,
                symbol: "functoin".to_owned(),
                case: nomos_cap_naming_policy::Case::LowerSnake,
            }],
        };
        File_Fact(&mut store, &offer, nomos_cap_naming_policy::Payload_Schema(), nomos_cap_naming_policy::Encode_Payload(&payload));
        let mut facts = Reader::On(&store, &registry, Test_Context());

        assert!(Check_Undeclared_Policy_Key(&mut facts).is_empty());
    }

    #[test]
    fn Test_Is_A_Declared_Axis_Should_Know_Every_Limits_Axis_And_Nothing_Else()
    {
        assert!(LIMITS_A_REPOSITORY_WRITES.iter().all(|key| return Is_A_Declared_Axis(key)));
        assert!(!Is_A_Declared_Axis(MISSPELLED));
    }

    #[test]
    fn Test_Finding_For_Row_Should_Address_The_Key_At_Its_Scope()
    {
        let found = Finding_For_Row(&Row(Scope::Repository, MISSPELLED));

        assert_eq!(found.address.as_deref(), Some("nomos-limits.json#*.nesting-depth-maxx"));
        assert_eq!(found.locations, vec!["nomos-limits.json".to_owned()]);
    }

    fn Row(scope: Scope, key: &str) -> PolicyRow
    {
        return PolicyRow { scope, key: key.to_owned(), value: 1 };
    }

    /// What the rule reports over a repository whose `nomos-limits.json` declares `rows`.
    fn Findings_Over(rows: Vec<PolicyRow>) -> Vec<Finding>
    {
        let TestOffering { mut store, registry, offer } = test_support::Offered_Registry(OfferedProvider {
            contract: nomos_cap_limits_policy::Capability_Contract(),
            capability: nomos_cap_limits_policy::Capability(),
            version: nomos_cap_limits_policy::CONTRACT_VERSION,
            provider: "nomos.test.limits.provides",
            guarantee: nomos_cap_limits_policy::Ceiling(),
        })
        .expect("a fresh Registry holds neither this contract nor this provider");
        let payload = nomos_cap_limits_policy::LimitsPolicyPayload { rows };
        File_Fact(&mut store, &offer, nomos_cap_limits_policy::Payload_Schema(), nomos_cap_limits_policy::Encode_Payload(&payload));
        let mut facts = Reader::On(&store, &registry, Test_Context());

        return Check_Undeclared_Policy_Key(&mut facts);
    }

    fn File_Fact(store: &mut MemoryFactStore, offer: &ProviderOffer, schema: nomos_contracts::SchemaId, bytes: Vec<u8>)
    {
        test_support::Materialize_Fact(
            store,
            FactToFile { subject: nomos_model::Subject_Of_Path(""), offer, semantic_inputs: InputDigest::Of(&[]), schema, bytes },
        )
        .expect("the fixture's store holds no fact under this key at a newer generation");
    }
}
