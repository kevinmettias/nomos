//! Reading `go.work` and `go.mod` directly, and finding a workspace's own first-party
//! module edges in what they declare.
//!
//! No subprocess and no `go` toolchain — [`crate`]'s own module doc says why a text read
//! is a complete answer to this question rather than an approximation of one.

use nomos_cap_dependency::{DependencyEdge, DependencyKind, DependencyPayload};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[path = "discovery/discovered_module.rs"]
mod discovered_module;

pub use discovered_module::DiscoveredModule;

/// A `go.work`/`go.mod` file could not be read, or did not declare what this reader
/// expects.
///
/// A boundary this repository already lives by: a provider's `Discover_Workspace` reports
/// its own failures to a caller rather than panicking, the same shape
/// `nomos_lang_rust_cargo::MetadataError` takes for the identical reason.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleError
{
    pub reason: String,
}

impl core::fmt::Display for ModuleError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}

/// Every first-party module in a Go workspace rooted at `root`, restricted to edges that
/// name another *workspace* module.
///
/// A `go.work` at `root` names every member directory; its absence means `root` itself is
/// the workspace's one module, the same single-module default `go` itself takes. External
/// (non-workspace) requirements are read like any other and deliberately not returned, the
/// identical restriction `nomos-lang-rust-cargo`'s own `--no-deps` invocation imposes.
///
/// # Errors
///
/// [`ModuleError`] if `root` names no module at all (no `go.work` and no `go.mod`), a
/// named member's own `go.mod` cannot be read or does not declare a `module` line, or the
/// workspace resolves to no members.
pub fn Discover_Workspace(root: &Path) -> Result<Vec<DiscoveredModule>, ModuleError>
{
    let members = Member_Directories(root)?;
    let modules = Read_Modules(&members, root)?;
    let module_paths = Module_Paths(&modules);
    let discovered = modules
        .into_iter()
        .map(|module| Discover_Module(module, &module_paths))
        .collect();

    return Require_Nonempty(discovered);
}

/// One `go.mod`'s own declared module path and requirements, before edges are filtered to
/// first-party ones.
struct ReadModule
{
    module_path: String,
    manifest_relative_root: String,
    requires: Vec<String>,
}

/// `root`'s own workspace members: every directory a `go.work` names, or `root` alone if
/// there is none.
fn Member_Directories(root: &Path) -> Result<Vec<PathBuf>, ModuleError>
{
    let work_path = root.join("go.work");
    if !work_path.is_file()
    {
        return Ok(vec![root.to_path_buf()]);
    }

    let text = Read_To_String(&work_path)?;
    let uses = Parse_Use_Directives(&text);
    if uses.is_empty()
    {
        return Err(ModuleError {
            reason: format!("{} declares no `use` directive", work_path.display()),
        });
    }

    return Ok(uses.into_iter().map(|relative| root.join(relative)).collect());
}

/// Every `use` directive a `go.work` file's text declares, in the order they appear:
/// either the single-line `use <dir>` form or the block `use (\n <dir>\n ... )` form.
fn Parse_Use_Directives(text: &str) -> Vec<String>
{
    return Directive_Entries(text, Directive::Use);
}

/// Every module `root`'s own members declare, read from each one's own `go.mod`.
fn Read_Modules(members: &[PathBuf], root: &Path) -> Result<Vec<ReadModule>, ModuleError>
{
    let mut modules = Vec::with_capacity(members.len());
    for member in members
    {
        let module = Read_Module(member, root)?;
        modules.push(module);
    }

    return Ok(modules);
}

/// One member directory's own `go.mod`: its declared module path and every `require`
/// entry's module path, before either is checked against the rest of the workspace.
fn Read_Module(member: &Path, root: &Path) -> Result<ReadModule, ModuleError>
{
    let mod_path = member.join("go.mod");
    let text = Read_To_String(&mod_path)?;
    let module_path = Parse_Module_Line(&text).ok_or_else(|| ModuleError {
        reason: format!("{} has no `module` line", mod_path.display()),
    })?;
    let requires = Directive_Entries(&text, Directive::Require)
        .into_iter()
        .map(|entry| Require_Module_Path(&entry))
        .collect();

    return Ok(ReadModule {
        module_path,
        manifest_relative_root: Relative_To(member, root),
        requires,
    });
}

/// The `module <path>` line's own path, stripped of any trailing comment.
fn Parse_Module_Line(text: &str) -> Option<String>
{
    for line in text.lines()
    {
        let stripped = Strip_Comment(line).trim();
        if let Some(rest) = stripped.strip_prefix("module ")
        {
            return Some(rest.trim().to_owned());
        }
    }

    return None;
}

/// A `require` entry's own module path, `Directive_Entries`' `First_Field` unchanged —
/// named at the call site so a reader does not have to re-derive what the first field of
/// a `require` line means from a function that also serves `use`.
fn Require_Module_Path(entry: &str) -> String
{
    return entry.to_owned();
}

/// `path`, made relative to `root` and normalized to forward slashes — the same
/// convention every other subject in this workspace is addressed by.
fn Relative_To(path: &Path, root: &Path) -> String
{
    let relative = path.strip_prefix(root).unwrap_or(path);

    return relative.to_string_lossy().replace('\\', "/");
}

/// Every workspace member's own declared module path, for matching a `require` entry
/// against.
fn Module_Paths(modules: &[ReadModule]) -> BTreeSet<String>
{
    return modules.iter().map(|module| module.module_path.clone()).collect();
}

/// One module's own dependency payload, its edges restricted to the ones naming another
/// workspace member.
fn Discover_Module(module: ReadModule, module_paths: &BTreeSet<String>) -> DiscoveredModule
{
    let edges = First_Party_Edges(&module, module_paths);

    return Discovered_Module(module, edges);
}

/// Every edge `module` declares that names another workspace member, mapped into this
/// capability's payload shape and put in canonical order.
fn First_Party_Edges(module: &ReadModule, module_paths: &BTreeSet<String>) -> Vec<DependencyEdge>
{
    let mut edges: Vec<DependencyEdge> = module
        .requires
        .iter()
        .filter(|required| required.as_str() != module.module_path && module_paths.contains(required.as_str()))
        .map(|target| DependencyEdge {
            target: target.clone(),
            // Go's module system has no dev/build dependency tables and no optional,
            // feature-gated requirement — every `require` line is this one kind, the
            // crate's own module doc states why.
            kind: DependencyKind::Normal,
            optional: false,
        })
        .collect();

    // Canonical order: the fact's bytes must not depend on the order this file's own
    // `require` lines happened to be written in, which is an authoring detail rather
    // than anything this fact is about — the identical reasoning
    // `nomos_lang_rust_cargo::metadata::Dependency_Edges` sorts for.
    edges.sort_by(|left, right| left.target.cmp(&right.target));

    return edges;
}

fn Discovered_Module(module: ReadModule, edges: Vec<DependencyEdge>) -> DiscoveredModule
{
    return DiscoveredModule {
        payload: DependencyPayload {
            package: module.module_path,
            edges,
        },
        manifest_relative_root: module.manifest_relative_root,
    };
}

/// Refuses an empty result: a workspace resolving to no members means this reader saw
/// nothing, not that a real workspace has no modules.
fn Require_Nonempty(discovered: Vec<DiscoveredModule>) -> Result<Vec<DiscoveredModule>, ModuleError>
{
    if discovered.is_empty()
    {
        return Err(ModuleError {
            reason: "the workspace resolved to no modules; refusing to report a clean \
                     result over an empty graph"
                .to_owned(),
        });
    }

    return Ok(discovered);
}

/// The two directive keywords [`Directive_Entries`] reads -- named rather than passed as an
/// adjacent `&str` alongside the text being scanned, so a caller cannot transpose the two.
#[derive(Clone, Copy)]
enum Directive
{
    Use,
    Require,
}

impl Directive
{
    fn Keyword(self) -> &'static str
    {
        return match self
        {
            Directive::Use => "use",
            Directive::Require => "require",
        };
    }
}

/// Every entry a named block or single-line directive declares.
///
/// `go.mod`'s `require` and `go.work`'s `use` share one grammar: `<keyword> <entry>` on
/// one line, or `<keyword> (` opening a block of one entry per line until a lone `)`.
/// Reading both through this one function is not folding two different questions into
/// one — it is the same question (every entry one keyword declares) asked of two files
/// that happen to use the identical shape to answer it.
fn Directive_Entries(text: &str, directive: Directive) -> Vec<String>
{
    let keyword = directive.Keyword();
    let mut entries = Vec::new();
    let mut lines = text.lines();

    while let Some(line) = lines.next()
    {
        let Some(rest) = Directive_Rest(line, keyword)
        else
        {
            continue;
        };

        match rest.strip_prefix('(')
        {
            Some(block) => Record_Block_Entries(block, &mut lines, &mut entries),
            None => Record_Single_Entry(rest, &mut entries),
        }
    }

    return entries;
}

/// `line`'s own directive body, if it opens with `keyword` followed by whitespace — `None`
/// for any other line, including one this same keyword merely appears inside.
fn Directive_Rest<'a>(line: &'a str, keyword: &str) -> Option<&'a str>
{
    let stripped = Strip_Comment(line).trim();
    let rest = stripped.strip_prefix(keyword)?;

    if !rest.starts_with(char::is_whitespace)
    {
        return None;
    }

    return Some(rest.trim());
}

/// The single entry a non-block directive line names, if it names one at all.
fn Record_Single_Entry(rest: &str, entries: &mut Vec<String>)
{
    if !rest.is_empty()
    {
        entries.push(First_Field(rest));
    }
}

/// A `(...)` block's entries: whatever the opening line itself names, plus the rest of the
/// block, read until its closing `)`.
fn Record_Block_Entries<'a>(block: &str, lines: &mut impl Iterator<Item = &'a str>, entries: &mut Vec<String>)
{
    if !block.trim().is_empty()
    {
        entries.push(First_Field(block.trim()));
    }

    Read_Block(lines, entries);
}

/// Reads a `(...)` block's remaining lines, one entry per line, until the closing `)`.
fn Read_Block<'a>(lines: &mut impl Iterator<Item = &'a str>, entries: &mut Vec<String>)
{
    for line in lines.by_ref()
    {
        if Has_Closed_Block(line, entries)
        {
            return;
        }
    }
}

/// One line inside a `(...)` block: records whatever entry it names, if any, and reports
/// whether it also closed the block.
fn Has_Closed_Block(line: &str, entries: &mut Vec<String>) -> bool
{
    let stripped = Strip_Comment(line).trim();
    if stripped == ")"
    {
        return true;
    }
    if stripped.is_empty()
    {
        return false;
    }
    if let Some(rest) = stripped.strip_suffix(')')
    {
        Closing_Line_Entry(rest, entries);
        return true;
    }

    entries.push(First_Field(stripped));

    return false;
}

/// The entry a closing line names before its `)`, pushed if there is one -- a closing paren
/// sharing a line with the last entry is not seen in practice, but handled anyway rather
/// than left to read past the block's own end.
fn Closing_Line_Entry(rest_before_paren: &str, entries: &mut Vec<String>)
{
    let entry = rest_before_paren.trim();
    if !entry.is_empty()
    {
        entries.push(First_Field(entry));
    }
}

/// The first whitespace-delimited field of a directive's entry — a directory for `use`, a
/// module path for `require`. `require`'s second field (the version) and any trailing
/// `// indirect` comment are both already gone by the time this runs: the version is not
/// carried into this capability's payload at all, the same way `nomos-lang-rust-cargo`'s
/// own payload states a target and a kind but not a resolved version, and the comment was
/// already stripped by [`Strip_Comment`] before this function ever sees the line.
fn First_Field(entry: &str) -> String
{
    return entry.split_whitespace().next().unwrap_or(entry).to_owned();
}

/// Strips a `//` comment from a line, if it has one.
///
/// Splitting on the first `//` is sufficient here: neither a `go.work`/`go.mod` module
/// path nor a directory entry ever legitimately contains `//`, so this cannot cut a real
/// field short.
fn Strip_Comment(line: &str) -> &str
{
    return line.find("//").map_or(line, |at| &line[..at]);
}

fn Read_To_String(path: &Path) -> Result<String, ModuleError>
{
    return std::fs::read_to_string(path).map_err(|error| ModuleError {
        reason: format!("could not read {}: {error}", path.display()),
    });
}

#[cfg(test)]
#[path = "discovery/tests.rs"]
mod tests;

#[cfg(test)]
mod local_tests
{
    use super::*;

    #[test]
    fn Test_Discover_Workspace_Should_Refuse_A_Root_With_No_Go_Mod_Or_Go_Work()
    {
        let root = std::env::temp_dir().join(format!(
            "nomos-lang-go-modules-local-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).expect("a fresh temp directory can be created");

        let result = Discover_Workspace(&root);

        // Best-effort cleanup of the temp directory this test created. The test's own
        // outcome was already decided by `result` above; a failure here only leaves debris
        // on disk and changes nothing this assertion checks.
        let _ = std::fs::remove_dir_all(&root);
        assert!(result.is_err(), "an empty root names no module at all");
    }
}
