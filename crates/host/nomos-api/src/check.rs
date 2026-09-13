//! [`Handle_Check_Run`] and its own [`CheckResponse`], paired in one file: the response
//! type exists only for this one handler, the same "handler beside its own response"
//! locality [`crate::response::gate_run_response`] and [`crate::correction`] both keep.
//!
//! `P62-API-CHECK-SEAM` gives this crate its first bare Check verb. `nomos-cli` exposes
//! `nomos check` as its own command, distinct from `nomos gate`, but before this file
//! existed this crate could only reach `nomos_check_orchestration::Run` transitively,
//! through `nomos_gate_orchestration::Run_Gate`, which always applies `GateCommand`'s
//! suppression, baseline and coverage policy on top of it. This calls `Run` directly, the
//! same policy-free shape `nomos-cli`'s own `nomos check` already has, walking `command.root`
//! and choosing a build variant the same way [`crate::correction`] does for correction --
//! with this crate's own [`composition::Host_Variant`] and [`sources::Walked_Sources`]
//! rather than sharing either with `nomos-cli`, since a walk and a build variant are each a
//! composition-root concern, `OD-HOST-002`'s own division.

use crate::{composition, sources};
use nomos_check_orchestration::{CheckCommand, CheckOutcome, Run, RunContext};
use nomos_contracts::Finding;
use nomos_composer_std::{ENVIRONMENT, FILE_SYSTEM, LAUNCHER};
use serde::Serialize;

/// Walks `command.root` and runs every registered rule over it -- `nomos-cli::check`'s own
/// "empty selects everything" default -- and hands back a JSON-serializable [`CheckResponse`].
#[must_use]
pub fn Handle_Check_Run(command: &CheckCommand) -> CheckResponse
{
    let Some(sources) = sources::Walked_Sources(&command.root)
    else
    {
        return CheckResponse::Unreadable;
    };

    if sources.is_empty()
    {
        return CheckResponse::NoSource;
    }

    let mut store = nomos_analysis::MemoryFactStore::New();
    let outcome = Run(
        &sources,
        RunContext {
            variant: composition::Host_Variant(),
            root: &command.root,
            launcher: &LAUNCHER,
            filesystem: &FILE_SYSTEM,
            environment: &ENVIRONMENT,
            workspace: &mut None,
            store: &mut store,
        },
        &[],
    );

    return CheckResponse::From(outcome);
}

/// A serializable twin of [`nomos_check_orchestration::CheckOutcome`].
///
/// A twin rather than a re-export because the type it mirrors does not derive `Serialize`,
/// for the reason [`crate::response`]'s own doc gives for [`crate::response::Disposition`].
/// Kept to the same variants, in the same order, so a mismatch between the two is a compile
/// error in [`CheckResponse::From`] rather than a silent divergence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "outcome")]
pub enum CheckResponse
{
    Unreadable,
    Contradictory
    {
        /// What went wrong, as `RegistryError`'s own `Debug` renders it -- it does not
        /// derive `Display`, the same reason `crates/host/nomos-cli/src/check/report.rs`'s
        /// own render prints `{error:?}` too.
        cause: String,
    },
    NoSource,
    NoFacts
    {
        files: usize,
    },
    Judged
    {
        findings: Vec<Finding>,
        examined: ExaminedResponse,
        claim: ClaimResponse,
    },
}

impl CheckResponse
{
    pub(crate) fn From(outcome: CheckOutcome) -> Self
    {
        return match outcome
        {
            CheckOutcome::Unreadable => Self::Unreadable,
            CheckOutcome::Contradictory(cause) => Self::Contradictory { cause: format!("{cause:?}") },
            CheckOutcome::NoSource => Self::NoSource,
            CheckOutcome::NoFacts { files } => Self::NoFacts { files },
            CheckOutcome::Judged { findings, examined, claim } => Self::Judged {
                findings,
                examined: ExaminedResponse::From(examined),
                claim: ClaimResponse::From(claim),
            },
        };
    }
}

/// A serializable twin of [`nomos_check_orchestration::Examined`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ExaminedResponse
{
    /// Files the walk read.
    pub files: usize,
    /// Files a syntax fact was materialized for.
    pub facts: usize,
}

impl ExaminedResponse
{
    fn From(examined: nomos_check_orchestration::Examined) -> Self
    {
        return Self { files: examined.files, facts: examined.facts };
    }
}

/// A serializable twin of [`nomos_check_orchestration::Claim`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimResponse
{
    Complete,
    Incomplete,
}

impl ClaimResponse
{
    fn From(claim: nomos_check_orchestration::Claim) -> Self
    {
        return match claim
        {
            nomos_check_orchestration::Claim::Complete => Self::Complete,
            nomos_check_orchestration::Claim::Incomplete => Self::Incomplete,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::path::PathBuf;

    fn Command_At(root: PathBuf) -> CheckCommand
    {
        return CheckCommand { root };
    }

    /// A real run over a fixture tree with a real blocking phantom claim reaches a real
    /// `Judged` outcome carrying that finding -- not `Unreadable` or `NoSource` -- proving
    /// this crate, not `nomos-cli`, can produce one.
    #[test]
    fn Test_Handle_Check_Run_Should_Judge_A_Real_Tree_And_Report_A_Real_Finding()
    {
        let root = std::env::temp_dir().join("nomos-api-check-run-judged");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("creates a fresh directory");
        std::fs::write(
            root.join("a.rs"),
            "/// A list.\n/// Mirrored by `Test_Api_Ghost`.\npub const TABLES: &[&str] = &[];\n",
        )
        .expect("writes a fixture whose stale mirror is one real blocking finding");

        let response = Handle_Check_Run(&Command_At(root.clone()));

        let _ignored = std::fs::remove_dir_all(&root);
        match &response
        {
            CheckResponse::Judged { findings, .. } =>
            {
                assert!(findings.iter().any(|finding| return finding.summary.contains("Test_Api_Ghost")), "{findings:?}");
            }
            other => panic!("expected Judged, got {other:?}"),
        }
    }

    /// An empty tree cannot be judged, the same distinction `CheckOutcome::NoSource` already
    /// keeps apart from a clean judged run.
    #[test]
    fn Test_An_Empty_Tree_Should_Report_No_Source_Found()
    {
        let root = std::env::temp_dir().join("nomos-api-check-run-empty-tree");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("creates an empty directory");

        let response = Handle_Check_Run(&Command_At(root.clone()));

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(response, CheckResponse::NoSource);
    }

    /// A root that does not exist is `Unreadable`, not `NoSource`: nothing was walked at all.
    #[test]
    fn Test_A_Missing_Root_Should_Report_Unreadable()
    {
        let root = std::env::temp_dir().join("nomos-api-check-run-missing-root-does-not-exist");
        let _ignored = std::fs::remove_dir_all(&root);

        let response = Handle_Check_Run(&Command_At(root));

        assert_eq!(response, CheckResponse::Unreadable);
    }

    /// The whole point of this crate: the response a real run produces is valid JSON, and
    /// its outcome round-trips through `serde_json` under the field name a wire caller
    /// would actually read.
    #[test]
    fn Test_From_Should_Produce_A_Response_That_Round_Trips_As_Json()
    {
        let response = CheckResponse::NoSource;

        let json = serde_json::to_string(&response).expect("a CheckResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");

        assert_eq!(parsed.get("outcome").expect("a serialized CheckResponse always has this field"), "no_source", "{json}");
    }
}
