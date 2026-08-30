//! Test-only fixtures shared by more than one response module's own test suite --
//! `crate::spec` and `crate::response` today. Declared `#[cfg(test)]` by this crate's own
//! `lib.rs`, so nothing here compiles into a real build, the same discipline
//! `crate::work::tests_support` already keeps for its own verbs.

use serde::Serialize;
use std::path::{Path, PathBuf};

/// A scratch directory of this test's own -- never a real shared directory a live session
/// writes to concurrently. `area` names the caller's own subsystem (`"spec-commit"`,
/// `"gate-explain"`, and so on) so two areas naming the same `label` still land in different
/// places; a call-local counter on top of that, since several fixtures in one area share one
/// `label`, and the default test runner's threads would otherwise race on one directory a
/// bare pid gave them. `process::id()` alone tells two runs of the whole suite apart; it says
/// nothing about two calls inside one.
pub(crate) fn Unique_Scratch_Directory(area: &str, label: &str) -> PathBuf
{
    // scope: allow this test-only counter has no owner beyond disambiguating calls within one
    // process; a bare pid does not distinguish two calls in the same test run.
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    // atomic-ordering: allow: only used to give two calls in this process different numbers;
    // nothing else synchronizes on it or reads memory ordered by this counter.
    let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let directory = std::env::temp_dir().join(format!("nomos-api-{area}-{label}-{}-{unique}", std::process::id()));
    let _ignored = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("a fresh scratch directory can always be created");

    return directory;
}

/// Asserts that `response` serializes, round-trips through `serde_json`, and carries
/// `expected_outcome` under the `"outcome"` field every `#[serde(tag = "outcome", ...)]`
/// response in this crate tags itself with.
pub(crate) fn Assert_Round_Trips_As_Json<T: Serialize>(response: &T, expected_outcome: &str)
{
    let json = serde_json::to_string(response).expect("a tagged response always serializes");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
    let outcome = parsed.get("outcome").expect("a serialized response always has this field");

    assert_eq!(outcome, expected_outcome, "{json}");
}

/// Stages a canonical heading rename against `id`'s own real, embedded markdown (read
/// through this crate's own `Handle_Spec_Markdown`, so this needs no corpus and touches no
/// file this repository tracks), writes it to `staged.md` under `into`, and hands back the
/// path it was written to. Shared by every `Handle_Spec_*` test fixture that needs a real
/// staged edit to preview or commit.
pub(crate) fn Staged_Heading_Rename(id: &str, into: &Path) -> PathBuf
{
    let request = nomos_spec_orchestration::RecordRequest { id: id.to_owned(), revision: None };
    let crate::spec::MarkdownResponse::Resolved { markdown, .. } = crate::spec::Handle_Spec_Markdown(&request)
    else
    {
        // Every id this test fixture is called with names a real governing record that ships
        // embedded in the binary, so this branch means the fixture was called with the wrong
        // id, not a runtime condition the fixture should tolerate.
        panic!("{id} is a governing record, embedded even with no corpus")
    };
    let edited = markdown.replace("## Decision", "## The decision");
    let staged = into.join("staged.md");
    std::fs::write(&staged, &edited).expect("writes the staged edit");

    return staged;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Unique_Scratch_Directory_Should_Give_Two_Calls_In_One_Area_Different_Directories()
    {
        let first = Unique_Scratch_Directory("test-support", "same-label");
        let second = Unique_Scratch_Directory("test-support", "same-label");

        assert_ne!(first, second);
        assert!(first.is_dir());
        assert!(second.is_dir());

        let _ignored = std::fs::remove_dir_all(&first);
        let _ignored = std::fs::remove_dir_all(&second);
    }

    #[derive(Debug, Serialize)]
    #[serde(tag = "outcome", rename_all = "snake_case")]
    enum Sample
    {
        Accepted,
        Completed,
        Rejected,
    }

    /// Every `Sample` variant paired with the exact snake_case address `serde`'s own
    /// `rename_all = "snake_case"` gives it -- the one outcome name `Assert_Round_Trips_As_Json`
    /// must accept for each, and every other name it must reject.
    fn Sample_Outcomes() -> Vec<(Sample, &'static str)>
    {
        return vec![
            (Sample::Accepted, "accepted"),
            (Sample::Completed, "completed"),
            (Sample::Rejected, "rejected"),
        ];
    }

    #[test]
    fn Test_Assert_Round_Trips_As_Json_Should_Accept_Every_Tagged_Values_Own_Outcome_Name()
    {
        for (value, outcome) in Sample_Outcomes()
        {
            Assert_Round_Trips_As_Json(&value, outcome);
        }
    }

    #[test]
    fn Test_Assert_Round_Trips_As_Json_Should_Panic_When_The_Outcome_Field_Does_Not_Match()
    {
        for (value, _outcome) in Sample_Outcomes()
        {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                Assert_Round_Trips_As_Json(&value, "not-a-real-outcome");
            }));

            assert!(result.is_err(), "{value:?} must panic when checked against the wrong outcome");
        }
    }

    #[test]
    fn Test_Staged_Heading_Rename_Should_Replace_The_Decision_Heading_In_A_Real_Record()
    {
        let into = Unique_Scratch_Directory("test-support", "staged-heading-rename");

        let staged = Staged_Heading_Rename("D-132", &into);
        let content = std::fs::read_to_string(&staged).expect("reads the staged file back");

        let _ignored = std::fs::remove_dir_all(&into);

        assert!(content.contains("## The decision"), "{content}");
        assert!(!content.contains("## Decision\n"), "{content}");
    }
}
