use crate::Item;
use crate::Projection;
use crate::Section;
use crate::GENERATED_FILE_NOTICE;
use crate::ProjectError;
use core::fmt::Write as _;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
// `clippy::struct_field_names` objects to `nomos_generated` repeating the type's own name. The
// field names here are the serialized keys, not internal names: `nomos_generated: true` is the
// marker every generated artifact opens with — `spec/domain-specification.md` and
// `diagrams/relations.mmd` both carry it, and it is asserted by that exact spelling in
// `read_surface.rs` and `renderers.rs`. Renaming the field to satisfy the lint would rewrite
// every projection this workspace ships.
#[allow(clippy::struct_field_names)]
struct Generated<'a>
{
    nomos_generated: bool,
    do_not_edit: &'a str,
    #[serde(flatten)]
    projection: &'a Projection,
}

pub fn Render_Projection(projection: &Projection) -> Result<String, ProjectError>
{
    use crate::Format;

    return match projection.format
    {
        Format::Markdown => Ok(Render_Markdown(projection)),
        Format::Html => Ok(Render_Html(projection)),
        Format::Mermaid => Ok(Render_Mermaid(projection)),
        Format::Json => Render_Json(projection),
        Format::Yaml => Render_Yaml(projection),
        Format::Contextpack => Render_Contextpack(projection),
    };
}

fn Render_Markdown(projection: &Projection) -> String
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

fn Render_Html(projection: &Projection) -> String
{
    let mut out = Html_Head(projection);

    for section in &projection.sections
    {
        let _ = writeln!(
            out,
            "<section id=\"{}\">\n<h2>{}</h2>",
            Slug_Of_Text(&section.title),
            Escape_Html(&section.title)
        );

        for item in &section.items
        {
            Write_Article(&mut out, item);
        }

        out.push_str("</section>\n");
    }

    out.push_str("</body>\n</html>\n");

    return out;
}

/// The document down to the opening heading, carrying the profile that generated it.
fn Html_Head(projection: &Projection) -> String
{
    let mut out = String::new();
    let _ = writeln!(
        out,
        "<!doctype html>\n<html lang=\"en\" data-nomos-generated=\"true\">\n<head>\n\
         <meta charset=\"utf-8\">\n<meta name=\"nomos-profile\" content=\"{}\">\n\
         <meta name=\"nomos-do-not-edit\" content=\"{}\">\n<title>{}</title>\n</head>\n<body>\n\
         <h1>{}</h1>",
        Escape_Html(&projection.profile),
        Escape_Html(GENERATED_FILE_NOTICE),
        Escape_Html(&projection.title),
        Escape_Html(&projection.title)
    );

    return out;
}

/// One item as an article: its identity, the fields it states, and its body.
fn Write_Article(out: &mut String, item: &Item)
{
    let _ = writeln!(out, "<article>\n<h3>{}</h3>", Escape_Html(&item.identity));
    if !item.fields.is_empty()
    {
        out.push_str("<dl>\n");
        for (name, value) in &item.fields
        {
            let _ = writeln!(out, "<dt>{}</dt><dd>{}</dd>", Escape_Html(name), Escape_Html(value));
        }
        out.push_str("</dl>\n");
    }
    if let Some(body) = &item.body
    {
        let _ = writeln!(out, "<pre>{}</pre>", Escape_Html(body.trim_end()));
    }
    out.push_str("</article>\n");
}

fn Render_Mermaid(projection: &Projection) -> String
{
    let mut out = format!(
        "%% nomos_generated: true\n%% do_not_edit: {GENERATED_FILE_NOTICE}\n%% profile: {}\ngraph LR\n",
        projection.profile
    );
    // One name table across every section, because an edge in one subgraph routinely names
    // a node declared in another and two tables would give that node two identifiers.
    let mut names = Names {
        known: BTreeMap::new(),
        taken: Vec::new(),
        labelled: Vec::new(),
    };

    for section in &projection.sections
    {
        Write_Subgraph(&mut out, &mut names, section);
    }

    return out;
}

/// One section as a subgraph.
fn Write_Subgraph(out: &mut String, names: &mut Names, section: &Section)
{
    let _ = writeln!(out, "  subgraph {}", Quoted_For_Mermaid(&section.title));

    for item in &section.items
    {
        Node_Or_Edge(out, names, item);
    }

    out.push_str("  end\n");
}

/// An item that names both ends is an edge; anything else is a node.
///
/// An edge declares both of its ends, because a relation may name a node no section
/// carried and a graph with a dangling reference does not render at all.
fn Node_Or_Edge(out: &mut String, names: &mut Names, item: &Item)
{
    let Some((from, to)) = item.Field("from").zip(item.Field("to"))
    else
    {
        let label = item.Field("title").unwrap_or(&item.identity);
        let declared = names.Declared(Identity(&item.identity), Label(label));
        let _ = writeln!(out, "    {declared}");

        return;
    };

    let relation = item.Field("relation").unwrap_or("relates");
    let tail = names.Declared(Identity(from), Label(from));
    let head = names.Declared(Identity(to), Label(to));
    let _ = writeln!(out, "    {tail} -->|{}| {head}", Quoted_For_Mermaid(relation));
}

fn Render_Json(projection: &Projection) -> Result<String, ProjectError>
{
    let generated = Generated {
        nomos_generated: true,
        do_not_edit: GENERATED_FILE_NOTICE,
        projection,
    };
    let mut rendered = serde_json::to_string_pretty(&generated)
        .map_err(|error| return ProjectError::Malformed(error.to_string()))?;
    rendered.push('\n');

    return Ok(rendered);
}

fn Render_Yaml(projection: &Projection) -> Result<String, ProjectError>
{
    let generated = Generated {
        nomos_generated: true,
        do_not_edit: GENERATED_FILE_NOTICE,
        projection,
    };

    return serde_yaml_ng::to_string(&generated)
        .map_err(|error| return ProjectError::Malformed(error.to_string()));
}

fn Render_Contextpack(projection: &Projection) -> Result<String, ProjectError>
{
    let pack = Pack {
        nomos_generated: true,
        do_not_edit: GENERATED_FILE_NOTICE,
        profile: &projection.profile,
        title: &projection.title,
        inputs_digest: projection.Inputs_Digest(),
        sections: projection.sections.iter().map(Packed_Section).collect(),
    };

    let mut rendered = serde_json::to_string_pretty(&pack)
        .map_err(|error| return ProjectError::Malformed(error.to_string()))?;
    rendered.push('\n');

    return Ok(rendered);
}

fn Escape_Html(value: &str) -> String
{
    return value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");
}

fn Slug_Of_Text(value: &str) -> String
{
    let mut slug = String::new();
    let mut pending = false;

    for character in value.chars()
    {
        if character.is_ascii_alphanumeric()
        {
            if pending && !slug.is_empty()
            {
                slug.push('-');
            }
            pending = false;
            slug.extend(character.to_lowercase());
        }
        else
        {
            pending = true;
        }
    }

    return slug;
}

fn Quoted_For_Mermaid(value: &str) -> String
{
    return format!("\"{}\"", value.replace('"', "'").replace(['\n', '\r'], " "));
}

#[derive(Serialize)]
struct Pack<'a>
{
    nomos_generated: bool,
    do_not_edit: &'a str,
    profile: &'a str,
    title: &'a str,
    inputs_digest: String,
    sections: Vec<PackSection<'a>>,
}

#[derive(Serialize)]
struct PackSection<'a>
{
    title: &'a str,
    items: Vec<PackItem<'a>>,
}

#[derive(Serialize)]
struct PackItem<'a>
{
    identity: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    title: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<&'a String>,
}

/// One section reduced to what a context pack carries: identity, title, and prose.
///
/// The other fields are dropped on purpose. A pack is read by something with a budget, and
/// a field that only a table renderer uses costs that budget without answering anything.
fn Packed_Section(section: &Section) -> PackSection<'_>
{
    return PackSection {
        title: &section.title,
        items: section
            .items
            .iter()
            .map(|item| {
                return PackItem {
                    identity: &item.identity,
                    title: item.Field("title").unwrap_or_default(),
                    text: item.body.as_ref(),
                };
            })
            .collect(),
    };
}

/// A node's identity, kept distinct from [`Label`] so [`Names::Declared`]'s two positions
/// cannot be swapped at a call site — both are plain strings and nothing else would tell
/// them apart.
struct Identity<'a>(&'a str);

/// A node's rendered label, kept distinct from [`Identity`] for the same reason.
struct Label<'a>(&'a str);

struct Names
{
    known: BTreeMap<String, String>,
    taken: Vec<String>,
    labelled: Vec<String>,
}

impl Names
{
    fn Declared(&mut self, identity: Identity<'_>, label: Label<'_>) -> String
    {
        let identity = identity.0;
        let label = label.0;

        let named = self.For(identity);
        if self.labelled.contains(&named)
        {
            return named;
        }
        self.labelled.push(named.clone());

        return format!("{named}[{}]", Quoted_For_Mermaid(label));
    }

    fn For(&mut self, identity: &str) -> String
    {
        if let Some(known) = self.known.get(identity)
        {
            return known.clone();
        }

        let candidate = match Slug_Of_Text(identity).replace('-', "_")
        {
            slug if slug.is_empty() => "n".to_owned(),
            slug => slug,
        };
        let mut unique = candidate.clone();
        let mut ordinal = 1_u32;
        while self.taken.contains(&unique)
        {
            ordinal = ordinal.saturating_add(1);
            unique = format!("{candidate}_{ordinal}");
        }

        self.taken.push(unique.clone());
        self.known.insert(identity.to_owned(), unique.clone());

        return unique;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{Content, Format, Name, Value};

    fn One_Section_Projection() -> Projection
    {
        return Projection {
            profile: "one".to_owned(),
            title: "One".to_owned(),
            format: Format::Markdown,
            output: "one.md".to_owned(),
            sections: vec![Section {
                title: "Nodes".to_owned(),
                content: Content::Nodes,
                items: vec![Item::Of("CDM-ONE").With(Name("title"), Value("One"))],
            }],
            inputs: Vec::new(),
        };
    }

    #[test]
    fn Test_Render_Projection_Should_Dispatch_To_The_Format_The_Projection_Declares()
    {
        let mut projection = One_Section_Projection();
        let markdown = Render_Projection(&projection).expect("renders");

        assert!(markdown.contains("# One"), "{markdown}");
        assert!(markdown.contains("## Nodes"), "{markdown}");

        projection.format = Format::Json;
        let json = Render_Projection(&projection).expect("renders");

        assert!(json.contains("\"title\": \"One\""), "{json}");
    }
}
