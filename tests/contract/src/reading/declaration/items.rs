//! Walking one module's text and recording what it declares.
//!
//! The traversal half: where an item begins, what owns it, and what gets written down for
//! it. [`crate::recogniser`] answers what kind of item begins at an offset and how far it
//! runs; this module decides what to do with the answer.

use crate::reading::declaration::exported::{Exported, Inline, Load_Inline, Named, Tail};
use crate::reading::declaration::members::{Members_In, Reading};
use crate::reading::module_tree::{Item, Module};
use crate::reading::recogniser::{Head, Recognised};
use crate::reading::text::{Line_End, Next_Line};
use std::collections::BTreeMap;

/// What the walker is currently inside.
#[derive(Clone, Copy)]
pub(crate) enum Inside<'a>
{
    /// An inherent `impl Type` block, carrying the type's name.
    ///
    /// Its members need their own `pub`. `Workspace::Name_From_Package_Id` is an inherent
    /// helper with no visibility of its own and is not an export, and a scanner that took
    /// every member of every `impl` for public would list it.
    Implementation(&'a str),
    /// An `impl Trait for Type` block, carrying the type's name.
    ///
    /// Its members are as public as the trait, so they carry no `pub` of their own.
    Conformance(&'a str),
    /// A `pub trait` block, carrying the trait's name.
    Declaration(&'a str),
    /// A `pub struct` body, carrying the struct's name.
    Fields(&'a str),
    /// A `pub enum` body, carrying the enum's name.
    Variants(&'a str),
}

/// The bytes a walk reads, and the classification of them it reads through.
///
/// The two are one value because neither answers anything alone: an offset into the text is
/// only meaningful beside the mask that says whether it is code, a comment or a string.
#[derive(Clone, Copy)]
pub(crate) struct Source<'a>
{
    pub(crate) text: &'a str,
    pub(crate) masks: &'a crate::reading::masks::Masks,
}

/// Where a walk files what it finds.
///
/// Four destinations that are threaded together through every step of the descent: the
/// module being built, the child modules still to descend into, the path this module sits
/// at, and the map every module lands in. Passing them one at a time made eight-argument
/// signatures whose order was the only thing keeping them straight.
pub(crate) struct Filing<'a>
{
    pub(crate) module: &'a mut Module,
    pub(crate) children: &'a mut Vec<(String, bool)>,
    pub(crate) path: &'a [String],
    pub(crate) into: &'a mut BTreeMap<Vec<String>, Module>,
}

/// Collects the public items in a byte range.
pub(crate) fn Walk(source: Source<'_>, range: (usize, usize), inside: Option<Inside<'_>>, filing: &mut Filing<'_>)
{
    let (start, end) = range;
    let mut cursor = start;

    while cursor < end
    {
        if let Some(body) = Member_Body(inside)
        {
            Collect_Members(source, (cursor, end), body, filing);

            return;
        }
        cursor = Advanced(source, (cursor, end), inside, filing);
    }
}

/// Which member list a walk is inside, if it is inside one at all.
fn Member_Body(inside: Option<Inside<'_>>) -> Option<(Reading, &str)>
{
    return match inside
    {
        Some(Inside::Variants(owner)) => Some((Reading::Variants, owner)),
        Some(Inside::Fields(owner)) => Some((Reading::Fields, owner)),
        _ => None,
    };
}

/// Files every member of a struct or enum body, which is the whole of what that body holds.
fn Collect_Members(
    source: Source<'_>,
    range: (usize, usize),
    body: (Reading, &str),
    filing: &mut Filing<'_>,
)
{
    let Source { text, masks } = source;
    let (reading, owner) = body;
    let kind = match reading
    {
        Reading::Variants => "enum variant",
        Reading::Fields => "struct field",
    };

    for (name, carried) in Members_In(text, masks, range, reading)
    {
        filing.module.items.push(Item {
            kind: kind.to_owned(),
            name: format!("{owner}::{name}"),
            tail: carried,
            modifiers: String::new(),
        });
    }
}

/// Files whatever declaration begins at `range.0`, and answers where to look next.
fn Advanced(
    source: Source<'_>,
    range: (usize, usize),
    inside: Option<Inside<'_>>,
    filing: &mut Filing<'_>,
) -> usize
{
    let Source { text, masks } = source;
    let (cursor, end) = range;
    let Some(head) = Head(text, masks, cursor, end)
    else
    {
        let line_end = Line_End(text, cursor, end);

        return Next_Line(text, line_end, end);
    };

    Record(&head, inside, source, filing);

    return Next_Line(text, head.after, end);
}

/// Files one recognised declaration, and descends into it where that is meaningful.
#[allow(clippy::too_many_arguments)]
fn Record(
    head: &Recognised,
    inside: Option<Inside<'_>>,
    source: Source<'_>,
    filing: &mut Filing<'_>,
)
{
    match head.kind.as_str()
    {
        "mod" => Record_Mod(head, source, filing),
        "use" => Record_Use(head, filing),
        "impl" => Record_Impl(head, source, filing),
        _ => Record_Item(head, inside, source, filing),
    }
}

/// A `mod` declaration: an inline body is loaded now, a file is queued for its parent.
fn Record_Mod(head: &Recognised, source: Source<'_>, filing: &mut Filing<'_>)
{
    let Some(name) = Named(&head.text, "mod")
    else
    {
        return;
    };
    let public = head.visibility == "pub";
    let Some(body) = head.body
    else
    {
        filing.children.push((name, public));

        return;
    };
    let inline = Inline {
        name: &name,
        exported: Exported::Of(public),
        body,
    };

    Load_Inline(inline, source, filing.path, filing.into);
}

/// A `pub use`, recorded as a re-export to be resolved once every module is loaded.
fn Record_Use(head: &Recognised, filing: &mut Filing<'_>)
{
    if head.visibility == "pub"
        && let Some(target) = head.text.split_once("use ").map(|(_, rest)| return rest)
    {
        filing.module.re_exports.push(target.trim().to_owned());
    }
}

/// Any other declaration: filed when it is public, then descended into where its members are
/// part of the surface too.
///
/// A trait's items and a trait implementation's items are as public as the trait, so they
/// carry no `pub` of their own. An inherent `impl` is not like that.
fn Record_Item(
    head: &Recognised,
    inside: Option<Inside<'_>>,
    source: Source<'_>,
    filing: &mut Filing<'_>,
)
{
    let associated = matches!(inside, Some(Inside::Conformance(_) | Inside::Declaration(_)));
    let public = head.visibility == "pub" || (associated && head.visibility.is_empty());
    if !public
    {
        return;
    }
    let Some(name) = Named(&head.text, &head.kind)
    else
    {
        return;
    };
    let owner = Owner_Prefix(inside);

    filing.module.items.push(Item {
        kind: head.kind.clone(),
        name: format!("{owner}{name}"),
        tail: Tail(&head.text, &head.kind, &name),
        modifiers: head.modifiers.clone(),
    });
    Descend(head, &name, source, filing);
}

/// The type or trait a member is written under, as a prefix for its own name.
fn Owner_Prefix(inside: Option<Inside<'_>>) -> String
{
    return match inside
    {
        Some(
            Inside::Implementation(owner)
            | Inside::Conformance(owner)
            | Inside::Declaration(owner)
            | Inside::Fields(owner),
        ) => format!("{owner}::"),
        _ => String::new(),
    };
}

/// Walks into the body of a declaration whose members are part of the surface.
#[allow(clippy::too_many_arguments)]
fn Descend(head: &Recognised, name: &str, source: Source<'_>, filing: &mut Filing<'_>)
{
    let Some(body) = head.body
    else
    {
        return;
    };

    let inside = match head.kind.as_str()
    {
        "trait" => Inside::Declaration(name),
        "struct" | "union" => Inside::Fields(name),
        "enum" => Inside::Variants(name),
        _ => return,
    };

    Walk(source, body, Some(inside), filing);
}

/// An `impl` block: the header is surface when it implements a trait, and its `pub`
/// members are surface either way.
fn Record_Impl(head: &Recognised, source: Source<'_>, filing: &mut Filing<'_>)
{
    let Some((subject, implemented)) = Implemented(&head.text)
    else
    {
        return;
    };
    let inside = Header_Of(&subject, implemented.as_deref(), filing.module);
    let Some(body) = head.body
    else
    {
        return;
    };
    let mut ignored = Vec::new();
    Walk(source, body, Some(inside), &mut Filing {
        module: filing.module,
        children: &mut ignored,
        path: filing.path,
        into: filing.into,
    });
}

/// An `impl` header is itself surface when it implements a trait, and the members below it
/// are then as public as that trait; an inherent `impl` contributes only its `pub` members.
fn Header_Of<'a>(subject: &'a str, implemented: Option<&str>, module: &mut Module) -> Inside<'a>
{
    let Some(trait_name) = implemented
    else
    {
        return Inside::Implementation(subject);
    };

    module.items.push(Item {
        kind: "impl".to_owned(),
        name: subject.to_owned(),
        tail: format!(" implements {trait_name}"),
        modifiers: String::new(),
    });

    return Inside::Conformance(subject);
}

/// The type an `impl` header is about, and the trait it implements if it implements one.
///
/// The subject loses its generic arguments — `impl<T: Read> Drain<T>` is `Drain` — because
/// it is only being used to prefix the members below it, and those carry their own types.
/// The trait keeps everything: `From<StoreError>` and `From<rusqlite::Error>` are two
/// different implementations, and reducing both to `From` collapses them into one line
/// that a snapshot then reports as unchanged when one of them is deleted.
fn Implemented(head: &str) -> Option<(String, Option<String>)>
{
    let rest = head.strip_prefix("impl")?.trim_start();
    let rest = Without_Generics(rest);
    let body = rest.split(" where ").next().unwrap_or(&rest).trim().to_owned();

    if let Some((implemented, subject)) = body.split_once(" for ")
    {
        return Some((Bare(subject.trim()), Some(implemented.trim().to_owned())));
    }

    return Some((Bare(body.trim()), None));
}

/// A type expression without its generic arguments.
fn Bare(text: &str) -> String
{
    return text
        .split(['<', ' '])
        .next()
        .unwrap_or(text)
        .trim()
        .to_owned();
}

/// A header with a leading `<…>` generic parameter list removed.
fn Without_Generics(text: &str) -> String
{
    if !text.starts_with('<')
    {
        return text.to_owned();
    }
    let mut depth = 0_i32;
    for (offset, character) in text.char_indices()
    {
        match character
        {
            '<' => depth = depth.saturating_add(1),
            '>' =>
            {
                depth = depth.saturating_sub(1);
                if depth == 0
                {
                    return text
                        .get(offset.saturating_add(1)..)
                        .unwrap_or_default()
                        .trim()
                        .to_owned();
                }
            }
            _ =>
            {}
        }
    }

    return text.to_owned();
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_An_Impl_Header_Should_Lose_Its_Generics_And_Keep_Its_Trait()
    {
        assert_eq!(
            Implemented("impl<T: Read + Send> Reader for Drain<T>"),
            Some(("Drain".to_owned(), Some("Reader".to_owned())))
        );
        assert_eq!(Implemented("impl Catalogue"), Some(("Catalogue".to_owned(), None)));
        // Two implementations of one trait must not reduce to one line.
        assert_ne!(
            Implemented("impl From<StoreError> for ProjectError"),
            Implemented("impl From<rusqlite::Error> for ProjectError")
        );
    }
}
