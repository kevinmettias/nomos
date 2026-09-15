//! [`Handle_Correction_Run`] and its own [`CorrectionResponse`], paired in one file: the
//! response type exists only for this one handler, the same "handler beside its own
//! response" locality [`crate::response::gate_run_response`] keeps.
//!
//! `P40-CORRECTIONS-CANONICAL-SEAM` gives this crate its first correction verb: a deliberate
//! twin of `nomos-cli`'s own `correct.rs`, calling the identical
//! `nomos_correction_orchestration::Run_Correction` seam that module now renders text over,
//! using this crate's own `composition::Host_Variant` and `sources::Walked_Sources` rather
//! than sharing either with `nomos-cli` -- a walk and a build variant are each a
//! composition-root concern, `OD-HOST-002`'s own division.

use crate::{composition, sources};
use nomos_correction_orchestration::{CorrectionCommand, CorrectionEnvironment, CorrectionOutcome, Run_Correction};
use nomos_composer_std::{ENVIRONMENT, FILE_SYSTEM, LAUNCHER};
use serde::Serialize;

/// Walks `command.root` and runs the correction exactly as `nomos correct phantom-mirrors`
/// would, and hands back a JSON-serializable [`CorrectionResponse`].
#[must_use]
pub fn Handle_Correction_Run(command: &CorrectionCommand) -> CorrectionResponse
{
    let walked = sources::Walked_Sources(&command.root);
    let outcome = Run_Correction(
        walked,
        CorrectionEnvironment { variant: composition::Host_Variant(), launcher: &LAUNCHER, filesystem: &FILE_SYSTEM, environment: &ENVIRONMENT },
        command,
    );

    return CorrectionResponse::From(outcome);
}

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

impl CorrectionResponse
{
    pub(crate) fn From(outcome: CorrectionOutcome) -> Self
    {
        return match outcome
        {
            CorrectionOutcome::UnreadableRoot => Self::UnreadableRoot,
            CorrectionOutcome::NoSourceFound => Self::NoSourceFound,
            CorrectionOutcome::UnreadableWorkspaceState => Self::UnreadableWorkspaceState,
            CorrectionOutcome::ContradictoryRegistry(reason) => Self::ContradictoryRegistry { reason },
            CorrectionOutcome::NoFactsMaterialized(files) => Self::NoFactsMaterialized { files },
            CorrectionOutcome::Clean => Self::Clean,
            CorrectionOutcome::Refused(reason) => Self::Refused { reason },
            CorrectionOutcome::Staged { path, summary, preview } => Self::Staged { path, summary, preview: String::from_utf8_lossy(&preview).into_owned() },
            CorrectionOutcome::Committed { path, summary, preview, base, after_snapshot } => Self::Committed {
                path,
                summary,
                preview: String::from_utf8_lossy(&preview).into_owned(),
                base,
                after: after_snapshot,
            },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::path::PathBuf;

    /// A real run over a fixture tree with a real blocking phantom claim reaches a real
    /// `Staged` outcome -- not `UnreadableRoot` or `NoSourceFound`, proving this crate, not
    /// `nomos-cli`, can produce one.
    #[test]
    fn Test_Handle_Correction_Run_Should_Stage_A_Real_Phantom_Claim()
    {
        let root = Tree_With_A_Phantom_Mirror("nomos-api-correction-run-stage");

        let command = Command_At(root.clone());
        let response = Handle_Correction_Run(&command);
        let on_disk = std::fs::read_to_string(root.join("a.rs")).expect("still readable");

        let _ignored = std::fs::remove_dir_all(&root);
        match &response
        {
            CorrectionResponse::Staged { path, summary, .. } =>
            {
                assert_eq!(path, "a.rs");
                assert!(summary.contains("Test_Api_Ghost"), "{summary}");
            }
            other => panic!("expected Staged, got {other:?}"),
        }
        assert!(on_disk.contains("Mirrored by"), "a dry run must not touch the file");
    }

    /// A fresh tree under `name` holding one source file whose doc comment names a mirror
    /// that does not exist -- one real blocking `check-doc-references` finding.
    fn Tree_With_A_Phantom_Mirror(name: &str) -> PathBuf
    {
        let root = std::env::temp_dir().join(name);
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("creates a fresh directory");
        std::fs::write(
            root.join("a.rs"),
            "/// A list.\n/// Mirrored by `Test_Api_Ghost`.\npub const TABLES: &[&str] = &[];\n",
        )
        .expect("writes a fixture whose stale mirror is one real blocking finding");

        return root;
    }

    /// An empty tree cannot be judged, the same distinction
    /// `nomos_check_orchestration::CheckOutcome::NoSource` already keeps apart from a clean
    /// judged run.
    #[test]
    fn Test_An_Empty_Tree_Should_Report_No_Source_Found()
    {
        let root = std::env::temp_dir().join("nomos-api-correction-run-empty-tree");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("creates an empty directory");

        let command = Command_At(root.clone());
        let response = Handle_Correction_Run(&command);

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(response, CorrectionResponse::NoSourceFound);
    }

    /// The whole point of this crate: the response a real run produces is valid JSON, and
    /// its outcome round-trips through `serde_json` under the field name a wire caller
    /// would actually read.
    #[test]
    fn Test_From_Should_Produce_A_Response_That_Round_Trips_As_Json()
    {
        let response = CorrectionResponse::Clean;

        let json = serde_json::to_string(&response)
            .expect("a derived Serialize over owned data has nothing to refuse");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");

        assert_eq!(parsed.get("outcome").expect("a serialized CorrectionResponse always has this field"), "clean", "{json}");
    }

    /// A `CorrectionCommand` over `root`, left a dry run -- nothing here commits, so a test
    /// that reads the tree back afterwards reads it exactly as it was.
    fn Command_At(root: PathBuf) -> CorrectionCommand
    {
        return CorrectionCommand { root, commit: false };
    }
}
