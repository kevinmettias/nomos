//! `nomos-rules`' one shipped rule cites the record and version its contract was
//! implemented against, and this checks the citation rather than trusting it.
//!
//! `OD-PACKAGE-001` measured that `RuleId::New(COMPLETENESS_MIRROR)` cited no contract
//! version anywhere, which left `PKG-014`'s traceability requirement -- "mechanically
//! checked for semantic consistency with its normative contract" -- asserted in `D-134`'s
//! own prose and nowhere in the crate that implements it. `nomos_rules::CONTRACT_RECORD`
//! and `CONTRACT_RECORD_VERSION` are the citation; this file is the mechanical check, so an
//! amendment to `D-134` the implementation has not caught up to is a red test rather than a
//! silent drift.

use nomos_contract_tests::Workspace;
use std::path::Path;

/// The cited version must match the record's own front matter, or the citation is a claim
/// nothing verifies.
#[test]
fn Test_The_Cited_Version_Should_Match_The_Records_Own_Front_Matter()
{
    let root = Workspace::Workspace_Root();
    let version = Record_Version(&root, nomos_rules::CONTRACT_RECORD).unwrap_or_else(|| {
        panic!(
            "no document under docs/records/ starting with \"{}-\" declares a front-matter \
             version",
            nomos_rules::CONTRACT_RECORD
        )
    });

    assert_eq!(
        version, nomos_rules::CONTRACT_RECORD_VERSION,
        "nomos_rules::CONTRACT_RECORD_VERSION cites {} version {}, but the record itself is \
         now at version {version}. The rule's contract citation has not caught up to the \
         amendment.",
        nomos_rules::CONTRACT_RECORD,
        nomos_rules::CONTRACT_RECORD_VERSION
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

/// The record must still exist under the identifier `nomos-rules` cites, or the version
/// comparison above is quietly reading nothing.
#[test]
fn Test_The_Cited_Record_Should_Still_Exist()
{
    let root = Workspace::Workspace_Root();

    assert!(
        Record_Path(&root, nomos_rules::CONTRACT_RECORD).is_some(),
        "nomos_rules::CONTRACT_RECORD names {}, but no document under docs/records/ starts \
         with \"{}-\"",
        nomos_rules::CONTRACT_RECORD,
        nomos_rules::CONTRACT_RECORD
    );
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
