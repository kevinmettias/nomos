//! [`CorrectionResponse`], what [`super::Handle_Correction_Run`] answers.

use serde::Serialize;

/// A serializable twin of [`nomos_correction_orchestration::CorrectionOutcome`].
///
/// A twin rather than a re-export because the type it mirrors does not derive
/// `Serialize`, for the reason `crate::response`'s own doc gives for `Disposition`; kept
/// to the same variants, in the same order, so a mismatch between the two is a compile
/// error in [`CorrectionResponse::From`] rather than a silent divergence. `preview` is
/// rendered lossily to a `String` rather than carried as `Vec<u8>`: every real preview
/// this seam produces is `nomos_corrections::Preview`'s own UTF-8 rendering, and a wire
/// caller reading JSON has no use for a byte array it would only decode back to text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "outcome")]
pub enum CorrectionResponse
{
    UnreadableRoot,
    NoSourceFound,
    UnreadableWorkspaceState,
    ContradictoryRegistry
    {
        reason: String,
    },
    NoFactsMaterialized
    {
        files: usize,
    },
    Clean,
    Refused
    {
        reason: String,
    },
    Staged
    {
        path: String, summary: String, preview: String,
    },
    Committed
    {
        path: String, summary: String, preview: String, base: String, after: String,
    },
}
