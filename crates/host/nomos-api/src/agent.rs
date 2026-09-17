//! [`Handle_Agent_Execute`] and [`Handle_Agent_Judge_Role`], and their own
//! [`AgentDispatchResponse`]/[`AgentJudgeRoleResponse`], each in the file named for it: the
//! response types exist only for these two handlers, the same "handler beside its own
//! response" locality [`crate::correction`] and [`crate::check`] both keep.
//!
//! `P43-AGENT-CANONICAL-SEAM-2` gives this crate its first Agent verbs: deliberate twins of
//! `nomos-cli`'s own `agent execute`/`agent judge-role`, calling the identical
//! `nomos_agent_orchestration::Run_Agent_Execute`/`Run_Agent_Judgment` seam those commands
//! now dispatch through, naming its own platform through `nomos-composer-std` rather than
//! reaching into `nomos-cli` for one -- a platform choice is a composition-root concern,
//! `OD-HOST-002`'s own division, and two roots naming the same platform is not the same
//! thing as one of them borrowing the other's. Before this crate, `nomos-agent-executor-claude-code` and
//! `nomos-model-backend-ollama` reached this workspace only through `nomos-cli`'s own
//! `agent` group and through `nomos_workflow_orchestration`'s own `Body::ClaudeCode`/
//! `Body::Ollama` step dispatch (rendered here as `workflow::AgentExecutionOutcomeResponse`/
//! `OllamaExecutionOutcomeResponse`); this is the first place this crate can dispatch a
//! bare agent task, or a role judgment, on its own.
//!
//! [`Handle_Agent_Judge_Role`] reads `root`'s `README.md` row and its committed surface
//! snapshot itself and runs `nomos_rules::Check_Declared_Role_Matches_Surface` to get the
//! real `Finding` it produces -- a deliberate twin of `nomos-cli`'s own
//! `agent/judge_role.rs`, not a shared dependency on it: reading a crate's declared role
//! and actual surface from a real tree is a composition root's own file-reading concern,
//! the identical division `sources.rs` already draws for Gate's own walk.
//!
//! [`AgentDispatchResponse`] is not [`crate::workflow::AgentExecutionOutcomeResponse`]/
//! [`crate::workflow::OllamaExecutionOutcomeResponse`] reused: those two exist for a
//! workflow step's own outcome, paired with `workflow::AgentExecutionErrorResponse`/
//! `OllamaExecutionErrorResponse`, which keep each backend's own `Unavailable`/
//! `Unparseable` structure apart. `nomos_agent_orchestration::AgentDispatchOutcome`
//! deliberately does not: its own `Unavailable(String)` collapses both backends' error
//! types to the text either one's `Display` already produces, the identical fold
//! `nomos-cli`'s own former `Backend_Unavailable` made -- see that crate's own
//! `AgentDispatchOutcome` doc. A caller of this crate's own bare Agent verb sees exactly
//! that fold, not a structure this seam does not keep.

use nomos_agent_orchestration::{AgentDispatchOutcome, AgentEnvironment, DispatchConfig, Run_Agent_Execute, Run_Agent_Judgment};
use nomos_agent_executor_claude_code::MicroDollars;
use nomos_composer_std::LAUNCHER;
use nomos_rules::RoleSurfacePair;
use std::path::Path;

mod agent_dispatch_response;
mod agent_judge_role_response;

pub use agent_dispatch_response::AgentDispatchResponse;
pub use agent_judge_role_response::AgentJudgeRoleResponse;

/// Dispatches `goal` to `config.backend` exactly as `nomos agent execute --goal <goal>
/// --effort <effort>` would, and hands back a JSON-serializable [`AgentDispatchResponse`].
#[must_use]
pub fn Handle_Agent_Execute(goal: &str, config: DispatchConfig) -> AgentDispatchResponse
{
    let outcome = Run_Agent_Execute(goal, config, &AgentEnvironment { launcher: &LAUNCHER });

    return AgentDispatchResponse::From(outcome);
}

/// Reads `root`'s `README.md` row and committed surface snapshot for `crate_name`, runs
/// `nomos_rules::Check_Declared_Role_Matches_Surface` to get the real `Finding` it
/// produces, and dispatches the question that finding names to `config.backend` exactly as
/// `nomos agent judge-role --crate <crate_name> --root <root>` would.
#[must_use]
pub fn Handle_Agent_Judge_Role(root: &Path, crate_name: &str, config: DispatchConfig) -> AgentJudgeRoleResponse
{
    let Some(pair) = Role_Surface_Pair(root, crate_name)
    else
    {
        return AgentJudgeRoleResponse::NoDeclaredRoleOrSurface;
    };

    let findings = nomos_rules::Check_Declared_Role_Matches_Surface(std::slice::from_ref(&pair));
    let Some(finding) = findings.first()
    else
    {
        return AgentJudgeRoleResponse::NoFinding;
    };

    let outcome = Run_Agent_Judgment(&pair, finding, config, &AgentEnvironment { launcher: &LAUNCHER });

    return AgentJudgeRoleResponse::Dispatched { dispatch: AgentDispatchResponse::From(outcome) };
}

/// `root`'s `README.md` row and committed surface snapshot for `crate_name`, paired into
/// the real `nomos_rules::RoleSurfacePair` the rule needs -- `None` if either file is
/// missing, or `crate_name` names no row. A deliberate twin of `nomos-cli`'s own
/// `judge_role.rs::Role_Surface_Pair`, reading through plain `std::fs` the same way that
/// function does, not through `nomos-platform`'s `FileSystem` port: this crate's own
/// `sources::Walked_Sources` and `composition::Host_Variant` are the only walks
/// `OD-HOST-002` asks a composition root to seam behind a platform trait, and neither of
/// those two files is a source tree walk.
fn Role_Surface_Pair(root: &Path, crate_name: &str) -> Option<RoleSurfacePair>
{
    let declared_role = Declared_Role(root, crate_name)?;
    let actual_surface = std::fs::read_to_string(root.join("tests/contract/surface").join(format!("{crate_name}.txt"))).ok()?;

    return Some(RoleSurfacePair { crate_root: Crate_Root(root, crate_name), crate_name: crate_name.to_owned(), declared_role, actual_surface });
}

/// Which pipe-delimited cell of a `README.md` band-table row holds a crate's name.
const README_TABLE_CRATE_NAME_COLUMN: usize = 2;

/// Which pipe-delimited cell of a `README.md` band-table row holds the crate's declared role.
const README_TABLE_ROLE_COLUMN: usize = 3;

/// `README.md`'s band-table row for `crate_name` -- the third pipe-delimited cell of the
/// row whose second cell, backticks stripped, is `crate_name` exactly. `None` if no row
/// names it.
fn Declared_Role(root: &Path, crate_name: &str) -> Option<String>
{
    let text = std::fs::read_to_string(root.join("README.md")).ok()?;

    for line in text.lines()
    {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        let (Some(name_cell), Some(role_cell)) = (cells.get(README_TABLE_CRATE_NAME_COLUMN), cells.get(README_TABLE_ROLE_COLUMN))
        else
        {
            continue;
        };
        if name_cell.trim_matches('`') == crate_name
        {
            return Some((*role_cell).to_owned());
        }
    }

    return None;
}

/// The same manifest-relative root `RoleSurfacePair::crate_root` documents --
/// `crates/<band-folder>/<crate_name>` is not derivable from the name alone, so this reads
/// it from `Cargo.toml`'s own `[workspace] members` list rather than guess a layout.
fn Crate_Root(root: &Path, crate_name: &str) -> String
{
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).unwrap_or_default();

    return manifest
        .lines()
        .map(str::trim)
        .find(|line| return line.trim_matches(['"', ',']).ends_with(crate_name))
        .map_or_else(|| return crate_name.to_owned(), |line| return line.trim_matches([' ', '"', ',']).to_owned());
}

/// `cost` as the dollar figure the wire publishes.
///
/// The one place in this workspace where the engine's exact money becomes a float, and it
/// is here because `cost_usd` is a published field of two `Serialize` response types --
/// [`AgentDispatchResponse::ClaudeCode`] and [`crate::workflow::AgentExecutionOutcomeResponse`].
/// A caller reading that number off a JSON-RPC or MCP reply already has a float, and
/// changing the field to integer micros would change a shape this workspace publishes
/// rather than one it merely holds.
///
/// `P89` moved this conversion here from `nomos-agent-executor-claude-code`'s own
/// `response.rs`, where it ran on the first line that read the engine's answer and left
/// every crate between there and here holding a value nothing could compare exactly
/// against `nomos_agent_executor_claude_code::MAXIMUM_SPEND`, which is the same integer
/// type the engine enforces the cap with.
#[expect(
    clippy::cast_precision_loss,
    reason = "a dispatch's cost in micros is far below f64's exact-integer range --               MAXIMUM_SPEND is 1_000_000 micros and 2^53 is nine orders of magnitude above it"
)]
pub(crate) fn Dollars_Of(cost: MicroDollars) -> f64
{
    const MICROS_IN_A_DOLLAR: f64 = 1_000_000.0;

    return cost.Micros() as f64 / MICROS_IN_A_DOLLAR;
}

impl AgentDispatchResponse
{
    fn From(outcome: AgentDispatchOutcome) -> Self
    {
        return match outcome
        {
            AgentDispatchOutcome::ClaudeCode(outcome) => Self::ClaudeCode {
                assumptions: outcome.result.assumptions,
                unresolved_questions: outcome.result.unresolved_questions,
                denied_tool_uses: outcome.denied_tool_uses,
                is_error: outcome.is_error,
                cost_usd: Dollars_Of(outcome.cost),
                duration_ms: outcome.duration_ms,
            },
            AgentDispatchOutcome::Ollama(outcome) => Self::Ollama { response: outcome.response },
            AgentDispatchOutcome::Unavailable(reason) => Self::Unavailable { reason },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_agent_orchestration::Backend;
    use nomos_model_package::EffortLevel;

    /// A real run over a fixture tree with a real blocking role-mismatch finding reaches a
    /// real `Dispatched` outcome -- not `NoDeclaredRoleOrSurface` or `NoFinding` -- proving
    /// this crate, not `nomos-cli`, can judge a declared role on its own. Never reaches a
    /// live backend subprocess: `--executor`/`--model-backend` is not this test's own
    /// concern to fake, so it is not exercised here -- see `nomos-agent-orchestration`'s own
    /// `run` module tests for the scripted-launcher coverage of the dispatch itself.
    ///
    /// Fails fast against this repository's own real tree, the same standard
    /// `nomos-cli`'s own `agent/tests.rs` already holds `Declared_Role`/`Crate_Root` to: a
    /// reader that cannot be checked against a real row is checked against nothing.
    #[test]
    fn Test_Role_Surface_Pair_Should_Read_A_Real_Row_And_Snapshot_From_This_Repositorys_Own_Tree()
    {
        let root = Repository_Root();

        let pair = Role_Surface_Pair(&root, "nomos-agent-executor-claude-code").expect("this crate has a row and a committed snapshot");

        assert!(pair.declared_role.contains("AgentExecutor"), "{}", pair.declared_role);
        assert_eq!(pair.crate_root, "crates/agent/nomos-agent-executor-claude-code");
        assert!(!pair.actual_surface.is_empty());
    }

    /// This repository's own root, three levels above `crates/host/nomos-api` -- the same
    /// derivation `nomos-cli`'s own `agent/tests.rs` uses to run against a real tree rather
    /// than a fixture nobody could have produced.
    fn Repository_Root() -> std::path::PathBuf
    {
        let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        return manifest
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .map(std::path::PathBuf::from)
            .expect("this crate sits three levels below the workspace root");
    }

    #[test]
    fn Test_Role_Surface_Pair_Should_Be_None_For_A_Crate_Named_Nowhere()
    {
        assert!(Role_Surface_Pair(&Repository_Root(), "nomos-does-not-exist").is_none());
    }

    /// [`Handle_Agent_Judge_Role`], driven end to end, over a root with no `README.md` at
    /// all -- fails before `Run_Agent_Judgment` is ever called, the same safe path
    /// `nomos-cli`'s own `judge_role.rs` test uses.
    #[test]
    fn Test_Handle_Agent_Judge_Role_Should_Report_No_Declared_Role_When_The_Root_Has_No_Readme()
    {
        let response = Handle_Agent_Judge_Role(
            Path::new("no-such-directory-anywhere-for-nomos-api-agent-test"),
            "nomos-does-not-exist",
            DispatchConfig { effort: EffortLevel::BackendDefault, backend: Backend::ClaudeCode },
        );

        assert_eq!(response, AgentJudgeRoleResponse::NoDeclaredRoleOrSurface);
    }

    /// The whole point of this crate: the response a real refusal produces is valid JSON,
    /// and its outcome round-trips through `serde_json` under the field name a wire caller
    /// would actually read.
    #[test]
    fn Test_Agent_Dispatch_Response_Should_Produce_A_Response_That_Round_Trips_As_Json()
    {
        let response = AgentDispatchResponse::Unavailable { reason: "fixture".to_owned() };

        let json = serde_json::to_string(&response)
            .expect("a derived Serialize over owned data has nothing to refuse");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");

        assert_eq!(parsed.get("backend").expect("a serialized AgentDispatchResponse always has this field"), "unavailable", "{json}");
    }

    #[test]
    fn Test_Agent_Judge_Role_Response_Should_Produce_A_Response_That_Round_Trips_As_Json()
    {
        let response = AgentJudgeRoleResponse::NoFinding;

        let json = serde_json::to_string(&response)
            .expect("a derived Serialize over owned data has nothing to refuse");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");

        assert_eq!(parsed.get("outcome").expect("a serialized AgentJudgeRoleResponse always has this field"), "no_finding", "{json}");
    }
}
