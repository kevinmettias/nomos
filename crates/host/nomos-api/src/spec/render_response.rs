//! [`Handle_Spec_Render`] and its own [`RenderResponse`].

use nomos_spec_orchestration::{RenderAnswer, RenderRefusal, RenderRequest, SpecCommand};
use nomos_spec_project::Stamp;
use serde::Serialize;
use std::path::PathBuf;

/// Builds a projection profile and writes both halves of it under `request.into`, exactly as
/// `nomos spec render` would, and hands back a JSON-serializable response.
///
/// Follows [`crate::spec::record_response::Handle_Spec_Record`]'s own composition. Unlike `Record`,
/// `Table`, `Markdown`, `Freshness` and `Preview`, this verb does write: `run::render::Rendered_Projection`
/// places a built projection's body and its sidecar under `request.into` through
/// `nomos_platform::FileSystem::Replace_Atomically`, unconditionally overwriting whatever
/// was there. This crate already performs real, unauthenticated filesystem-backed
/// writes over a wire call today -- [`crate::Handle_Work_Claim`] writes a real ledger file at
/// a caller-named directory -- and `request.into` here plays the same role `directory` does
/// there: a caller-named root, with the leaf path underneath it chosen by this crate's own
/// store/profile machinery rather than by the caller. The one thing `Render`'s write is not
/// is destructive in the way `Commit`'s vacate step can be: a rendered projection is a
/// derived, regenerable artifact, not the governing record itself.
#[must_use]
pub fn Handle_Spec_Render(request: &RenderRequest) -> RenderResponse
{
    use super::Build_Corpus_Request;
    use nomos_composer_std::FILE_SYSTEM;

    let corpus_request = Build_Corpus_Request();

    let outcome =
        nomos_spec_orchestration::Run(&SpecCommand::Render(request.clone()), &corpus_request, &FILE_SYSTEM);

    let nomos_spec_orchestration::SpecOutcome::Render(result) = outcome
    else
    {
        return RenderResponse::Unreadable {
            cause: super::UNANSWERED_SPEC_OUTCOME.to_owned(),
        };
    };

    return RenderResponse::From(result);
}

/// What a real `nomos spec render` produced, in a shape `serde_json` can hand across a wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum RenderResponse
{
    /// Both halves of the built projection, placed where the run asked for them.
    Placed
    {
        /// The profile identifier that was built.
        id: String,
        /// Where the body landed.
        body: PathBuf,
        /// Where its sidecar landed.
        sidecar: PathBuf,
        /// What the build selected, and what it hashes to. Already `Serialize`, reused
        /// directly.
        stamp: Stamp,
    },
    /// The catalogue does not carry a profile by this name.
    NoSuchProfile
    {
        requested: String,
        known: Vec<String>,
    },
    /// The built projection could not be written where it was asked to go.
    Unwritable
    {
        path: String,
        /// What went wrong, as `FileSystemError`'s own `Display` renders it.
        cause: String,
    },
    /// The catalogue could not be read, resolving the subject failed, building the
    /// projection failed, or the store could not be assembled at all.
    Unreadable
    {
        /// What went wrong, as the underlying error's own `Display` renders it.
        cause: String,
    },
}

impl RenderResponse
{
    pub(crate) fn From(result: Result<RenderAnswer, RenderRefusal>) -> Self
    {
        return match result
        {
            Ok(answer) => Self::Placed { id: answer.id, body: answer.body, sidecar: answer.sidecar, stamp: answer.stamp },
            Err(RenderRefusal::NoSuchProfile { requested, known }) => Self::NoSuchProfile { requested, known },
            Err(RenderRefusal::Unwritable { path, error }) =>
            {
                Self::Unwritable { path: path.display().to_string(), cause: error.to_string() }
            }
            Err(RenderRefusal::Project(error)) => Self::Unreadable { cause: error.to_string() },
            Err(RenderRefusal::Store(error)) => Self::Unreadable { cause: error.to_string() },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::{Area, Assert_Round_Trips_As_Json, Unique_Scratch_Directory};

    /// The area every scratch path in this module is named under.
    const RENDER_AREA: Area = Area("spec-render");

    /// Builds from the embedded governing records alone, so a test naming it needs no corpus
    /// -- the same profile `nomos_spec_orchestration`'s own `tests.rs` and
    /// `crates/host/nomos-cli/tests/read_surface.rs` both use for the same reason.
    const EMBEDDED_PROFILE: &str = "domain-specification";

    #[test]
    fn Test_Handle_Spec_Render_Should_Place_Both_Files_On_Disk()
    {
        let into = Unique_Scratch_Directory(RENDER_AREA, "render")
            .expect("the temp directory is writable and this call's own name is fresh");
        let request = RenderRequest { profile: EMBEDDED_PROFILE.to_owned(), into: into.clone(), subject: None };

        let response = Handle_Spec_Render(&request);

        let RenderResponse::Placed { id, body, sidecar, .. } = response
        else
        {
            // This profile builds from embedded governing records alone, with a fresh scratch
            // directory to write into, so reaching any other variant here means the render
            // path itself regressed, not a condition this test should assert around.
            panic!("{EMBEDDED_PROFILE} builds from the embedded governing records alone: {response:?}");
        };
        assert_eq!(id, EMBEDDED_PROFILE);
        assert!(body.starts_with(&into), "{}", body.display());
        let written = std::fs::read_to_string(&body).expect("Render actually wrote the body to disk");
        assert!(!written.is_empty());
        std::fs::read_to_string(&sidecar).expect("Render actually wrote the sidecar to disk");
    }

    #[test]
    fn Test_An_Unknown_Profile_Should_Report_No_Such_Profile()
    {
        let request = RenderRequest {
            profile: "definitely-not-a-real-profile".to_owned(),
            into: Unique_Scratch_Directory(RENDER_AREA, "render-unknown")
                .expect("the temp directory is writable and this call's own name is fresh"),
            subject: None,
        };

        let response = Handle_Spec_Render(&request);

        assert!(matches!(response, RenderResponse::NoSuchProfile { .. }), "{response:?}");
    }

    #[test]
    fn Test_From_Should_Round_Trip_As_Json()
    {
        let request = RenderRequest {
            profile: EMBEDDED_PROFILE.to_owned(),
            into: Unique_Scratch_Directory(RENDER_AREA, "render-json")
                .expect("the temp directory is writable and this call's own name is fresh"),
            subject: None,
        };

        let response = Handle_Spec_Render(&request);

        Assert_Round_Trips_As_Json(&response, "placed")
            .expect("a placed response serializes and parses back as a tagged object");
    }
}
