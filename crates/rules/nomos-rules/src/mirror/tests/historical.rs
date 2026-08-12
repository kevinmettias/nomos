//! The three instances `OD-COMPLETENESS-001` analyses.
//!
//! None of the three can be replayed from git; each was repaired at the site. They
//! are reproduced here as they were originally written, which is the whole reason
//! this rule takes its subject as an argument rather than reading the tree. That the
//! subject is now a pair — text and a reader — is the amendment `OD-RULES-001` makes
//! to `D-134`, and these tests are the property it claims is unaffected. If they could
//! not be expressed against a reader, the amendment would be wrong.

use super::*;

/// Instance one and two: `Table::All`, a list kept beside the enum, over which
/// `Assert_Complete` and `Assert_Landed` both quantified. A migration adding a table
/// without adding it here left that table outside both guards.
#[test]
fn Test_The_Table_Universe_As_Originally_Written_Should_Be_Found_Unmirrored()
{
    let findings = Findings_Over(&[Source(
        "crates/spec/nomos-spec-store/src/store.rs",
        "impl Table\n\
         {\n\
         \x20   /// Every table in the schema.\n\
         \x20   pub const fn All() -> &'static [Self]\n\
         \x20   {\n\
         \x20   }\n\
         }\n",
    )]);

    let finding = Only(&findings);

    assert_eq!(finding.subject_name, "Table::All");
    assert_eq!(finding.rule.As_Str(), COMPLETENESS_MIRROR);
    assert!(finding.summary.contains("declares no mirror"), "{}", finding.summary);
}

/// Instance three: `GOVERNING_RECORD_IDS`, compared against a store seeded from
/// `GOVERNING_RECORD_IDS` — a comparison that cannot fail. Six governing records sat
/// outside it for months.
#[test]
fn Test_The_Governing_Universe_As_Originally_Written_Should_Be_Found_Unmirrored()
{
    let findings = Findings_Over(&[Source(
        "crates/spec/nomos-spec-store/src/governing.rs",
        "/// The records this store seeds itself with.\n\
         pub const GOVERNING_RECORD_IDS: &[&str] = &[\n\
         \x20   \"ARC-SPECDB-001\",\n\
         ];\n",
    )]);

    assert_eq!(Only(&findings).subject_name, "GOVERNING_RECORD_IDS");
}

/// And all three together, in one run, because `P10-FIRST-CHECK` asks for the rule
/// to fail on all three rather than on each in isolation.
#[test]
fn Test_All_Three_Historical_Instances_Should_Fail_This_Rule()
{
    let findings = Findings_Over(&[
        Source(
            "crates/spec/nomos-spec-store/src/store.rs",
            "impl Table\n{\n    pub const fn All() -> &'static [Self]\n    {\n    }\n}\n",
        ),
        Source(
            "crates/spec/nomos-spec-store/src/governing.rs",
            "pub const GOVERNING_RECORD_IDS: &[&str] = &[];\n",
        ),
        Source("tests/contract/src/gates.rs", "pub const CORPUS_VARIABLES: &[&str] = &[];\n"),
    ]);

    let judged: Vec<&str> = findings
        .iter()
        .map(|finding| return finding.subject_name.as_str())
        .collect();

    assert_eq!(
        judged,
        vec!["CORPUS_VARIABLES", "GOVERNING_RECORD_IDS", "Table::All"],
        "all three instances OD-COMPLETENESS-001 analyses must be found"
    );
}
