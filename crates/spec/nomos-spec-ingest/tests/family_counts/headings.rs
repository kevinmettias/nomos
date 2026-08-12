//! Sections, addressed the way the documents number them.
//!
//! A heading is read as the segmenter sees it, which is what keeps a `###` inside a fenced
//! block from being counted as a section. Everything here is a different way of asking which
//! headings a figure covers: by prefix, by an appendix letter and depth of numbering, by
//! ancestor, or by walking a section until the next one at its level.

#![allow(dead_code)]

use crate::volumes::Volume;
use nomos_spec_model::{BlockKind, Segment};
use std::path::Path;

pub(crate) struct Heading
{
    pub(crate) depth: usize,
    pub(crate) title: String,
    pub(crate) path: Vec<String>,
}

/// Headings as the segmenter sees them, which is what keeps a `###` inside a fenced block
/// from being counted as a section.
fn Headings(markdown: &str) -> Vec<Heading>
{
    return Segment(markdown)
        .into_iter()
        .filter(|block| return block.kind == BlockKind::Heading)
        .map(|block| {
            return Heading {
                depth: block.text.chars().take_while(|character| return *character == '#').count(),
                title: block.text.trim_start_matches('#').trim().to_owned(),
                path: block.heading_path,
            };
        })
        .collect();
}

pub(crate) fn Headings_Matching(corpus: &Path, stem: &str, depth: usize, prefixes: &[&str]) -> u32
{
    let markdown = Volume(corpus, stem);
    let matched = Headings(&markdown)
        .iter()
        .filter(|heading| return heading.depth == depth)
        .filter(|heading| {
            return prefixes.iter().any(|prefix| {
                return heading
                    .title
                    .strip_prefix(*prefix)
                    .is_some_and(|rest| return rest.starts_with(|c: char| return c.is_ascii_digit()));
            });
        })
        .count();

    return u32::try_from(matched).unwrap_or(u32::MAX);
}

/// How an appendix numbers its sections: a letter, then that many dotted numbers.
#[derive(Clone, Copy)]
pub(crate) struct Appendix
{
    pub(crate) letter: char,
    pub(crate) parts: usize,
}

/// Appendix sections, addressed the way the documents number them: a letter, then
/// `parts` dotted numbers, then a space.
pub(crate) fn Lettered(corpus: &Path, stem: &str, depth: usize, address: Appendix) -> u32
{
    let Appendix { letter, parts } = address;
    let markdown = Volume(corpus, stem);
    let matched = Headings(&markdown)
        .iter()
        .filter(|heading| return heading.depth == depth)
        .filter(|heading| return Is_Lettered(&heading.title, letter, parts))
        .count();

    return u32::try_from(matched).unwrap_or(u32::MAX);
}

fn Is_Lettered(title: &str, letter: char, parts: usize) -> bool
{
    let Some(rest) = title.strip_prefix(letter)
    else
    {
        return false;
    };
    let Some((numbering, _)) = rest.split_once(' ')
    else
    {
        return false;
    };

    let segments: Vec<&str> = numbering.split('.').collect();
    if segments.len() != parts.saturating_add(1)
    {
        return false;
    }

    return segments.first() == Some(&"")
        && segments
            .iter()
            .skip(1)
            .all(|segment| return !segment.is_empty() && segment.chars().all(|c| return c.is_ascii_digit()));
}

pub(crate) fn Prefixed(corpus: &Path, stem: &str, depth: usize, prefix: &str) -> u32
{
    let markdown = Volume(corpus, stem);
    let matched = Headings(&markdown)
        .iter()
        .filter(|heading| return heading.depth == depth)
        .filter(|heading| {
            return heading
                .title
                .strip_prefix(prefix)
                .is_some_and(|rest| return rest.starts_with(|c: char| return c.is_ascii_digit()));
        })
        .count();

    return u32::try_from(matched).unwrap_or(u32::MAX);
}

pub(crate) fn Under_Path(corpus: &Path, stem: &str, depth: usize, ancestor: &str) -> u32
{
    let markdown = Volume(corpus, stem);
    let matched = Headings(&markdown)
        .iter()
        .filter(|heading| return heading.depth == depth)
        .filter(|heading| return heading.path.iter().any(|step| return step == ancestor))
        .count();

    return u32::try_from(matched).unwrap_or(u32::MAX);
}

pub(crate) fn End_To_End(corpus: &Path) -> u32
{
    let markdown = Volume(corpus, "09-reference");
    let matched = Headings(&markdown)
        .iter()
        .filter(|heading| return heading.depth == 3)
        .filter(|heading| {
            return Is_Lettered(&heading.title, 'G', 1)
                && heading.title.contains("End-to-end scenario:");
        })
        .count();

    return u32::try_from(matched).unwrap_or(u32::MAX);
}

/// Every heading of section 6, its leaves, and the leaves naming a service.
pub(crate) fn Section_Six(corpus: &Path) -> SectionCounts
{
    let markdown = Volume(corpus, "02-core");
    let headings = Headings(&markdown);
    let counted = Counted_Under_Section_Six(&headings);

    assert!(counted.all > 0, "section 6 is no longer in volume 02 under that title");

    return counted;
}

/// The walk itself: everything from section 6's own heading until the next section at its
/// level or above.
fn Counted_Under_Section_Six(headings: &[Heading]) -> SectionCounts
{
    const SECTION: &str = "6. Systems and subsystem responsibilities";
    let mut inside = false;
    let (mut all, mut leaves, mut services) = (0_u32, 0_u32, 0_u32);
    for heading in headings
    {
        if heading.title == SECTION
        {
            inside = true;
        }
        else if inside && heading.depth <= 2
        {
            break;
        }
        if inside
        {
            all = all.saturating_add(1);
            leaves = leaves.saturating_add(u32::from(heading.depth == 4));
            services = services.saturating_add(u32::from(Names_A_Service(heading)));
        }
    }

    return SectionCounts {
        all,
        leaves,
        services,
    };
}

/// Whether a leaf heading names a service, which is the count the register quotes.
fn Names_A_Service(heading: &Heading) -> bool
{
    if heading.depth != 4
    {
        return false;
    }

    return heading.title.split_whitespace().any(|word| return word == "Service");
}

/// What section 6 amounts to: every heading, the leaves, and the leaves naming a service.
///
/// Named rather than a triple of `u32`. At three members of one type a caller is counting
/// positions and the compiler is helping with none of it.
pub(crate) struct SectionCounts
{
    pub(crate) all: u32,
    pub(crate) leaves: u32,
    pub(crate) services: u32,
}
