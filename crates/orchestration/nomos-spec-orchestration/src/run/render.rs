//! Resolving `nomos spec render`'s request against an assembled store, and placing the
//! projection it builds.
//!
//! # Why this is generic over [`FileSystem`], and `corpus::Assemble_Corpus`'s walk is not
//!
//! Writing a projection's body and its sidecar is two single-file writes at paths this crate
//! already knows -- exactly [`FileSystem::Replace_Atomically`]'s shape, the same port
//! `nomos_work_orchestration::Run` already threads through for the ledger. It is not a
//! directory listing (no port exists for that, which is why [`crate::corpus::Assemble_Corpus`]'s own
//! walk and `nomos-cli::work::Published_Records` both stay client-side), and it is not read
//! from a caller-named arbitrary path outside this crate's control (which is why `Preview`'s
//! and `Commit`'s `--from` file, staged by an author, is deliberately left as future work
//! rather than folded into this decision by default).
//!
//! Going through `FileSystem` here is also a strict improvement over what it replaces:
//! `nomos-cli::spec::verb::render`'s old `Placed` wrote with `std::fs::create_dir_all` and
//! `std::fs::write` -- a truncating write with a window in which the file is empty or
//! half-written, precisely the defect `FileSystem::Replace_Atomically`'s own documentation
//! calls out. A crash between the body and the sidecar already left the pair mismatched
//! before this change (unchanged here, and not this item's problem to fix); a crash *during*
//! either half no longer can.

use nomos_platform::FileSystem;
use nomos_spec_project::{Build, Catalogue, Output, Profile};
use std::path::Path;

use crate::corpus::Assembly;
use crate::spec_outcome::{RenderAnswer, RenderRefusal};
use crate::request::RenderRequest;

/// Builds the profile `request` names and writes both halves of it under `request.into`.
///
/// Moved from `nomos-cli::spec::verb::render`'s own `Render`, `Declared` and
/// `Placed_Projection`, minus the writing of *text about* what happened: only the
/// projection's own two files are written here, and only through `filesystem`.
///
/// # Errors
///
/// Returns [`RenderRefusal::NoSuchProfile`] when the catalogue does not carry the named
/// profile, [`RenderRefusal::Project`] when resolving the subject or building the projection
/// fails, and [`RenderRefusal::Unwritable`] when a built projection could not be placed on
/// disk.
pub fn Rendered_Projection<Filesystem: FileSystem>(
    assembly: &Assembly,
    request: &RenderRequest,
    filesystem: &Filesystem,
) -> Result<RenderAnswer, RenderRefusal>
{
    let catalogue = Catalogue::Shipped().map_err(RenderRefusal::Project)?;
    let declared = Declared_Profile(&catalogue, request)?;
    let built = Build(&assembly.store, &declared).map_err(RenderRefusal::Project)?;

    return Placed_Projection(filesystem, &built, &declared.id, &request.into);
}

/// The shipped profile a run names, resolved against the subject it was given.
///
/// Resolved before the store is touched. A profile that names a subject and a run that does
/// not supply one disagree about what is being built, and the disagreement is answerable
/// without reading a single row.
fn Declared_Profile(catalogue: &Catalogue, request: &RenderRequest) -> Result<Profile, RenderRefusal>
{
    let declared = Resolved_Profile(catalogue, &request.profile)?;

    return declared.For(request.subject.as_deref()).map_err(RenderRefusal::Project);
}

/// The shipped profile that identifier names, or the refusal saying which ones exist.
fn Resolved_Profile<'a>(catalogue: &'a Catalogue, profile: &str) -> Result<&'a Profile, RenderRefusal>
{
    return catalogue.Named(profile).ok_or_else(|| {
        return RenderRefusal::NoSuchProfile {
            requested: profile.to_owned(),
            known: catalogue.Profiles().iter().map(|shipped| return shipped.id.clone()).collect(),
        };
    });
}

/// Both halves of a built projection, written where the run asked for them.
fn Placed_Projection<Filesystem: FileSystem>(
    filesystem: &Filesystem,
    built: &Output,
    id: &str,
    into: &Path,
) -> Result<RenderAnswer, RenderRefusal>
{
    let body_path = into.join(&built.path);
    let sidecar_path = into.join(&built.sidecar_path);
    let sidecar = built.Sidecar().map_err(RenderRefusal::Project)?;

    filesystem
        .Replace_Atomically(&body_path, &built.body)
        .map_err(|error| return RenderRefusal::Unwritable { path: body_path.clone(), error })?;
    filesystem
        .Replace_Atomically(&sidecar_path, &sidecar)
        .map_err(|error| return RenderRefusal::Unwritable { path: sidecar_path.clone(), error })?;

    return Ok(RenderAnswer {
        id: id.to_owned(),
        body: body_path,
        sidecar: sidecar_path,
        stamp: built.stamp.clone(),
    });
}

#[cfg(test)]
mod tests
{
    use super::{Rendered_Projection, RenderRequest};
    use crate::corpus::{Assemble_Corpus, CorpusRequest};
    use nomos_platform_std::StdFileSystem;

    const EMBEDDED_PROFILE: &str = "domain-specification";

    fn Scratch(name: &str) -> std::path::PathBuf
    {
        let root = std::env::temp_dir().join(format!("nomos-spec-orchestration-render-{name}-{}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a scratch build root");
        return root;
    }

    #[test]
    fn Test_Rendered_Projection_Should_Place_A_Projection_Built_With_No_Corpus()
    {
        let request = CorpusRequest { variable: "A_RENDER_TEST_CORPUS_VARIABLE".to_owned(), root: None, revision: "v14.36".to_owned() };
        let assembly = Assemble_Corpus(&request).expect("assembles from the embedded records alone");
        let into = Scratch("colocated");

        let answer = Rendered_Projection(&assembly, &RenderRequest { profile: EMBEDDED_PROFILE.to_owned(), into, subject: None }, &StdFileSystem)
            .expect("domain-specification builds from the embedded records alone");

        assert_eq!(answer.id, EMBEDDED_PROFILE);
        assert!(std::fs::read_to_string(&answer.body).is_ok(), "the body was written");
    }

    #[test]
    fn Test_Rendered_Projection_Should_Refuse_An_Unknown_Profile()
    {
        let request = CorpusRequest { variable: "A_RENDER_TEST_CORPUS_VARIABLE".to_owned(), root: None, revision: "v14.36".to_owned() };
        let assembly = Assemble_Corpus(&request).expect("assembles from the embedded records alone");
        let into = Scratch("unknown");

        let error = Rendered_Projection(&assembly, &RenderRequest { profile: "no-such-profile".to_owned(), into, subject: None }, &StdFileSystem)
            .expect_err("an unknown profile must refuse");

        assert!(matches!(error, super::RenderRefusal::NoSuchProfile { .. }), "{error:?}");
    }
}
