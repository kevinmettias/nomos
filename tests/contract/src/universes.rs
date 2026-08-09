//! Declared universes — the lists a completeness guard quantifies over.
//!
//! `OD-COMPLETENESS-001` names the shape: a completeness guard is only as complete as the
//! universe it quantifies over, and when that universe is *declared* rather than derived,
//! comparing the declaration against reality is the other direction. Three guards in this
//! workspace were half-checks for exactly that reason, and all three were found by
//! accident.
//!
//! This module does the mechanical half — finding the declarations. It cannot do the other
//! half. Whether a given list has a check comparing it against the reality it claims to
//! enumerate is a question about meaning, and `tests/contract` deliberately has no types to
//! answer it with. So the classification is declared in a table, and the table is checked
//! against what this module derives, which is the shape `OD-GATE-001` already uses: neither
//! side is trusted alone, and the table cannot go stale in the direction that flatters.

use crate::metadata::Workspace;
use std::path::{Path, PathBuf};

/// How a universe is written down.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UniverseKind
{
    /// A constant slice — a list of names kept beside the thing it enumerates.
    Constant,
    /// An `All()` over an enum — a list of variants kept beside the enum.
    ///
    /// The compiler does not check it. A variant added without adding it here drops out of
    /// every guard built on `All()`, and each of those guards then passes by not looking.
    Enumeration,
}

/// One list that some completeness guard quantifies over.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeclaredUniverse
{
    /// Repo-relative, forward slashes.
    pub path: String,
    /// `GOVERNING_RECORD_IDS`, or `Table::All` for an enumeration.
    pub name: String,
    /// How it is written down.
    pub kind: UniverseKind,
}

/// Every declared universe in the workspace, derived from the source.
///
/// Deliberately over-inclusive. A list that turns out to need no mirror is cheap to
/// classify once; a list that is never surfaced is the defect this exists to prevent.
#[must_use]
pub fn Declared_Universes() -> Vec<DeclaredUniverse>
{
    let workspace = Workspace::Load();
    let root = Workspace::Workspace_Root();
    let mut universes = Vec::new();

    for member in workspace.Members()
    {
        for directory in ["src", "tests"]
        {
            let source_root = member.root.join(directory);
            if !source_root.is_dir()
            {
                continue;
            }

            for file in Source_Files(&source_root)
            {
                let Ok(text) = std::fs::read_to_string(&file)
                else
                {
                    continue;
                };

                let relative = file
                    .strip_prefix(&root)
                    .unwrap_or(&file)
                    .display()
                    .to_string()
                    .replace('\\', "/");

                universes.extend(Universes_In(&relative, &text));
            }
        }
    }

    universes.sort();
    universes.dedup();
    return universes;
}

/// The declarations in one file.
fn Universes_In(path: &str, text: &str) -> Vec<DeclaredUniverse>
{
    let mut found = Vec::new();
    let mut current_type: Option<String> = None;

    for line in text.lines()
    {
        let trimmed = line.trim();

        // `impl Table` / `impl core::fmt::Display for Table` — only the bare form names a
        // type whose `All()` would be its own variant list.
        if let Some(rest) = trimmed.strip_prefix("impl ")
            && !rest.contains(" for ")
            && let Some(name) = rest.split_whitespace().next()
        {
            current_type = Some(name.trim_end_matches('{').trim().to_owned());
        }

        if let Some(rest) = trimmed.strip_prefix("pub const ")
            && let Some((name, tail)) = rest.split_once(':')
            && tail.trim_start().starts_with("&[")
        {
            found.push(DeclaredUniverse {
                path: path.to_owned(),
                name: name.trim().to_owned(),
                kind: UniverseKind::Constant,
            });
        }

        if trimmed.contains("fn All()")
            && let Some(owner) = current_type.as_ref()
        {
            found.push(DeclaredUniverse {
                path: path.to_owned(),
                name: format!("{owner}::All"),
                kind: UniverseKind::Enumeration,
            });
        }
    }

    return found;
}

/// Every `.rs` file under a directory.
fn Source_Files(root: &Path) -> Vec<PathBuf>
{
    let mut files = Vec::new();
    let mut pending = vec![root.to_path_buf()];

    while let Some(directory) = pending.pop()
    {
        let Ok(entries) = std::fs::read_dir(&directory)
        else
        {
            continue;
        };

        for entry in entries.flatten()
        {
            let path = entry.path();
            if path.is_dir()
            {
                pending.push(path);
            }
            else if path.extension().is_some_and(|extension| extension == "rs")
            {
                files.push(path);
            }
        }
    }

    files.sort();
    return files;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Constant_Slice_Should_Be_Found()
    {
        let found = Universes_In("a.rs", "pub const GOVERNING_RECORD_IDS: &[&str] = &[\n];\n");

        assert_eq!(found.len(), 1);
        assert_eq!(
            found.first().map(|universe| universe.kind),
            Some(UniverseKind::Constant)
        );
    }

    #[test]
    fn Test_An_All_Should_Be_Attributed_To_Its_Type()
    {
        let found = Universes_In(
            "a.rs",
            "impl Table\n{\n    pub const fn All() -> &'static [Self]\n    {\n    }\n}\n",
        );

        assert_eq!(
            found.first().map(|universe| universe.name.clone()),
            Some("Table::All".to_owned())
        );
    }

    /// A trait implementation does not own the type's variant list, so attributing an
    /// `All()` to it would name the wrong universe.
    #[test]
    fn Test_A_Trait_Impl_Should_Not_Claim_The_Type()
    {
        let found = Universes_In(
            "a.rs",
            "impl Table\n{\n}\nimpl core::fmt::Display for Other\n{\n    fn All() {}\n}\n",
        );

        assert_eq!(
            found.first().map(|universe| universe.name.clone()),
            Some("Table::All".to_owned())
        );
    }

    /// A scalar constant is not a universe. Matching it would bury the real ones.
    #[test]
    fn Test_A_Scalar_Constant_Should_Not_Be_A_Universe()
    {
        assert!(Universes_In("a.rs", "pub const LIMIT: usize = 2_000;\n").is_empty());
        assert!(Universes_In("a.rs", "pub const NAME: &str = \"x\";\n").is_empty());
    }
}

