//! Every `nomos-rules` rule that cites a versioned contract cites one that exists, at the
//! version the record itself declares -- checked rather than trusted.
//!
//! `OD-PACKAGE-001` measured that `RuleId::New(COMPLETENESS_MIRROR)` cited no contract
//! version anywhere, which left `PKG-014`'s traceability requirement -- "mechanically
//! checked for semantic consistency with its normative contract" -- asserted in `D-134`'s
//! own prose and nowhere in the crate that implements it. `nomos_rules::CONTRACT_RECORD`
//! and `CONTRACT_RECORD_VERSION` are the citation; this file is the mechanical check, so an
//! amendment to `D-134` the implementation has not caught up to is a red test rather than a
//! silent drift.
//!
//! [`CITATIONS`] is why this file iterates rather than naming one rule. Written for the
//! mirror rule alone, it stayed green while `Check_Dependency_Direction` shipped and was
//! composed into `nomos check`, because a check aimed at one subject cannot notice a second
//! -- the shape `Check_Completeness_Mirrors` itself exists to catch in declared lists.
//! `Check_Naming_Convention` is deliberately absent: `naming.rs`'s own "why this has no
//! `CONTRACT_RECORD`" section says its contract is `README.md` prose, and a record-less rule
//! has no front matter to compare against. That absence is a fact about the rule, not a hole
//! in this file, and every rule carrying a real record means a row below --
//! `OD-GATE-019-REGISTRY-COHERENCE-B-4` added `Check_Lint_Diagnostics`, `Check_Dependency_
//! Policy` and `Check_Cross_Language_Correspondence` this way, the same shape `Check_Every_
//! Member_Declares_A_Band` already took by citing `DEPENDENCY_CONTRACT_RECORD` rather than a
//! record of its own.

use nomos_contract_tests::Workspace;
use std::path::Path;

/// Every rule citation in `nomos-rules`: the constant's own name, the record it names, and
/// the version it claims that record is at.
///
/// The name is carried so a failure says which citation is wrong rather than only which
/// record it was wrong about.
const CITATIONS: &[(&str, &str, u32)] = &[
    (
        "nomos_rules::CONTRACT_RECORD",
        nomos_rules::CONTRACT_RECORD,
        nomos_rules::CONTRACT_RECORD_VERSION,
    ),
    (
        "nomos_rules::DEPENDENCY_CONTRACT_RECORD",
        nomos_rules::DEPENDENCY_CONTRACT_RECORD,
        nomos_rules::DEPENDENCY_CONTRACT_RECORD_VERSION,
    ),
    (
        "nomos_rules::UNREAD_REACHES_FINDING_CONTRACT_RECORD",
        nomos_rules::UNREAD_REACHES_FINDING_CONTRACT_RECORD,
        nomos_rules::UNREAD_REACHES_FINDING_CONTRACT_RECORD_VERSION,
    ),
    (
        "nomos_rules::LINT_CONTRACT_RECORD",
        nomos_rules::LINT_CONTRACT_RECORD,
        nomos_rules::LINT_CONTRACT_RECORD_VERSION,
    ),
    (
        "nomos_rules::DEPENDENCY_POLICY_CONTRACT_RECORD",
        nomos_rules::DEPENDENCY_POLICY_CONTRACT_RECORD,
        nomos_rules::DEPENDENCY_POLICY_CONTRACT_RECORD_VERSION,
    ),
    (
        "nomos_rules::CROSS_LANGUAGE_CONTRACT_RECORD",
        nomos_rules::CROSS_LANGUAGE_CONTRACT_RECORD,
        nomos_rules::CROSS_LANGUAGE_CONTRACT_RECORD_VERSION,
    ),
    (
        "nomos_rules::WRITE_AUTHORITY_CONTRACT_RECORD",
        nomos_rules::WRITE_AUTHORITY_CONTRACT_RECORD,
        nomos_rules::WRITE_AUTHORITY_CONTRACT_RECORD_VERSION,
    ),
];

/// Every cited version must match its record's own front matter, or the citation is a claim
/// nothing verifies.
#[test]
fn Test_Every_Cited_Version_Should_Match_The_Records_Own_Front_Matter()
{
    let root = Workspace::Workspace_Root();

    for (constant, record, cited) in CITATIONS
    {
        let version = Record_Version(&root, record).unwrap_or_else(|| {
            panic!(
                "no document under docs/records/ named {record}- declares a front-matter \
                 version, so {constant} cites nothing checkable"
            )
        });

        assert_eq!(
            version, *cited,
            "{constant} cites {record} version {cited}, but the record itself is now at \
             version {version}. The rule's contract citation has not caught up to the \
             amendment."
        );
    }
}

/// The control the iteration needs. A citation table that has fallen behind the rules it
/// covers cannot be told from a complete one by reading it, so the row count is stated here
/// and has to be raised deliberately.
///
/// Not a mirror in `Check_Completeness_Mirrors`' sense -- there is no fact to resolve a Rust
/// constant list against -- but the same failure that rule exists to prevent: a row silently
/// missing, and every assertion above passing by not looking at it. This file was written
/// for one rule and stayed green through a second shipping.
#[test]
fn Test_The_Citation_Table_Should_Cover_Every_Cited_Rule()
{
    assert_eq!(
        CITATIONS.len(),
        7,
        "nomos-rules ships many rules, seven of which cite a versioned record: \
         Check_Completeness_Mirrors cites D-134, Check_Dependency_Direction and \
         Check_Every_Member_Declares_A_Band both cite OD-RULES-003, \
         Check_Unread_Reaches_A_Finding cites OD-RULES-008, Check_Lint_Diagnostics and \
         Check_Dependency_Policy both cite OD-RULES-010, Check_Cross_Language_Correspondence \
         cites OD-CAPABILITY-010, and Check_Write_Authority cites OD-RULES-023. \
         Check_Naming_Convention cites README.md prose and has no front matter \
         to compare against. A rule added with a real record needs a row in CITATIONS and \
         this number raised with it."
    );
}

/// The control this item's `done_when` asks for: the comparison has to actually fail when
/// the cited version and the record's own front matter disagree, not merely hold today by
/// coincidence. A fixture record is written under a scratch root so the amendment this
/// exercises never touches the committed one.
#[test]
fn Test_A_Mismatched_Version_Should_Fail_The_Comparison()
{
    let root = Scratch_Root("mismatched-version");
    Write_Fixture_Record(&root, "D-999", 3);

    let version = Record_Version(&root, "D-999").expect("the fixture record must be readable");

    assert_ne!(
        version, 2,
        "a fixture amended to version 3 must not read back as the citation's version 2"
    );
}

/// A scratch workspace root of this test's own, under `docs/records/`.
fn Scratch_Root(name: &str) -> std::path::PathBuf
{
    let root = std::env::temp_dir().join(format!("nomos-rule-contract-citation-{name}"));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("docs/records")).expect("creates a scratch docs/records");

    return root;
}

/// A minimal record document, front matter only, under a scratch root.
fn Write_Fixture_Record(root: &Path, record: &str, version: u32)
{
    let path = root.join("docs/records").join(format!("{record}-a-fixture-record.md"));
    let text = format!("---\nid: {record}\nversion: {version}\n---\n\n# A fixture record\n");

    std::fs::write(path, text).expect("writes a fixture record");
}

/// Every cited record must still exist under the identifier it is cited by, or the version
/// comparison above is quietly reading nothing.
#[test]
fn Test_Every_Cited_Record_Should_Still_Exist()
{
    let root = Workspace::Workspace_Root();

    for (constant, record, _) in CITATIONS
    {
        assert!(
            Record_Path(&root, record).is_some(),
            "{constant} names {record}, but no document under docs/records/ has a filename \
             starting with it"
        );
    }
}

/// The document under `docs/records/` whose filename starts with `{record}-`.
///
/// By stem prefix, because the slug is not derivable from the identifier alone --
/// `tests/contract/tests/requirement_trace/predicates.rs` locates a registration the same
/// way and for the same reason.
fn Record_Path(root: &Path, record: &str) -> Option<std::path::PathBuf>
{
    let prefix = format!("{record}-");
    let entries = std::fs::read_dir(root.join("docs/records")).ok()?;

    return entries
        .flatten()
        .map(|entry| return entry.path())
        .find(|path| {
            let is_markdown = path.extension().is_some_and(|extension| return extension == "md");
            let name_matches = path
                .file_name()
                .and_then(|name| return name.to_str())
                .is_some_and(|name| return name.starts_with(&prefix));

            return is_markdown && name_matches;
        });
}

/// The `version:` a record's own front matter declares, as an integer.
///
/// Deliberately not a YAML parser. Every record in `docs/records/` has exactly one
/// `version:` line inside its `---` front matter, and a full parser would be more code
/// trusted to answer a question one line already answers.
fn Record_Version(root: &Path, record: &str) -> Option<u32>
{
    let path = Record_Path(root, record)?;
    let text = std::fs::read_to_string(&path).ok()?;
    let body = text.strip_prefix("---")?;
    let end = body.find("\n---")?;

    return body.get(..end)?.lines().find_map(|line| {
        let value = line.strip_prefix("version:")?;
        return value.trim().parse().ok();
    });
}
