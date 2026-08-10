use crate::profile::Format;
use crate::projection::{Item, Projection, Section, DO_NOT_EDIT};
use crate::ProjectError;
use core::fmt::Write as _;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
#[allow(clippy::struct_field_names)]
struct Generated<'a>
{
    nomos_generated: bool,
    do_not_edit: &'a str,
    #[serde(flatten)]
    projection: &'a Projection,
}

pub fn Render(projection: &Projection) -> Result<String, ProjectError>
{
    return match projection.format
    {
        Format::Markdown => Ok(Markdown(projection)),
        Format::Html => Ok(Html(projection)),
        Format::Mermaid => Ok(Mermaid(projection)),
        Format::Json => Json(projection),
        Format::Yaml => Yaml(projection),
        Format::Contextpack => Contextpack(projection),
    };
}

fn Carries_A_Body(section: &Section) -> bool
{
    return section.items.iter().any(|item| return item.body.is_some());
}

fn Columns(section: &Section) -> Vec<String>
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

fn Markdown(projection: &Projection) -> String
{
    let mut out = String::new();
    let _ = writeln!(
        out,
        "---\nnomos_generated: true\ndo_not_edit: {DO_NOT_EDIT}\nprofile: {}\n---\n\n# {}",
        projection.profile, projection.title
    );

    for section in &projection.sections
    {
        let _ = writeln!(out, "\n## {}", section.title);

        if Carries_A_Body(section)
        {
            Bodies(&mut out, section);
            continue;
        }

        let columns = Columns(section);
        if columns.is_empty()
        {
            for item in &section.items
            {
                let _ = writeln!(out, "\n- {}", item.identity);
            }
            continue;
        }

        let _ = writeln!(out, "\n| identity | {} |", columns.join(" | "));
        let _ = writeln!(out, "| --- |{}", " --- |".repeat(columns.len()));
        for item in &section.items
        {
            let cells: Vec<String> = columns
                .iter()
                .map(|column| return Cell(item.Field(column).unwrap_or_default()))
                .collect();
            let _ = writeln!(out, "| {} | {} |", Cell(&item.identity), cells.join(" | "));
        }
    }

    return out;
}

/// A section whose items carry prose, written as headed bodies rather than table rows.
fn Bodies(out: &mut String, section: &Section)
{
    for item in &section.items
    {
        let _ = writeln!(out, "\n### {}", item.identity);
        let stated = Stated(item);
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

fn Stated(item: &Item) -> String
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

fn Cell(value: &str) -> String
{
    return value.replace('|', "\\|").replace(['\n', '\r'], " ");
}

fn Html(projection: &Projection) -> String
{
    let mut out = String::new();
    let _ = writeln!(
        out,
        "<!doctype html>\n<html lang=\"en\" data-nomos-generated=\"true\">\n<head>\n\
         <meta charset=\"utf-8\">\n<meta name=\"nomos-profile\" content=\"{}\">\n\
         <meta name=\"nomos-do-not-edit\" content=\"{}\">\n<title>{}</title>\n</head>\n<body>\n\
         <h1>{}</h1>",
        Escaped(&projection.profile),
        Escaped(DO_NOT_EDIT),
        Escaped(&projection.title),
        Escaped(&projection.title)
    );

    for section in &projection.sections
    {
        let _ = writeln!(
            out,
            "<section id=\"{}\">\n<h2>{}</h2>",
            Slug(&section.title),
            Escaped(&section.title)
        );

        for item in &section.items
        {
            Article(&mut out, item);
        }

        out.push_str("</section>\n");
    }

    out.push_str("</body>\n</html>\n");

    return out;
}

/// One item as an article: its identity, the fields it states, and its body.
fn Article(out: &mut String, item: &Item)
{
    let _ = writeln!(out, "<article>\n<h3>{}</h3>", Escaped(&item.identity));
    if !item.fields.is_empty()
    {
        out.push_str("<dl>\n");
        for (name, value) in &item.fields
        {
            let _ = writeln!(out, "<dt>{}</dt><dd>{}</dd>", Escaped(name), Escaped(value));
        }
        out.push_str("</dl>\n");
    }
    if let Some(body) = &item.body
    {
        let _ = writeln!(out, "<pre>{}</pre>", Escaped(body.trim_end()));
    }
    out.push_str("</article>\n");
}

fn Escaped(value: &str) -> String
{
    return value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");
}

fn Slug(value: &str) -> String
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

struct Names
{
    known: BTreeMap<String, String>,
    taken: Vec<String>,
    labelled: Vec<String>,
}

impl Names
{
    fn For(&mut self, identity: &str) -> String
    {
        if let Some(known) = self.known.get(identity)
        {
            return known.clone();
        }

        let candidate = match Slug(identity).replace('-', "_")
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

    fn Declared(&mut self, identity: &str, label: &str) -> String
    {
        let named = self.For(identity);
        if self.labelled.contains(&named)
        {
            return named;
        }
        self.labelled.push(named.clone());

        return format!("{named}[{}]", Quoted(label));
    }
}

fn Mermaid(projection: &Projection) -> String
{
    let mut out = format!(
        "%% nomos_generated: true\n%% do_not_edit: {DO_NOT_EDIT}\n%% profile: {}\ngraph LR\n",
        projection.profile
    );
    let mut names = Names {
        known: BTreeMap::new(),
        taken: Vec::new(),
        labelled: Vec::new(),
    };

    for section in &projection.sections
    {
        let _ = writeln!(out, "  subgraph {}", Quoted(&section.title));

        for item in &section.items
        {
            if let (Some(from), Some(to)) = (item.Field("from"), item.Field("to"))
            {
                let relation = item.Field("relation").unwrap_or("relates");
                let tail = names.Declared(from, from);
                let head = names.Declared(to, to);
                let _ = writeln!(out, "    {tail} -->|{}| {head}", Quoted(relation));
            }
            else
            {
                let label = item.Field("title").unwrap_or(&item.identity);
                let declared = names.Declared(&item.identity, label);
                let _ = writeln!(out, "    {declared}");
            }
        }

        out.push_str("  end\n");
    }

    return out;
}

fn Quoted(value: &str) -> String
{
    return format!("\"{}\"", value.replace('"', "'").replace(['\n', '\r'], " "));
}

fn Json(projection: &Projection) -> Result<String, ProjectError>
{
    let generated = Generated {
        nomos_generated: true,
        do_not_edit: DO_NOT_EDIT,
        projection,
    };
    let mut rendered = serde_json::to_string_pretty(&generated)
        .map_err(|error| return ProjectError::Malformed(error.to_string()))?;
    rendered.push('\n');

    return Ok(rendered);
}

fn Yaml(projection: &Projection) -> Result<String, ProjectError>
{
    let generated = Generated {
        nomos_generated: true,
        do_not_edit: DO_NOT_EDIT,
        projection,
    };

    return serde_yaml_ng::to_string(&generated)
        .map_err(|error| return ProjectError::Malformed(error.to_string()));
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

fn Contextpack(projection: &Projection) -> Result<String, ProjectError>
{
    let pack = Pack {
        nomos_generated: true,
        do_not_edit: DO_NOT_EDIT,
        profile: &projection.profile,
        title: &projection.title,
        inputs_digest: projection.Inputs_Digest(),
        sections: projection
            .sections
            .iter()
            .map(|section| {
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
            })
            .collect(),
    };

    let mut rendered = serde_json::to_string_pretty(&pack)
        .map_err(|error| return ProjectError::Malformed(error.to_string()))?;
    rendered.push('\n');

    return Ok(rendered);
}
