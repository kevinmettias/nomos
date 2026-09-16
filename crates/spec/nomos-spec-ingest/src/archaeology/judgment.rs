//! Deciding what became of one member of a family.

use super::{Member, Later, BTreeMap, MemberFate, Fate, Restored, Position, Body, Hollow};

pub(super) fn Judge_Member(member: &Member, later: &Later, documents: &BTreeMap<String, String>) -> MemberFate
{
    return MemberFate {
        id: member.id.clone(),
        family: member.family,
        name: member.name.clone(),
        was: member.document.clone(),
        fate: Still_In_Later(member, later).unwrap_or_else(|| return Mentions_In_Documents(&member.name, documents)),
    };
}

/// What the later revision still says about a member, if it says anything.
///
/// A single preserving position settles it, so the walk returns on the first one and keeps
/// a hollowed position only as the answer of last resort. A member preserved in one document
/// and hollow in another is preserved.
pub(super) fn Still_In_Later(member: &Member, later: &Later) -> Option<Fate>
{
    if let Some(named) = Named_In_Row(member, later)
    {
        return Some(named);
    }

    let positions = later.authored.get(&member.name)?;
    let mut hollow: Option<Fate> = None;

    for position in positions
    {
        match Fate_Of_Position(position)
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
pub(super) fn Fate_Of_Position(position: &Position) -> Fate
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

pub(super) fn Mentions_In_Documents(name: &str, documents: &BTreeMap<String, String>) -> Fate
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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Origin;
    use std::collections::BTreeSet;

    /// The reach the template fixture below is built with. It is the fixture's own choice of
    /// a shared-body count, and the assertion reads the same constant so the two agree on
    /// which number the `Hollow::Template` evidence is expected to carry through.
    const SECTIONS_SHARING_THE_TEMPLATE_BODY: u32 = 3;

    #[test]
    fn Test_Judge_Member_Should_Combine_Identity_With_Its_Computed_Fate()
    {
        let member = A_Member("WorkspaceContext", Restored::CanonicalDomainModel);
        let documents = Documents_By_Path(&[("a.md", "# A\n\nThe WorkspaceContext is discussed.\n")]);
        let later = Later::Read(&documents);

        let judged = Judge_Member(&member, &later, &documents);

        assert_eq!(judged.id, member.id);
        assert_eq!(judged.name, "WorkspaceContext");
        assert_eq!(judged.fate, Fate::Mentioned { documents: vec!["a.md".to_owned()] });
    }

    #[test]
    fn Test_Still_In_Later_Should_Prefer_A_Preserving_Position_Over_A_Hollow_One()
    {
        let member = A_Member("Widget", Restored::Service);
        let mut later = Empty_Later();
        later.authored.insert(
            "Widget".to_owned(),
            vec![
                Position::Heading { document: "a.md".to_owned(), body: None },
                Position::Row { document: "b.md".to_owned() },
            ],
        );

        let fate = Still_In_Later(&member, &later);

        assert_eq!(fate, Some(Fate::Preserved { document: "b.md".to_owned() }));
    }

    #[test]
    fn Test_Named_In_Row_Should_Only_Apply_To_The_Canonical_Domain_Model()
    {
        let mut later = Empty_Later();
        later.named_in_row.insert("WorkspaceContext".to_owned(), "a.md".to_owned());
        let model = A_Member("WorkspaceContext", Restored::CanonicalDomainModel);
        let other_family_member = A_Member("WorkspaceContext", Restored::Service);

        assert_eq!(Named_In_Row(&model, &later), Some(Fate::Preserved { document: "a.md".to_owned() }));
        assert_eq!(Named_In_Row(&other_family_member, &later), None);
    }

    #[test]
    fn Test_Fate_Of_Position_Should_Map_Each_Shape_To_Its_Own_Fate()
    {
        assert_eq!(
            Fate_Of_Position(&Position::Row { document: "a.md".to_owned() }),
            Fate::Preserved { document: "a.md".to_owned() }
        );
        assert_eq!(
            Fate_Of_Position(&Position::Heading { document: "a.md".to_owned(), body: Some(Body::Narrative) }),
            Fate::Preserved { document: "a.md".to_owned() }
        );
        assert_eq!(
            Fate_Of_Position(&Position::Heading {
                document: "a.md".to_owned(),
                body: Some(Body::Template { shared_with: SECTIONS_SHARING_THE_TEMPLATE_BODY, declared: None }),
            }),
            Fate::Hollowed {
                document: "a.md".to_owned(),
                evidence: Hollow::Template { shared_with: SECTIONS_SHARING_THE_TEMPLATE_BODY, declared: None },
            }
        );
        assert_eq!(
            Fate_Of_Position(&Position::Heading { document: "a.md".to_owned(), body: None }),
            Fate::Hollowed { document: "a.md".to_owned(), evidence: Hollow::NoBody }
        );
    }

    #[test]
    fn Test_Mentions_In_Documents_Should_List_Every_Document_Containing_The_Name()
    {
        let documents = Documents_By_Path(&[
            ("a.md", "The WorkspaceContext is discussed.\n"),
            ("b.md", "Nothing of the kind.\n"),
            ("c.md", "WorkspaceContext appears here too.\n"),
        ]);

        assert_eq!(
            Mentions_In_Documents("WorkspaceContext", &documents),
            Fate::Mentioned { documents: vec!["a.md".to_owned(), "c.md".to_owned()] }
        );
        assert_eq!(Mentions_In_Documents("Nowhere", &documents), Fate::Gone);
    }

    fn Documents_By_Path(pairs: &[(&str, &str)]) -> BTreeMap<String, String>
    {
        return pairs.iter().map(|(path, text)| return ((*path).to_owned(), (*text).to_owned())).collect();
    }

    fn A_Member(name: &str, family: Restored) -> Member
    {
        return Member {
            id: format!("TEST-{name}"),
            family,
            name: name.to_owned(),
            document: "source.md".to_owned(),
            origin: Origin::Block { ordinal: 0 },
            alias: None,
        };
    }

    fn Empty_Later() -> Later
    {
        return Later {
            authored: BTreeMap::new(),
            named_in_row: BTreeMap::new(),
            templates: BTreeMap::new(),
            declared: BTreeSet::new(),
            bodies: BTreeMap::new(),
        };
    }
}
