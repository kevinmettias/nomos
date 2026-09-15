//! HTML: one section per heading, one article per item, and a link wherever an identity lands.

use crate::GENERATED_FILE_NOTICE;
use crate::Item;
use crate::Projection;
use crate::Section;
use core::fmt::Write as _;
use std::collections::BTreeSet;

use super::text::{Escape_Html, Slug_Of_Text};

pub(super) fn Render_Html(projection: &Projection) -> String
{
    let addressable = Addressable_Identities(projection);

    let mut out = Html_Head(projection);

    for section in &projection.sections
    {
        Write_Section(&mut out, section, &addressable);
    }

    out.push_str("</body>\n</html>\n");

    return out;
}

/// Every identity this projection carries, gathered before anything is written.
///
/// Gathered first because an item in the first section routinely names one in the last, and a
/// reader following a reference forward should not depend on which order the sections were
/// declared in.
fn Addressable_Identities(projection: &Projection) -> BTreeSet<&str>
{
    return projection
        .sections
        .iter()
        .flat_map(|section| return section.items.iter())
        .map(|item| return item.identity.as_str())
        .collect();
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

/// One section as a heading and the articles under it.
fn Write_Section(out: &mut String, section: &Section, addressable: &BTreeSet<&str>)
{
    let _ = writeln!(
        out,
        "<section id=\"{}\">\n<h2>{}</h2>",
        Slug_Of_Text(&section.title),
        Escape_Html(&section.title)
    );

    for item in &section.items
    {
        Write_Article(out, item, addressable);
    }

    out.push_str("</section>\n");
}

/// One item as an article: its identity, the fields it states, and its body.
fn Write_Article(out: &mut String, item: &Item, addressable: &BTreeSet<&str>)
{
    // The identity becomes the article's own anchor, which is what lets anything else in the
    // projection point at it. Through `Slug_Of_Text` rather than raw: that function emits
    // ASCII alphanumerics and `-` and nothing else, so the attribute cannot be broken by an
    // identity whatever it contains -- which is why this one position needs no escaping while
    // every other position below does.
    let _ = writeln!(
        out,
        "<article id=\"{}\">\n<h3>{}</h3>",
        Slug_Of_Text(&item.identity),
        Escape_Html(&item.identity)
    );
    if !item.fields.is_empty()
    {
        out.push_str("<dl>\n");
        for (name, value) in &item.fields
        {
            let _ = writeln!(out, "<dt>{}</dt><dd>{}</dd>", Escape_Html(name), Resolved(value, addressable));
        }
        out.push_str("</dl>\n");
    }
    if let Some(body) = &item.body
    {
        let _ = writeln!(out, "<pre>{}</pre>", Escape_Html(body.trim_end()));
    }
    out.push_str("</article>\n");
}

/// A field value, as a link when it names another item of this projection and as plain text
/// when it does not.
///
/// # Why an unresolvable identity stays text
///
/// A `relations` section names both endpoints of every edge, and in a subject-scoped profile
/// the far end is routinely a record the projection does not carry. Rendering that as a link
/// would give a reader an anchor that scrolls nowhere — which is worse than the text it
/// replaced, because a dead link reads as a promise. The projection knows exactly which
/// identities it holds, so it can tell the two apart rather than guess.
///
/// # Why the displayed text is still escaped
///
/// The href is a slug and cannot break the attribute, but the text between the tags is the
/// identity as authored and can. Both positions are handled here so a caller cannot get one
/// right and the other wrong.
fn Resolved(value: &str, addressable: &BTreeSet<&str>) -> String
{
    if addressable.contains(value)
    {
        return format!("<a href=\"#{}\">{}</a>", Slug_Of_Text(value), Escape_Html(value));
    }

    return Escape_Html(value);
}
