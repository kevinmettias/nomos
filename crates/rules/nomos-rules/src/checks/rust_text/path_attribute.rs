//! `a-rust-path-stays-within-its-own-subtree`: a `#[path]` that leaves the file's subtree.
//!
//! Split out of [`super`], which states the family's shared reasoning. Unlike its siblings,
//! this rule reads an attribute's own quoted value, so it judges `Code_Prefix`'s output
//! rather than the string-body-masked text.

use crate::checks::code_prefix::Code_Prefix;
use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::Finding;
use std::path::Component;

use super::{Finding_For_Line, Is_Own_Implementation_File, Line_Number, A_RUST_PATH_STAYS_WITHIN_ITS_OWN_SUBTREE};

/// Reports Rust `#[path = "..."]` attributes whose value is absolute or escapes upward.
#[must_use]
pub fn Check_A_Rust_Path_Stays_Within_Its_Own_Subtree(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE) && !Is_Own_Implementation_File(source)
        {
            findings.extend(Path_Attribute_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Path_Attribute_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for (index, line) in source.text.lines().enumerate()
    {
        let code = Code_Prefix(line);
        if let Some(value) = Path_Attribute_Value(&code)
        {
            if Is_Path_Absolute_Or_Escaping(value)
            {
                let finding = Finding_For_Line(
                    source,
                    A_RUST_PATH_STAYS_WITHIN_ITS_OWN_SUBTREE,
                    Line_Number(index),
                    "`#[path]` leaves the declaring file subtree",
                );
                findings.push(finding);
            }
        }
    }

    return findings;
}

fn Path_Attribute_Value(code: &str) -> Option<&str>
{
    let attribute_start = code.find("#[path")?;
    let after_attribute = code.get(attribute_start..)?;
    let first_quote = after_attribute.find('"')?;
    let after_first_quote = after_attribute.get(first_quote.saturating_add(1)..)?;
    let second_quote = after_first_quote.find('"')?;
    return after_first_quote.get(..second_quote);
}

fn Is_Path_Absolute_Or_Escaping(value: &str) -> bool
{
    let path = std::path::Path::new(value);
    if path.is_absolute()
    {
        return true;
    }

    return Is_Relative_Path_Escaping_Its_Own_Subtree(path);
}

/// Walks a relative path's components, tracking how many directories deep it has descended,
/// and reports whether a `..` ever climbs back above the starting point.
fn Is_Relative_Path_Escaping_Its_Own_Subtree(path: &std::path::Path) -> bool
{
    let mut depth = 0usize;
    for component in path.components()
    {
        match component
        {
            Component::ParentDir =>
            {
                let Some(next_depth) = depth.checked_sub(1)
                else
                {
                    return true;
                };
                depth = next_depth;
            }
            Component::Normal(_) => depth = depth.saturating_add(1),
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => return true,
        }
    }

    return false;
}
