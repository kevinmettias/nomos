//! [`AgentJudgeRoleResponse`], what `Handle_Agent_Judge_Role` answers.

use super::AgentDispatchResponse;
use serde::Serialize;

/// What [`super::Handle_Agent_Judge_Role`] answers -- `NoDeclaredRoleOrSurface` and `NoFinding`
/// are the two composition-root-level answers this handler gives before
/// `Run_Agent_Judgment` is ever called, the identical two refusals
/// `nomos-cli`'s own `judge_role.rs` reports as `ExitCode::NotFound`, kept apart here
/// because a wire caller reading JSON needs to know which one it got even though this
/// binary's own exit code does not distinguish them.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "outcome")]
pub enum AgentJudgeRoleResponse
{
    /// `crate_name` names no row in `root`'s `README.md` band table, or `root` has no
    /// committed surface snapshot for it.
    NoDeclaredRoleOrSurface,
    /// `Check_Declared_Role_Matches_Surface` produced no finding for its own subject --
    /// unreached by that rule's own current implementation (it maps one subject to
    /// exactly one finding), kept as a real, checked case rather than an `unwrap`, the
    /// same defensive shape `nomos-cli`'s own `Judged_Finding` already holds.
    NoFinding,
    Dispatched
    {
        dispatch: AgentDispatchResponse
    },
}
