//! Deciding what became of one member of a family.

use super::{Member, Later, BTreeMap, MemberFate, Fate, Restored, Position, Body, Hollow};

pub(super) fn Judge(member: &Member, later: &Later, documents: &BTreeMap<String, String>) -> MemberFate
{
    return MemberFate {
        id: member.id.clone(),
        family: member.family,
        name: member.name.clone(),
        was: member.document.clone(),
        fate: Still(member, later).unwrap_or_else(|| return Mentions(&member.name, documents)),
    };
}

/// What the later revision still says about a member, if it says anything.
///
/// A single preserving position settles it, so the walk returns on the first one and keeps
/// a hollowed position only as the answer of last resort. A member preserved in one document
/// and hollow in another is preserved.
pub(super) fn Still(member: &Member, later: &Later) -> Option<Fate>
{
    if let Some(named) = Named_In_Row(member, later)
    {
        return Some(named);
    }

    let positions = later.authored.get(&member.name)?;
    let mut hollow: Option<Fate> = None;

    for position in positions
    {
        match At(position)
        {
            preserved @ Fate::Preserved { .. } => return Some(preserved),
            hollowed =>
            {
                hollow.get_or_insert(hollowed);
            }
        }
    }

    return hollow;
}

/// A domain model named in a table row, which is preservation for that family alone.
///
/// Only the canonical domain model is carried in rows; for every other family a row is a
/// mention, and treating it as preservation would report a deleted member as surviving.
pub(super) fn Named_In_Row(member: &Member, later: &Later) -> Option<Fate>
{
    if member.family != Restored::CanonicalDomainModel
    {
        return None;
    }

    let document = later.named_in_row.get(&member.name)?;

    return Some(Fate::Preserved {
        document: document.clone(),
    });
}

/// What one position amounts to on its own.
pub(super) fn At(position: &Position) -> Fate
{
    return match position
    {
        Position::Row { document } | Position::Heading {
            document,
            body: Some(Body::Narrative),
        } => Fate::Preserved {
            document: document.clone(),
        },
        Position::Heading {
            document,
            body: Some(Body::Template {
                shared_with,
                declared,
            }),
        } => Fate::Hollowed {
            document: document.clone(),
            evidence: Hollow::Template {
                shared_with: *shared_with,
                declared: *declared,
            },
        },
        Position::Heading {
            document,
            body: None,
        } => Fate::Hollowed {
            document: document.clone(),
            evidence: Hollow::NoBody,
        },
    };
}

pub(super) fn Mentions(name: &str, documents: &BTreeMap<String, String>) -> Fate
{
    let found: Vec<String> = documents
        .iter()
        .filter(|(_, text)| return text.contains(name))
        .map(|(path, _)| return path.clone())
        .collect();

    if found.is_empty()
    {
        return Fate::Gone;
    }

    return Fate::Mentioned { documents: found };
}
