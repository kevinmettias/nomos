//! Markdown: prose becomes headed bodies, fielded items a table, and items with neither a list.

use crate::GENERATED_FILE_NOTICE;
use crate::Item;
use crate::Projection;
use crate::Section;
use core::fmt::Write as _;

pub(super) fn Render_Markdown(projection: &Projection) -> String
{
    let mut out = String::new();
    let _ = writeln!(
        out,
        "---\nnomos_generated: true\ndo_not_edit: {GENERATED_FILE_NOTICE}\nprofile: {}\n---\n\n# {}",
        projection.profile, projection.title
    );

    for section in &projection.sections
    {
        let _ = writeln!(out, "\n## {}", section.title);
        Section_Body(&mut out, section);
    }

    return out;
}

/// One section, written as whatever its items can carry.
///
/// Prose becomes headed bodies, fielded items become a table, and items with neither
/// become a bare list. A table of one empty column is not a better answer than a list.
fn Section_Body(out: &mut String, section: &Section)
{
    if Has_A_Body(section)
    {
        Write_Bodies(out, section);

        return;
    }

    let columns = Table_Columns(section);
    if columns.is_empty()
    {
        for item in &section.items
        {
            let _ = writeln!(out, "\n- {}", item.identity);
        }

        return;
    }

    Write_Table(out, section, &columns);
}

fn Has_A_Body(section: &Section) -> bool
{
    return section.items.iter().any(|item| return item.body.is_some());
}

/// A section whose items carry prose, written as headed bodies rather than table rows.
fn Write_Bodies(out: &mut String, section: &Section)
{
    for item in &section.items
    {
        let _ = writeln!(out, "\n### {}", item.identity);
        let stated = Stated_Fields(item);
        if !stated.is_empty()
        {
            let _ = writeln!(out, "\n{stated}");
        }
        if let Some(body) = &item.body
        {
            let _ = writeln!(out, "\n{}", body.trim_end());
        }
    }
}

fn Stated_Fields(item: &Item) -> String
{
    let stated: Vec<String> = item
        .fields
        .iter()
        .filter(|(_, value)| return !value.is_empty())
        .map(|(name, value)| return format!("{name}: {value}"))
        .collect();

    if stated.is_empty()
    {
        return String::new();
    }

    return format!("*{}*", stated.join(" \u{b7} "));
}

fn Table_Columns(section: &Section) -> Vec<String>
{
    let mut columns: Vec<String> = Vec::new();
    for item in &section.items
    {
        for (name, _) in &item.fields
        {
            if !columns.contains(name)
            {
                columns.push(name.clone());
            }
        }
    }

    return columns;
}

/// A section's items as a markdown table, one column per field any item states.
fn Write_Table(out: &mut String, section: &Section, columns: &[String])
{
    let _ = writeln!(out, "\n| identity | {} |", columns.join(" | "));
    let _ = writeln!(out, "| --- |{}", " --- |".repeat(columns.len()));

    for item in &section.items
    {
        let cells: Vec<String> = columns
            .iter()
            .map(|column| return Table_Cell(item.Field(column).unwrap_or_default()))
            .collect();
        let _ = writeln!(out, "| {} | {} |", Table_Cell(&item.identity), cells.join(" | "));
    }
}

fn Table_Cell(value: &str) -> String
{
    return value.replace('|', "\\|").replace(['\n', '\r'], " ");
}
