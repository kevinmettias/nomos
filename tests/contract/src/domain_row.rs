//! The six-row domain table, read out of `nomos-contracts` itself.
//!
//! The authority on what rows exist. Every peer that reimplements those types reads that
//! table, so a row there is a published claim whether or not anything in this workspace
//! occupies it — which is why the rows are derived from the contracts crate's own module
//! documentation rather than restated here.

use crate::Workspace;
use std::path::PathBuf;

/// One row of the domain table in `nomos-contracts`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DomainRow
{
    /// The domain as the table names it, for diagnostics.
    pub domain: String,
    /// The strength the row claims, without its backticks.
    pub strength: String,
    /// The scope the row claims.
    pub scope: String,
    /// The trace equivalence the row claims.
    pub trace: String,
}

impl DomainRow
{
    /// Whether the row promises anything at all.
    ///
    /// The `None` row promises nothing by design — it is what keeps determinism affordable,
    /// because the CLI, the reporting layer and the agent host pay nothing for it. A row
    /// that promises nothing owes no implementation, so a completeness guard that demanded
    /// one would be demanding a declaration that says "no promise" for every part of the
    /// system that was never going to make one.
    #[must_use]
    pub fn Claims_Reproducibility(&self) -> bool
    {
        return self.strength != "None";
    }
}

/// Where the table that says which domain claims what is written.
///
/// The module documentation of `nomos-contracts`, read as text.
#[must_use]
fn Domain_Table_Path() -> Option<PathBuf>
{
    let workspace = Workspace::Load();
    let contracts = workspace.Get("nomos-contracts")?;

    return Some(contracts.root.join("src/determinism.rs"));
}

/// Every row of the domain table, derived from the contracts crate's own source.
///
/// Returns an empty vector when the file cannot be read or holds no table, which every
/// caller must treat as a failure rather than as "there are no rows" — a completeness guard
/// quantifying over nothing passes having read nothing, and that is the shape this whole
/// directory exists to refuse.
///
/// # Panics
///
/// Panics if the dependency graph cannot be read, by way of [`Workspace::Load`].
#[must_use]
pub fn Domain_Table() -> Vec<DomainRow>
{
    let Some(path) = Domain_Table_Path()
    else
    {
        return Vec::new();
    };
    let Ok(text) = std::fs::read_to_string(path)
    else
    {
        return Vec::new();
    };

    let mut rows = Vec::new();
    for line in text.lines()
    {
        let row = Table_Row(line);

        rows.extend(row);
    }

    return rows;
}

/// One row of the table, or nothing where the line is prose or table syntax.
///
/// The header row and the `|---|` rule under it are table syntax rather than content, and
/// both survive the cell split. A row whose axes are not written as code spans is one of
/// those two.
fn Table_Row(line: &str) -> Option<DomainRow>
{
    let cell_text = line.trim().strip_prefix("//! |")?;
    let cells: Vec<&str> = cell_text.trim_end_matches('|').split('|').map(str::trim).collect();
    let [domain, strength, scope, trace] = cells.as_slice()
    else
    {
        return None;
    };
    let read = DomainRow {
        domain: (*domain).to_owned(),
        strength: Unquoted(strength),
        scope: Unquoted(scope),
        trace: Unquoted(trace),
    };
    if read.strength.is_empty() || strength.len() == read.strength.len()
    {
        return None;
    }

    return Some(read);
}

/// A markdown code span with its backticks removed.
fn Unquoted(cell: &str) -> String
{
    return cell.trim_matches('`').trim().to_owned();
}
