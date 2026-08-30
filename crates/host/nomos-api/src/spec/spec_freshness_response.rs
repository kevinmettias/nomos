//! [`Handle_Spec_Freshness`] and its own [`SpecFreshnessResponse`].

use nomos_spec_orchestration::{FreshnessAnswer, FreshnessRefusal, FreshnessRequest, SpecCommand};
use serde::Serialize;

use super::{Build_Corpus_Request, ProfileOutcomeResponse};

/// Compares every shipped profile's build root against the store, exactly as `nomos spec
/// freshness` would, and hands back a JSON-serializable response.
///
/// Follows [`crate::spec::spec_record_response::Handle_Spec_Record`]'s own composition. `run::freshness::
/// Freshness` is generic over `FileSystem` (it reads a rendered body and its sidecar at
/// `request.into`, through `nomos_platform::FileSystem::Read_To_String`), but never writes --
/// unlike `Render`, `Preview` and `Commit`, exposing it carries none of the "does a wire call
/// write to this host's disk" hazard those three do, since `StdFileSystem` here only ever
/// reads paths the caller already named.
#[must_use]
pub fn Handle_Spec_Freshness(request: &FreshnessRequest) -> SpecFreshnessResponse
{
    use nomos_platform_std::StdFileSystem;

    let corpus_request = Build_Corpus_Request();

    let outcome =
        nomos_spec_orchestration::Run(&SpecCommand::Freshness(request.clone()), &corpus_request, &StdFileSystem);

    let nomos_spec_orchestration::SpecOutcome::Freshness(result) = outcome
    else
    {
        // rust-panic: allow: Run's own contract guarantees it returns the SpecOutcome variant
        // naming the SpecCommand it was given -- any other outcome means Run itself is broken.
        unreachable!("Run always returns the SpecOutcome variant naming the SpecCommand it was given")
    };

    return SpecFreshnessResponse::From(result);
}

/// What a real `nomos spec freshness` produced, in a shape `serde_json` can hand across a
/// wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum SpecFreshnessResponse
{
    /// Every profile this run looked at, and what it required.
    Examined
    {
        examined: Vec<ProfileOutcomeResponse>,
        required: Vec<String>,
    },
    /// `--profile` or a `--require` names an identifier the catalogue does not carry.
    NoSuchProfile
    {
        requested: String,
        known: Vec<String>,
    },
    /// `--require` names a profile that `--profile` narrowed this run away from.
    RequirementUnexamined
    {
        requested: String,
        only: Option<String>,
    },
    /// The embedded catalogue or the store could not be read at all.
    Unreadable
    {
        /// What went wrong, as the underlying error's own `Display` renders it.
        cause: String,
    },
}

impl SpecFreshnessResponse
{
    pub(crate) fn From(result: Result<FreshnessAnswer, FreshnessRefusal>) -> Self
    {
        return match result
        {
            Ok(answer) => Self::Examined {
                examined: answer.examined.into_iter().map(ProfileOutcomeResponse::From).collect(),
                required: answer.required,
            },
            Err(FreshnessRefusal::NoSuchProfile { requested, known }) => Self::NoSuchProfile { requested, known },
            Err(FreshnessRefusal::RequirementUnexamined { requested, only }) =>
            {
                Self::RequirementUnexamined { requested, only }
            }
            Err(FreshnessRefusal::Project(error)) => Self::Unreadable { cause: error.to_string() },
            Err(FreshnessRefusal::Store(error)) => Self::Unreadable { cause: error.to_string() },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::spec::VerdictResponse;

    #[test]
    fn Test_A_Real_Call_Over_An_Empty_Root_Should_Examine_Every_Profile_As_Absent()
    {
        let request =
            FreshnessRequest { into: Unique_Scratch_Directory("empty-root"), profile: None, require: Vec::new() };

        let response = Handle_Spec_Freshness(&request);

        let SpecFreshnessResponse::Examined { examined, .. } = response
        else
        {
            // A freshly created, never-rendered scratch root has nothing to refuse against --
            // reaching any other variant here means the freshness path itself regressed, not
            // a condition this test should assert around.
            panic!("an empty scratch root examines cleanly: {response:?}");
        };
        assert!(!examined.is_empty(), "the embedded catalogue always ships at least one profile");
        assert!(
            examined.iter().all(|outcome| matches!(outcome.verdict, VerdictResponse::Absent)),
            "{examined:?}"
        );
    }

    #[test]
    fn Test_An_Unknown_Required_Profile_Should_Report_No_Such_Profile()
    {
        let request = FreshnessRequest {
            into: Unique_Scratch_Directory("unknown-required"),
            profile: None,
            require: vec!["definitely-not-a-real-profile".to_owned()],
        };

        let response = Handle_Spec_Freshness(&request);

        assert!(matches!(response, SpecFreshnessResponse::NoSuchProfile { .. }), "{response:?}");
    }

    #[test]
    fn Test_A_Real_Examined_Response_Should_Round_Trip_As_Json()
    {
        let request =
            FreshnessRequest { into: Unique_Scratch_Directory("round-trip"), profile: None, require: Vec::new() };

        let response = Handle_Spec_Freshness(&request);

        let json = serde_json::to_string(&response).expect("a SpecFreshnessResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized SpecFreshnessResponse always has this field");

        assert_eq!(outcome, "examined", "{json}");
    }

    /// An empty, unique scratch directory means nothing has ever been rendered there, so
    /// every shipped profile examines as `Absent` regardless of this session's own
    /// `NOMOS_V14_CORPUS` state -- the same zero-setup determinism every other test in this
    /// file already relies on.
    fn Unique_Scratch_Directory(label: &str) -> std::path::PathBuf
    {
        use std::sync::atomic::{AtomicU32, Ordering};
        // scope: allow this test-only counter has no owner beyond disambiguating calls within
        // one process; a bare pid does not distinguish two calls in the same test run.
        static COUNTER: AtomicU32 = AtomicU32::new(0);

        let directory = std::env::temp_dir().join(format!(
            "nomos-api-spec-freshness-{label}-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let _ignored = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("a fresh scratch directory can always be created");

        return directory;
    }
}
