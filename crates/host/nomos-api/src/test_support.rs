//! Test-only fixtures shared by more than one response module's own test suite --
//! `crate::spec` and `crate::response` today. Declared `#[cfg(test)]` by this crate's own
//! `lib.rs`, so nothing here compiles into a real build, the same discipline
//! `crate::work::tests_support` already keeps for its own verbs.
//!
//! Every fixture here hands its failure back rather than unwrapping its own steps. A fixture
//! is not the code under test: a scratch directory the machine will not let it create, or a
//! staged edit it cannot write, says nothing about the under-test code, and only the caller
//! knows which claim it was trying to prove when the step failed.

use serde::Serialize;
use std::path::{Path, PathBuf};

/// The caller's own subsystem (`"spec-commit"`, `"gate-explain"`, and so on) -- the half of a
/// scratch path's name a module fixes rather than varies.
///
/// A type of its own rather than a second `&str`, which is what made
/// `Unique_Scratch_Directory("commit", "spec-commit")` a call the compiler would have
/// accepted and nobody would have noticed.
pub(crate) struct Area(pub(crate) &'static str);

/// A scratch directory of this test's own -- never a real shared directory a live session
/// writes to concurrently. `area` names the caller's own subsystem (`"spec-commit"`,
/// `"gate-explain"`, and so on) so two areas naming the same `label` still land in different
/// places; a call-local counter on top of that, since several fixtures in one area share one
/// `label`, and the default test runner's threads would otherwise race on one directory a
/// bare pid gave them. `process::id()` alone tells two runs of the whole suite apart; it says
/// nothing about two calls inside one.
///
/// # Errors
///
/// Returns whatever [`std::fs::create_dir_all`] refuses -- a temp directory that is not
/// writable, or a file already sitting where this call's own name computes to.
pub(crate) fn Unique_Scratch_Directory(area: Area, label: &str) -> Result<PathBuf, std::io::Error>
{
    // scope: allow this test-only counter has no owner beyond disambiguating calls within one
    // process; a bare pid does not distinguish two calls in the same test run.
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    // atomic-ordering: allow: only used to give two calls in this process different numbers;
    // nothing else synchronizes on it or reads memory ordered by this counter.
    let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let directory = std::env::temp_dir().join(format!("nomos-api-{}-{label}-{}-{unique}", area.0, std::process::id()));
    let _ignored = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory)?;

    return Ok(directory);
}

/// The directory name a [`Probe_Tree`] is created under -- the half of that fixture's own
/// naming that varies from probe to probe, `area` being the half its caller fixes.
///
/// A type of its own rather than a second `&str`, the same reason [`Area`] is one: a call
/// reading `Probe_Tree(area, name, policy)` would take two transposable strings beside the
/// one input that is not a name, and the compiler would raise nothing.
pub(crate) struct TreeName<'a>(pub(crate) &'a str);

/// A one-crate tree with real source and a declared `nomos-gate.json`, so a gate run over it
/// reaches `Judged` and resolves a policy of its own -- the two things an empty directory
/// cannot do.
///
/// `area` and `name` name the scratch directory and nothing else; the tree itself is fixed
/// here, and `policy` is the one thing a caller varies, because the policy is what a gate run
/// reads. Built by `crate::response::gate_compare_response` and
/// `crate::response::gate_run_response`, which wrote byte-identical trees under their own
/// `<prefix>-{name}-{pid}` names before this existed.
///
/// # Errors
///
/// Returns whatever [`Unique_Scratch_Directory`] or the three writes below refuse -- a temp
/// directory that is not writable, or a name already taken.
pub(crate) fn Probe_Tree(area: Area, name: TreeName, policy: &str) -> Result<PathBuf, std::io::Error>
{
    let root = Unique_Scratch_Directory(area, name.0)?;
    std::fs::create_dir_all(root.join("src"))?;
    let manifest = "[package]\nname = \"probe\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
    std::fs::write(root.join("Cargo.toml"), manifest)?;
    std::fs::write(root.join("src").join("lib.rs"), "pub fn thing() -> i32 { return 1; }\n")?;
    std::fs::write(root.join("nomos-gate.json"), policy)?;

    return Ok(root);
}

/// Asserts that `response` serializes, round-trips through `serde_json`, and carries
/// `expected_outcome` under the `"outcome"` field every `#[serde(tag = "outcome", ...)]`
/// response in this crate tags itself with.
///
/// # Errors
///
/// Returns whatever [`serde_json`] refuses while serializing `response` or parsing that text
/// back. A mismatched outcome is an assertion rather than an error, because a wrong tag is
/// precisely what this function was called to report.
pub(crate) fn Assert_Round_Trips_As_Json<Response: Serialize>(
    response: &Response,
    expected_outcome: &str,
) -> Result<(), serde_json::Error>
{
    let json = serde_json::to_string(response)?;
    let parsed: serde_json::Value = serde_json::from_str(&json)?;
    assert_eq!(
        parsed.get("outcome"),
        Some(&serde_json::Value::String(expected_outcome.to_owned())),
        "{json}"
    );

    return Ok(());
}

/// Stages a canonical heading rename against `id`'s own real, embedded markdown (read
/// through this crate's own `Handle_Spec_Markdown`, so this needs no corpus and touches no
/// file this repository tracks), writes it to `staged.md` under `into`, and hands back the
/// path it was written to. Shared by every `Handle_Spec_*` test fixture that needs a real
/// staged edit to preview or commit.
///
/// # Errors
///
/// [`std::io::ErrorKind::NotFound`] when `id` names no record this binary carries, carrying
/// the store's own refusal text -- every id this fixture is called with names one, so that
/// arm means the call site asked for a record that does not exist rather than that a runtime
/// condition went wrong. And whatever [`std::fs::write`] refuses when the staged text cannot
/// be written under `into`.
pub(crate) fn Staged_Heading_Rename(id: &str, into: &Path) -> Result<PathBuf, std::io::Error>
{
    let request = nomos_spec_orchestration::RecordRequest { id: id.to_owned(), revision: None };

    let markdown = match crate::spec::Handle_Spec_Markdown(&request)
    {
        crate::spec::MarkdownResponse::Resolved { markdown, .. } => markdown,
        crate::spec::MarkdownResponse::Refused { cause } =>
        {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, cause));
        }
    };

    let edited = markdown.replace("## Decision", "## The decision");
    let staged = into.join("staged.md");
    std::fs::write(&staged, &edited)?;

    return Ok(staged);
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The area every fixture test in this module builds its scratch directories under.
    const TEST_SUPPORT_AREA: Area = Area("test-support");

    #[test]
    fn Test_Unique_Scratch_Directory_Should_Give_Two_Calls_In_One_Area_Different_Directories()
    {
        let first = Unique_Scratch_Directory(TEST_SUPPORT_AREA, "same-label")
            .expect("the temp directory is writable and this call's own name is fresh");
        let second = Unique_Scratch_Directory(TEST_SUPPORT_AREA, "same-label")
            .expect("the temp directory is writable and the counter gave this call a fresh name");

        assert_ne!(first, second);
        assert!(first.is_dir());
        assert!(second.is_dir());

        let _ignored = std::fs::remove_dir_all(&first);
        let _ignored = std::fs::remove_dir_all(&second);
    }

    #[test]
    fn Test_Assert_Round_Trips_As_Json_Should_Accept_Every_Tagged_Values_Own_Outcome_Name()
    {
        for (value, outcome) in Sample_Outcomes()
        {
            Assert_Round_Trips_As_Json(&value, outcome)
                .expect("every Sample variant serializes and parses back as a tagged object");
        }
    }

    #[test]
    fn Test_Assert_Round_Trips_As_Json_Should_Panic_When_The_Outcome_Field_Does_Not_Match()
    {
        for (value, _outcome) in Sample_Outcomes()
        {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                // The assertion is the subject here, so the returned error is not what this
                // test is asking about and the panic it raises is.
                let _ignored = Assert_Round_Trips_As_Json(&value, "not-a-real-outcome");
            }));

            assert!(result.is_err(), "{value:?} must panic when checked against the wrong outcome");
        }
    }

    #[test]
    fn Test_Staged_Heading_Rename_Should_Replace_The_Decision_Heading_In_A_Real_Record()
    {
        let into = Unique_Scratch_Directory(TEST_SUPPORT_AREA, "staged-heading-rename")
            .expect("the temp directory is writable and this call's own name is fresh");

        let staged = Staged_Heading_Rename("D-132", &into)
            .expect("D-132 is a governing record embedded in this binary");
        let content = std::fs::read_to_string(&staged).expect("reads the staged file back");

        let _ignored = std::fs::remove_dir_all(&into);

        assert!(content.contains("## The decision"), "{content}");
        assert!(!content.contains("## Decision\n"), "{content}");
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
}
