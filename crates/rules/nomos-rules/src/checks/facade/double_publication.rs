//! A facade that publishes one child both as a namespace and as a flattened re-export.
//!
//! Split out of [`super`], which states the family's shared reasoning and the two narrowings
//! all three rules share. This file is the reading and the judgment behind
//! `facade-chooses-flattening-or-namespace` alone: which lines publish a child module, which
//! lines re-export through it, and which files do both.

use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::Finding;

use super::text::{After_Public_Visibility, Keyword_Body, Leading_Identifier};
use super::{Code_Lines, Finding_At, Line_Number, FACADE_CHOOSES_FLATTENING_OR_NAMESPACE};

/// Reports a Rust file that exposes one child module both as a public module and through a
/// public re-export, so the same item is reachable by two public paths.
#[must_use]
pub fn Check_A_Facade_Publishes_A_Child_One_Way(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE)
        {
            findings.extend(Double_Publication_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Double_Publication_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let code_lines = Code_Lines(&source.text);
    let published: Vec<&str> = code_lines.iter().filter_map(|line| return Public_Module_Name(line)).collect();

    let mut findings = Vec::new();
    for (index, line) in code_lines.iter().enumerate()
    {
        let finding = Double_Publication_Finding_For_Line(source, line, index, &published);
        if let Some(finding) = finding
        {
            findings.push(finding);
        }
    }

    return findings;
}

/// The module name a `pub mod <name>;` line declares. An inline `pub mod name { ... }` block
/// declares no child file and is not one.
fn Public_Module_Name(code: &str) -> Option<&str>
{
    let after_visibility = After_Public_Visibility(code)?;
    let after_keyword = Keyword_Body(after_visibility, "mod")?;
    let (name, after_name) = Leading_Identifier(after_keyword)?;

    if !after_name.trim_start().starts_with(';')
    {
        return None;
    }

    return Some(name);
}

fn Double_Publication_Finding_For_Line(source: &SourceFile, line: &str, index: usize, published: &[&str]) -> Option<Finding>
{
    let child = Public_Use_Head(line)?;
    if !published.contains(&child)
    {
        return None;
    }

    let line_number = Line_Number(index);
    return Some(Finding_At(
        source,
        FACADE_CHOOSES_FLATTENING_OR_NAMESPACE,
        line_number,
        &format!(
            "publishes `{child}` both as a public module and through a public re-export; a facade either \
             publishes the child namespace with `pub mod` or keeps it private and lifts selected items with \
             `pub use`, never both"
        ),
    ));
}

/// The child module a `pub use <child>::...` line re-exports through.
fn Public_Use_Head(code: &str) -> Option<&str>
{
    let after_visibility = After_Public_Visibility(code)?;
    let after_keyword = Keyword_Body(after_visibility, "use")?;
    let rest = after_keyword.strip_prefix("self::").unwrap_or(after_keyword);
    let (head, after_head) = Leading_Identifier(rest)?;

    if !after_head.starts_with("::")
    {
        return None;
    }

    return Some(head);
}
