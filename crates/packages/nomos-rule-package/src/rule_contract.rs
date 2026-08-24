//! Which governing record a rule's normative specification cites, if any.

/// A rule's citation of the governing record that specifies it.
///
/// Mirrors `nomos_rules`' own `CONTRACT_RECORD`/`CONTRACT_RECORD_VERSION` shape exactly,
/// because `OD-PACKAGE-008`'s four-rule measurement found that shape real and
/// mechanically checked (`tests/contract/tests/rule_contract_citation.rs`) rather than
/// invented. Optional on [`crate::RulePackage`] for the same reason it is absent from
/// `nomos_rules::naming`: a rule's contract may be prose in `README.md` with no
/// `version:` field to cite, and a manifest that required one would misdescribe a real,
/// deliberate rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleContract
{
    /// The governing record's identifier, such as `"D-134"`.
    pub record: String,
    /// The record's own front-matter version this rule was written against.
    pub version: u32,
}

impl RuleContract
{
    /// Constructs a contract citation.
    #[must_use]
    pub const fn New(record: String, version: u32) -> Self
    {
        return Self { record, version };
    }
}
