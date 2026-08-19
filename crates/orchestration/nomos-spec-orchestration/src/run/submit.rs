//! Resolving `nomos request submit`'s request against an assembled store: accepting the
//! submission through the one door `OD-SPEC-009` decided, and, when `--into` asks for one,
//! rendering it through [`Render`] itself rather than a build of Submit's own.
//!
//! Moved from `nomos-cli::request`'s own `Submit`, `Constructed`, `Accepted` and `Projected`,
//! minus the writing of text about what happened -- `OD-HOST-005` decided this verb's real
//! work is `nomos-spec-orchestration`'s to run, and this crate's own discipline is to return
//! typed data and place only the files a verb's own job is to write, never a line of
//! commentary. `Projected`'s three steps (resolve the shipped `subject-dossier` profile,
//! build it, place both halves) are exactly [`Render`]'s own job over a
//! [`crate::request::RenderRequest`] this function constructs rather than duplicates: the
//! subject-dossier profile addressed at the submission's own id, under the same `into` root.

use nomos_platform::FileSystem;
use nomos_spec_model::{FieldValue, Origin, Submission};
use nomos_spec_store::Accept_Submission;

use crate::corpus::Assembly;
use crate::outcome::{SubmitAnswer, SubmitRefusal};
use crate::request::{RenderRequest, SubmitRequest};
use crate::run::render::Render;

/// The shipped profile a submission's own render takes its one chance at a freshness proof
/// through.
///
/// Moved verbatim from `nomos-cli::request`'s own `DOSSIER`: `OD-SPEC-013` made a submission a
/// row in `nodes`, and `subject-dossier` already projects any node by identity -- the store
/// cannot tell a submission's node from any other, and adding a second profile that says the
/// same thing about the same table would be a second answer to what a subject is.
const DOSSIER: &str = "subject-dossier";

/// Accepts `request` against `assembly.store`, and places its `subject-dossier` projection
/// under `request.into` when one is asked for.
///
/// # Errors
///
/// Returns [`SubmitRefusal::Refused`] when the submission fails `OD-SPEC-010`'s rule set,
/// [`SubmitRefusal::Store`] when the store itself could not be used, and
/// [`SubmitRefusal::Written`] when the submission was accepted and its projection could not be
/// built or placed.
pub fn Submit<F: FileSystem>(
    assembly: &mut Assembly,
    request: &SubmitRequest,
    filesystem: &F,
) -> Result<SubmitAnswer, SubmitRefusal>
{
    let submission = Constructed(request);
    let uid = Accept_Submission(&mut assembly.store, &submission)?;

    let written = match &request.into
    {
        Some(into) =>
        {
            let render_request = RenderRequest {
                profile: DOSSIER.to_owned(),
                subject: Some(submission.id.clone()),
                into: into.clone(),
            };

            Some(Render(&*assembly, &render_request, filesystem).map_err(SubmitRefusal::Written)?)
        }
        None => None,
    };

    return Ok(SubmitAnswer { submission, uid, written });
}

/// The submission `request`'s arguments describe.
///
/// Every field value carries origin `submitted`, because it is exactly what was typed: this
/// verb supplies nothing of its own that lands in `values`. `state` and `form_contract_version`
/// are not values and carry no origin -- `OD-SPEC-013` keeps them columns rather than
/// attributed rows for that reason.
fn Constructed(request: &SubmitRequest) -> Submission
{
    return Submission {
        id: request.id.clone(),
        kind: request.kind,
        form_contract_version: request.contract_version,
        state: request.state,
        submitted_by: request.by.clone(),
        submitted_through: request.submitted_through.clone(),
        values: request
            .fields
            .iter()
            .map(|(field, value)| {
                return FieldValue {
                    field: field.clone(),
                    value: value.clone(),
                    origin: Origin::Submitted,
                };
            })
            .collect(),
        gaps: request.gaps.clone(),
    };
}
