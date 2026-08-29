//! Recognising a family member in a heading or a table row.

use super::{
    BlockKind, BTreeMap, Collision, DOMAIN_MODEL, EXTENDED_TERMS, GLOSSARY, IngestError, Member, Origin, Restored,
    RowKind, Segment, SERVICES, SourceBlock, Table_Rows, TableRow,
};
use nomos_spec_store::DocumentPath;

/// Reads one volume's family members without touching a store.
///
/// Pure, so the recognition can be tested against a fixture rather than only against the
/// corpus, and so a caller can see what a restoration would mint before it mints it.
///
/// # Errors
///
/// Returns [`IngestError::Parse`] naming both members when two would take one identifier.
pub(crate) fn Extract(document: DocumentPath<'_>, markdown: &str) -> Result<Vec<Member>, IngestError>
{
    let document = document.0;
    let mut members: Vec<Member> = Vec::new();

    for block in &Segment(markdown)
    {
        if block.kind == BlockKind::Heading
        {
            From_Heading(document, block, &mut members);
        }
        else
        {
            From_Rows(document, block, &mut members);
        }
    }

    Refuse_Collisions(&members)?;

    return Ok(members);
}

pub(super) fn Refuse_Collisions(members: &[Member]) -> Result<(), IngestError>
{
    let mut seen: BTreeMap<&str, &str> = BTreeMap::new();

    for member in members
    {
        if let Some(first) = seen.insert(member.id.as_str(), member.name.as_str())
        {
            return Err(IngestError::Parse(
                Collision {
                    id: member.id.clone(),
                    first: first.to_owned(),
                    second: member.name.clone(),
                }
                .to_string(),
            ));
        }
    }

    return Ok(());
}

/// The volume is checked once here rather than at each recognition, because a heading only
/// belongs to a family if the document it sits in is that family's volume — asking once is
/// what keeps a newly added recognition from forgetting to ask at all.
pub(super) fn From_Heading(document: &str, block: &SourceBlock, members: &mut Vec<Member>)
{
    let title = block.text.trim_start_matches('#').trim();
    let origin = Origin::Block { ordinal: block.ordinal };

    for (family, key, alias) in Recognized(block, title)
    {
        if !document.starts_with(family.Volume())
        {
            continue;
        }
        members.push(Member {
            id: Identify(family, &key),
            family,
            name: title.to_owned(),
            document: document.to_owned(),
            origin,
            alias,
        });
    }
}

/// A heading recognised as a family member: the family, the key its identifier is minted
/// from, and the alias it answers to when it answers to one.
pub(super) type Recognition = (Restored, String, Option<String>);

/// What a heading names, judged by how deep it sits.
///
/// Depth is what separates a family member from a section that merely mentions one, so a
/// heading at any other level names nothing at all.
pub(super) fn Recognized(block: &SourceBlock, title: &str) -> Vec<Recognition>
{
    let depth = block.text.chars().take_while(|character| return *character == '#').count();

    return match depth
    {
        LEVEL_3 => At_Depth_3(title),
        LEVEL_4 => At_Depth_4(title, &block.heading_path),
        _ => Vec::new(),
    };
}

/// The two heading levels a family member can sit at. Anything shallower is a volume and
/// anything deeper is a subsection of a member rather than a member.
const LEVEL_3: usize = 3;
const LEVEL_4: usize = 4;

/// A level-4 heading's numbering carries two parts, `D.1.2`, where a level-3 one carries one.
const PARTS_AT_DEPTH_4: usize = 2;

/// The level-3 numbered series, and the family each one restores.
const DEPTH_3: &[(char, Restored)] = &[('D', Restored::AppendixD), ('H', Restored::AppendixH)];

/// The level-4 numbered series, and the family each one restores.
const DEPTH_4: &[(char, Restored)] = &[
    ('D', Restored::AppendixD),
    ('E', Restored::HeadlessInventory),
    ('F', Restored::IdeProfile),
];

/// Every family in `series` whose letter this title carries at `parts` levels of numbering.
pub(super) fn Numbered(title: &str, parts: usize, series: &[(char, Restored)]) -> Vec<Recognition>
{
    let mut found = Vec::new();

    for (letter, family) in series
    {
        if let Some(numbering) = Numbering(title, *letter, parts)
        {
            found.push((*family, numbering, None));
        }
    }

    return found;
}

/// What a level-3 heading names.
///
/// A `G` numbering is a scenario only when the title says so, because appendix G numbers
/// its prose sections in the same series as its scenarios.
pub(super) fn At_Depth_3(title: &str) -> Vec<Recognition>
{
    let mut found = Numbered(title, 1, DEPTH_3);

    if let Some(numbering) = Milestone(title)
    {
        found.push((Restored::RoadmapMilestone, numbering, None));
    }
    if let Some(numbering) = Numbering(title, 'G', 1)
        && title.contains("End-to-end scenario:")
    {
        found.push((Restored::Scenario, numbering, None));
    }

    return found;
}

/// What a level-4 heading names.
///
/// Services and glossary terms are recognised by where they sit rather than by a numbering,
/// because neither carries one — the heading path is the only thing that distinguishes a
/// service from any other level-4 heading in the same volume.
pub(super) fn At_Depth_4(title: &str, path: &[String]) -> Vec<Recognition>
{
    let mut found = Numbered(title, PARTS_AT_DEPTH_4, DEPTH_4);

    if Under(path, SERVICES) && Names_A_Service(title)
    {
        found.push((Restored::Service, title.to_owned(), None));
    }
    if Under(path, EXTENDED_TERMS)
    {
        found.push((Restored::GlossaryTerm, title.to_owned(), Some(title.to_owned())));
    }

    return found;
}

/// Whether a heading calls its subject a service in so many words.
fn Names_A_Service(title: &str) -> bool
{
    return title.split_whitespace().any(|word| return word == "Service");
}

pub(super) fn From_Rows(document: &str, block: &SourceBlock, members: &mut Vec<Member>)
{
    let Some(family) = Tabled_Family(block)
    else
    {
        return;
    };
    if !document.starts_with(family.Volume())
    {
        return;
    }

    for row in Table_Rows(block).iter().filter(|row| return row.kind == RowKind::Content)
    {
        let origin = Origin::Row {
            block_ordinal: block.ordinal,
            row_ordinal: row.ordinal,
        };
        for name in Named_By(row, family)
        {
            let member = Tabled(document, family, name, origin);
            members.push(member);
        }
    }
}

/// The family a table under this heading declares, if it declares one.
pub(super) fn Tabled_Family(block: &SourceBlock) -> Option<Restored>
{
    return match block.heading_path.last().map(String::as_str)
    {
        Some(DOMAIN_MODEL) => Some(Restored::CanonicalDomainModel),
        Some(GLOSSARY) => Some(Restored::GlossaryTerm),
        _ => None,
    };
}

/// One member as a table row declares it.
///
/// It carries an alias where a heading may not: a row's first cell is the name the rest of
/// the corpus refers to it by, whereas a heading's text is a sentence about it.
pub(super) fn Tabled(document: &str, family: Restored, name: &str, origin: Origin) -> Member
{
    return Member {
        id: Identify(family, name),
        family,
        name: name.to_owned(),
        document: document.to_owned(),
        origin,
        alias: Some(name.to_owned()),
    };
}

/// The members one content row declares.
///
/// Only the canonical domain model splits a cell: nine of its rows name more than one
/// model, and every other family's first cell is one name however it reads.
pub(super) fn Named_By(row: &TableRow, family: Restored) -> Vec<&str>
{
    let Some(cell) = First_Cell(row)
    else
    {
        return Vec::new();
    };

    if family == Restored::CanonicalDomainModel
    {
        return Models_In(cell);
    }

    return vec![cell];
}

/// The models one row of the canonical domain model names.
///
/// Nine of its 28 rows name more than one — `ModelUsageObservation and CostObservation`,
/// `Gate, Phase, Workflow` — so the row count is not the model count. The done-when
/// requires `ModelUsageObservation` to resolve individually and it shares a row, which
/// settles the reading: the row is the canonical source and the concepts minted from it
/// are projections of it. That is the property the restoration exists to establish, not an
/// exception to it.
///
/// Applied to this table only. A glossary term that happens to contain the word is one
/// term, and splitting it would invent two.
#[must_use]
pub fn Models_In(cell: &str) -> Vec<&str>
{
    return cell
        .split(',')
        .flat_map(|part| return part.split(" and "))
        .map(str::trim)
        .filter(|name| return !name.is_empty())
        .collect();
}

/// The first non-empty cell of a row, which is the name the table gives its subject.
pub(super) fn First_Cell(row: &TableRow) -> Option<&str>
{
    return row
        .cells
        .iter()
        .map(|cell| return cell.trim())
        .find(|cell| return !cell.is_empty());
}

pub(super) fn Under(path: &[String], heading: &str) -> bool
{
    return path.iter().any(|step| return step == heading);
}

/// `Foundation 0` and `Release 3`, as the roadmap numbers itself.
///
/// Both are milestones; only seven of the eight are Releases, which is why the family is
/// named for the milestone and not for the release.
pub(super) fn Milestone(title: &str) -> Option<String>
{
    let mut words = title.split_whitespace();
    let word = words.next()?;
    let number = words.next()?;

    if word != "Foundation" && word != "Release"
    {
        return None;
    }
    if !All_Digits(number)
    {
        return None;
    }

    let initial = word.get(..1)?;
    return Some(format!("{initial}.{number}"));
}

/// `D.7.1` from `D.7.1 atlas-profile.build-cost`, when it has exactly `parts` numbers.
pub(super) fn Numbering(title: &str, letter: char, parts: usize) -> Option<String>
{
    let rest = title.strip_prefix(letter)?;
    let (numbering, _) = rest.split_once(' ')?;

    let segments: Vec<&str> = numbering.split('.').collect();
    if segments.len() != parts.checked_add(1)?
    {
        return None;
    }
    if segments.first() != Some(&"")
    {
        return None;
    }
    if !segments.iter().skip(1).all(|segment| return All_Digits(segment))
    {
        return None;
    }

    return Some(format!("{letter}{numbering}"));
}

/// The identifier, minted from the family's prefix and what the document already says.
pub(super) fn Identify(family: Restored, name: &str) -> String
{
    return format!("{}-{}", family.Prefix(), Slug(name));
}

pub(super) fn Slug(name: &str) -> String
{
    let mut slug = String::new();
    let mut pending = false;

    for character in name.chars()
    {
        if character.is_ascii_alphanumeric()
        {
            if pending && !slug.is_empty()
            {
                slug.push('-');
            }
            pending = false;
            slug.extend(character.to_uppercase());
        }
        else
        {
            pending = true;
        }
    }

    return slug;
}

/// A non-empty run of ASCII digits and nothing else.
fn All_Digits(text: &str) -> bool
{
    return !text.is_empty() && text.bytes().all(|byte| return byte.is_ascii_digit());
}
