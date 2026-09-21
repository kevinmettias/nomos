//! `judge-role` -- reading a crate's declared role and actual surface, judging them with
//! `nomos_rules::Check_Declared_Role_Matches_Surface`, and dispatching the finding it
//! produces through `nomos-agent-orchestration`'s own [`Run_Agent_Judgment`].
//!
//! `P43-AGENT-CANONICAL-SEAM-2` moved this module's own former `Judgment_Task` -- building
//! the `TaskEnvelope` that carries the rule's own question -- into that crate, alongside
//! `Run_Agent_Execute`. What stays here is everything `OD-HOST-002` keeps at a composition
//! root: reading `root`'s `README.md` row and its committed surface snapshot, and running
//! the rule itself to get the real `Finding` it produces. `Run_Agent_Judgment` is handed
//! both already built, the identical "already walked" contract
//! `nomos_correction_orchestration::Run_Correction` holds for the source it is given.

use super::{DispatchConfig, ExitCode};
use nomos_contracts::Finding;
use nomos_rules::RoleSurfacePair;
use std::path::Path;

/// `--crate` and `--root` together -- the subject `judge-role` was asked to judge, as
/// distinct from `DispatchConfig`'s question of how to ask it. `pub(super)` rather than
/// private: [`Judge_Role`] takes this directly now, grouping its own former `crate_name`
/// and `root` parameters into the same value this module already builds them into
/// internally, so [`super::Run`] constructs it at the one real call site.
#[derive(Clone, Copy)]
pub(super) struct RoleRequest<'a>
{
    pub(super) crate_name: &'a str,
    pub(super) root: &'a Path,
}

/// Reads `root`'s `README.md` and `root`'s committed surface snapshot for `crate_name`,
/// builds the real `nomos_rules::RoleSurfacePair` `Check_Declared_Role_Matches_Surface`
/// would be handed, runs that rule to get the real `Finding` it produces, and dispatches
/// the question that finding names — never its own guess — to Claude Code.
pub(super) fn Judge_Role(
    request: RoleRequest<'_>,
    requested: super::Requested_Dispatch,
    output: &mut impl std::io::Write,
    notes: &mut impl std::io::Write,
) -> ExitCode
{
    use nomos_agent_orchestration::{AgentEnvironment, Run_Agent_Judgment};
    use nomos_composer_std::LAUNCHER;

    let pair = match Role_Surface_Pair(request, notes)
    {
        Ok(pair) => pair,
        Err(code) => return code,
    };

    let finding = match Judged_Finding(&pair, notes)
    {
        Ok(finding) => finding,
        Err(code) => return code,
    };

    let outcome = Run_Agent_Judgment(&pair, &finding, &requested.Selection(), &AgentEnvironment { launcher: &LAUNCHER });

    return super::dispatch::Rendered_Dispatch_Outcome(&outcome, output, notes);
}

/// `request`'s declared role and actual surface, read and paired -- [`Judge_Role`]'s own
/// first two steps, named so its body reads as "build the pair, judge it, dispatch it."
fn Role_Surface_Pair(request: RoleRequest<'_>, notes: &mut impl std::io::Write) -> Result<RoleSurfacePair, ExitCode>
{
    let declared_role = Resolve_Declared_Role(request.root, request.crate_name, notes)?;
    let actual_surface = Read_Surface_Snapshot(request.root, request.crate_name, notes)?;

    return Ok(RoleSurfacePair {
        crate_root: Crate_Root(request.root, request.crate_name),
        crate_name: request.crate_name.to_owned(),
        declared_role,
        actual_surface,
    });
}

/// `declared_role`'s own row, or `NotFound` noted against `crate_name`'s absence from
/// `root`'s band table.
fn Resolve_Declared_Role(root: &Path, crate_name: &str, notes: &mut impl std::io::Write) -> Result<String, ExitCode>
{
    let Some(declared_role) = Declared_Role(root, crate_name)
    else
    {
        let _ = writeln!(notes, "`{crate_name}` names no row in {}'s band table", root.join("README.md").display());
        return Err(ExitCode::NotFound);
    };

    return Ok(declared_role);
}

/// `crate_name`'s committed `tests/contract/surface` snapshot, or `NotFound` noted against
/// its absence.
fn Read_Surface_Snapshot(root: &Path, crate_name: &str, notes: &mut impl std::io::Write) -> Result<String, ExitCode>
{
    let surface_path = root.join("tests/contract/surface").join(format!("{crate_name}.txt"));
    let Ok(actual_surface) = std::fs::read_to_string(&surface_path)
    else
    {
        let _ = writeln!(notes, "no committed surface snapshot at {}", surface_path.display());
        return Err(ExitCode::NotFound);
    };

    return Ok(actual_surface);
}

/// Runs `Check_Declared_Role_Matches_Surface` over `pair` and takes its own finding, or
/// `NotFound` noted when the rule produced none.
fn Judged_Finding(pair: &RoleSurfacePair, notes: &mut impl std::io::Write) -> Result<Finding, ExitCode>
{
    let findings = nomos_rules::Check_Declared_Role_Matches_Surface(std::slice::from_ref(pair));
    let Some(finding) = findings.first()
    else
    {
        let _ = writeln!(notes, "the rule produced no finding for its own subject");
        return Err(ExitCode::NotFound);
    };

    return Ok(finding.clone());
}

/// Which pipe-delimited cell of a `README.md` band-table row holds a crate's name.
const README_TABLE_CRATE_NAME_COLUMN: usize = 2;

/// Which pipe-delimited cell of a `README.md` band-table row holds the crate's declared role.
const README_TABLE_ROLE_COLUMN: usize = 3;

/// `README.md`'s band-table row for `crate_name` — the third pipe-delimited cell of the
/// row whose second cell, backticks stripped, is `crate_name` exactly. `None` if no row
/// names it.
pub(super) fn Declared_Role(root: &Path, crate_name: &str) -> Option<String>
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

/// The same manifest-relative root `RoleSurfacePair::crate_root` documents —
/// `crates/<band-folder>/<crate_name>` is not derivable from the name alone, so this reads
/// it from `Cargo.toml`'s own `[workspace] members` list rather than guess a layout.
pub(super) fn Crate_Root(root: &Path, crate_name: &str) -> String
{
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).unwrap_or_default();

    return manifest
        .lines()
        .map(str::trim)
        .find(|line| return line.trim_matches(['"', ',']).ends_with(crate_name))
        .map_or_else(|| return crate_name.to_owned(), |line| return line.trim_matches([' ', '"', ',']).to_owned());
}

#[cfg(test)]
mod tests
{
    use super::*;
    use super::super::Backend;

    /// `Judge_Role`'s own first step, `Role_Surface_Pair`, fails at `Resolve_Declared_Role`
    /// before this ever reaches `Run_Agent_Judgment` -- a root with no `README.md` at all
    /// names no row for any crate. This drives the real, top-level function end to end
    /// without ever touching a live backend subprocess, the same way `Run`'s own judge-role
    /// test (in
    /// `agent/tests.rs`) stays safe by failing before dispatch.
    #[test]
    fn Test_Judge_Role_Should_Report_Not_Found_When_The_Root_Has_No_Readme()
    {
        let request = RoleRequest {
            crate_name: "nomos-does-not-exist",
            root: Path::new("no-such-directory-anywhere-for-judge-role-test"),
        };
        let requested = super::super::Requested(nomos_model_package::EffortLevel::BackendDefault, None);
        let mut output = Vec::new();
        let mut notes = Vec::new();

        let code = Judge_Role(request, requested, &mut output, &mut notes);

        assert_eq!(code, ExitCode::NotFound);
        assert!(
            String::from_utf8_lossy(&notes).contains("names no row"),
            "{}",
            String::from_utf8_lossy(&notes)
        );
    }

    /// The pipe-delimited band-table row this parses directly, rather than through
    /// `Judge_Role`'s own end-to-end path above.
    #[test]
    fn Test_Declared_Role_Should_Read_The_Crates_Own_Table_Row()
    {
        let directory = std::env::temp_dir().join("judge-role-declared-role-test");
        std::fs::create_dir_all(&directory).expect("creates a scratch directory");
        std::fs::write(
            directory.join("README.md"),
            "| Band | Crate | Role |\n|---|---|---|\n| 1 | `nomos-example` | Provider |\n",
        )
        .expect("writes a fixture README");

        assert_eq!(Declared_Role(&directory, "nomos-example"), Some("Provider".to_owned()));
        assert_eq!(Declared_Role(&directory, "nomos-not-in-the-table"), None);
    }

    /// The `Cargo.toml` member line this parses directly, rather than through `Judge_Role`'s
    /// own end-to-end path above.
    #[test]
    fn Test_Crate_Root_Should_Read_The_Members_Manifest_Relative_Path()
    {
        let directory = std::env::temp_dir().join("judge-role-crate-root-test");
        std::fs::create_dir_all(&directory).expect("creates a scratch directory");
        std::fs::write(
            directory.join("Cargo.toml"),
            "[workspace]\nmembers = [\n    \"crates/example/nomos-example\",\n]\n",
        )
        .expect("writes a fixture manifest");

        assert_eq!(Crate_Root(&directory, "nomos-example"), "crates/example/nomos-example");
        // No matching member line: falls back to the bare crate name rather than guessing a
        // layout.
        assert_eq!(Crate_Root(&directory, "nomos-not-a-member"), "nomos-not-a-member");
    }
}
