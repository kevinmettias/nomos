//! `judge-role` -- reading a crate's declared role and actual surface, judging them with
//! `nomos_rules::Check_Declared_Role_Matches_Surface`, and dispatching the finding it
//! produces.

use super::{DispatchConfig, ExitCode};
use nomos_agent_contracts::TaskEnvelope;
use nomos_contracts::{Finding, SchemaId};
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
    config: DispatchConfig,
    output: &mut impl std::io::Write,
    notes: &mut impl std::io::Write,
) -> ExitCode
{
    use super::dispatch::Dispatch_Task;

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

    let task = Judgment_Task(&pair, &finding, config.effort);

    return Dispatch_Task(&task, config.backend, output, notes);
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

/// The judgment `role_surface.rs`'s own module doc says this rule cannot reach itself —
/// whether `pair`'s declared role and actual surface agree — carrying `finding.summary`
/// so the dispatched question is traceably the rule's own, not a paraphrase invented here.
fn Judgment_Task(pair: &RoleSurfacePair, finding: &Finding, effort: nomos_model_package::EffortLevel) -> TaskEnvelope
{
    use nomos_ledger::Territory;

    let goal = format!(
        "A Rust crate's declared role, from its workspace README's band table: {}\n\n\
         The crate's actual public surface, as a list of every item it exports:\n{}\n\n\
         {}. Does the declared role accurately and completely describe what the surface \
         exports? Name anything the role claims that the surface does not show, or anything \
         the surface exports that the role does not mention, in 2-4 sentences.",
        pair.declared_role, pair.actual_surface, finding.summary
    );

    return TaskEnvelope {
        goal,
        scope: Territory::Of_Files(Vec::<String>::new()),
        knowledge_context: Vec::new(),
        applicable_rules: vec![finding.rule.clone()],
        prohibited_changes: Territory::Of_Files(Vec::<String>::new()),
        available_tools: Vec::new(),
        expected_output_schema: SchemaId::New("nomos.agent.executor.cli.v1"),
        effort,
    };
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
