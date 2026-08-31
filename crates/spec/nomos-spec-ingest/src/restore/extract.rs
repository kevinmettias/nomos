//! Recognising a family member in a heading or a table row.

// file-size: allow this file pairs its production code with its own inline #[cfg(test)]
// module; check-test-coverage keys a test's companion unit off the exact file it is
// textually written in, so these tests cannot move to a sibling file without losing
// their attribution to every function this file declares.
// responsibility: allow same reason -- the coupling that keeps this file whole is
// check-test-coverage's stem-based companion attribution, not a design choice.

use super::{
    BlockKind, BTreeMap, Collision, DOMAIN_MODEL, EXTENDED_TERMS, GLOSSARY, IngestError, Member, Origin, Restored,
    RowKind, Segment, SYSTEMS_HEADING, SourceBlock, Table_Rows, TableRow,
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
pub(crate) fn Extract_Members(document: DocumentPath<'_>, markdown: &str) -> Result<Vec<Member>, IngestError>
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

    for (family, key, alias) in Recognized_In_Heading(block, title)
    {
        if !document.starts_with(family.Volume())
        {
            continue;
        }
        members.push(Member {
            id: Identify_Member(family, &key),
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
pub(super) fn Recognized_In_Heading(block: &SourceBlock, title: &str) -> Vec<Recognition>
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
pub(super) fn Numbered_Families(title: &str, parts: usize, series: &[(char, Restored)]) -> Vec<Recognition>
{
    let mut found = Vec::new();

    for (letter, family) in series
    {
        if let Some(numbering) = Numbering_In(title, *letter, parts)
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
    let mut found = Numbered_Families(title, 1, DEPTH_3);

    if let Some(numbering) = Milestone_Numbering(title)
    {
        found.push((Restored::RoadmapMilestone, numbering, None));
    }
    if let Some(numbering) = Numbering_In(title, 'G', 1)
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
    let mut found = Numbered_Families(title, PARTS_AT_DEPTH_4, DEPTH_4);

    if Has_Ancestor(path, SYSTEMS_HEADING) && Has_The_Word_Service(title)
    {
        found.push((Restored::Service, title.to_owned(), None));
    }
    if Has_Ancestor(path, EXTENDED_TERMS)
    {
        found.push((Restored::GlossaryTerm, title.to_owned(), Some(title.to_owned())));
    }

    return found;
}

/// Whether a heading calls its subject a service in so many words.
fn Has_The_Word_Service(title: &str) -> bool
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
            let member = Tabled_Member(document, family, name, origin);
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
pub(super) fn Tabled_Member(document: &str, family: Restored, name: &str, origin: Origin) -> Member
{
    return Member {
        id: Identify_Member(family, name),
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

pub(super) fn Has_Ancestor(path: &[String], heading: &str) -> bool
{
    return path.iter().any(|step| return step == heading);
}

/// `Foundation 0` and `Release 3`, as the roadmap numbers itself.
///
/// Both are milestones; only seven of the eight are Releases, which is why the family is
/// named for the milestone and not for the release.
pub(super) fn Milestone_Numbering(title: &str) -> Option<String>
{
    let mut words = title.split_whitespace();
    let word = words.next()?;
    let number = words.next()?;

    if word != "Foundation" && word != "Release"
    {
        return None;
    }
    if !Is_All_Digits(number)
    {
        return None;
    }

    let initial = word.get(..1)?;
    return Some(format!("{initial}.{number}"));
}

/// `D.7.1` from `D.7.1 atlas-profile.build-cost`, when it has exactly `parts` numbers.
pub(super) fn Numbering_In(title: &str, letter: char, parts: usize) -> Option<String>
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
    if !segments.iter().skip(1).all(|segment| return Is_All_Digits(segment))
    {
        return None;
    }

    return Some(format!("{letter}{numbering}"));
}

/// The identifier, minted from the family's prefix and what the document already says.
pub(super) fn Identify_Member(family: Restored, name: &str) -> String
{
    return format!("{}-{}", family.Prefix(), Slug_Of(name));
}

pub(super) fn Slug_Of(name: &str) -> String
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
fn Is_All_Digits(text: &str) -> bool
{
    return !text.is_empty() && text.bytes().all(|byte| return byte.is_ascii_digit());
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Extract_Members_Should_Collect_Heading_And_Row_Members()
    {
        let markdown = "# Core\n\n## 5. Canonical domain model\n\n\
                        | Model | Responsibility |\n| --- | --- |\n\
                        | WorkspaceContext | Repository. |\n\n\
                        ## 6. Systems and subsystem responsibilities\n\n\
                        ### 6.1 Change reasoning\n\n\
                        #### Counterfactual Analysis Service\n\nEvaluates proposals.\n";

        let members = Extract_Members(DocumentPath("02-core.md"), markdown).expect("extracts");

        assert_eq!(
            members.iter().filter(|member| return member.family == Restored::CanonicalDomainModel).count(),
            1
        );
        assert_eq!(members.iter().filter(|member| return member.family == Restored::Service).count(), 1);
    }

    #[test]
    fn Test_Refuse_Collisions_Should_Reject_A_Repeated_Identifier()
    {
        let one = Member {
            id: "GLS-APPLICABILITY".to_owned(),
            family: Restored::GlossaryTerm,
            name: "Applicability".to_owned(),
            document: "09-reference.md".to_owned(),
            origin: Origin::Row {
                block_ordinal: 1,
                row_ordinal: 1,
            },
            alias: Some("Applicability".to_owned()),
        };
        let two = Member {
            name: "applicability".to_owned(),
            origin: Origin::Row {
                block_ordinal: 1,
                row_ordinal: 2,
            },
            ..one.clone()
        };

        let refusal = Refuse_Collisions(&[one, two]).expect_err("must refuse");

        assert!(format!("{refusal}").contains("GLS-APPLICABILITY"), "{refusal}");
    }

    #[test]
    fn Test_From_Heading_Should_Mint_A_Member_When_The_Document_Matches_The_Family()
    {
        let block = Block("### D.7 Profiles");
        let mut members = Vec::new();

        From_Heading("09-reference.md", &block, &mut members);

        assert_eq!(members.len(), 1);
        assert_eq!(members.first().map(|member| member.family), Some(Restored::AppendixD));
        assert_eq!(members.first().map(|member| member.name.as_str()), Some("D.7 Profiles"));

        let mut wrong_volume = Vec::new();
        From_Heading("07-clients.md", &block, &mut wrong_volume);
        assert!(wrong_volume.is_empty(), "a heading recognized outside its volume was still minted");
    }

    #[test]
    fn Test_Recognized_In_Heading_Should_Read_The_Heading_Depth()
    {
        let deep_enough = Block("### D.7 Profiles");
        let recognitions = Recognized_In_Heading(&deep_enough, "D.7 Profiles");
        assert_eq!(recognitions, vec![(Restored::AppendixD, "D.7".to_owned(), None)]);

        let shallow = Block("## Not a member");
        assert!(Recognized_In_Heading(&shallow, "Not a member").is_empty());
    }

    #[test]
    fn Test_Numbered_Families_Should_Match_Every_Letter_In_The_Series()
    {
        let found = Numbered_Families("D.7 Profiles", 1, DEPTH_3);
        assert_eq!(found, vec![(Restored::AppendixD, "D.7".to_owned(), None)]);

        assert!(Numbered_Families("Z.7 Nothing", 1, DEPTH_3).is_empty());
    }

    #[test]
    fn Test_At_Depth_3_Should_Recognize_A_Numbered_Family_A_Milestone_And_A_Scenario()
    {
        assert_eq!(At_Depth_3("D.7 Profiles"), vec![(Restored::AppendixD, "D.7".to_owned(), None)]);
        assert_eq!(
            At_Depth_3("Foundation 0 — Protocol"),
            vec![(Restored::RoadmapMilestone, "F.0".to_owned(), None)]
        );
        assert_eq!(
            At_Depth_3("G.2 End-to-end scenario: add a strategy"),
            vec![(Restored::Scenario, "G.2".to_owned(), None)]
        );
    }

    #[test]
    fn Test_At_Depth_4_Should_Recognize_A_Numbered_Family_A_Service_And_A_Glossary_Term()
    {
        let systems_path = vec![SYSTEMS_HEADING.to_owned()];
        let extended_path = vec![EXTENDED_TERMS.to_owned()];

        assert_eq!(
            At_Depth_4("E.1.1 Legacy client", &[]),
            vec![(Restored::HeadlessInventory, "E.1.1".to_owned(), None)]
        );
        assert_eq!(
            At_Depth_4("Counterfactual Analysis Service", &systems_path),
            vec![(Restored::Service, "Counterfactual Analysis Service".to_owned(), None)]
        );
        assert_eq!(
            At_Depth_4("SavedView", &extended_path),
            vec![(Restored::GlossaryTerm, "SavedView".to_owned(), Some("SavedView".to_owned()))]
        );
    }

    #[test]
    fn Test_From_Rows_Should_Mint_A_Member_Per_Content_Row_In_Its_Volume()
    {
        let block = SourceBlock {
            ordinal: 5,
            kind: BlockKind::Prose,
            heading_path: vec!["Reference".to_owned(), GLOSSARY.to_owned()],
            text: "| Term | Definition |\n| --- | --- |\n| Applicability | Whether a rule can run. |\n".to_owned(),
        };
        let mut members = Vec::new();

        From_Rows("09-reference.md", &block, &mut members);

        assert_eq!(members.len(), 1);
        assert_eq!(members.first().map(|member| member.id.as_str()), Some("GLS-APPLICABILITY"));

        let mut wrong_volume = Vec::new();
        From_Rows("02-core.md", &block, &mut wrong_volume);
        assert!(wrong_volume.is_empty(), "a table recognized outside its volume was still minted");
    }

    #[test]
    fn Test_Tabled_Family_Should_Read_The_Heading_Paths_Last_Step()
    {
        let domain_model = SourceBlock {
            ordinal: 1,
            kind: BlockKind::Prose,
            heading_path: vec![DOMAIN_MODEL.to_owned()],
            text: String::new(),
        };
        let glossary = SourceBlock {
            ordinal: 2,
            kind: BlockKind::Prose,
            heading_path: vec![GLOSSARY.to_owned()],
            text: String::new(),
        };
        let neither = SourceBlock {
            ordinal: 3,
            kind: BlockKind::Prose,
            heading_path: vec!["Something else".to_owned()],
            text: String::new(),
        };

        assert_eq!(Tabled_Family(&domain_model), Some(Restored::CanonicalDomainModel));
        assert_eq!(Tabled_Family(&glossary), Some(Restored::GlossaryTerm));
        assert_eq!(Tabled_Family(&neither), None);
    }

    #[test]
    fn Test_Tabled_Member_Should_Carry_The_Rows_Name_As_Its_Alias()
    {
        let origin = Origin::Row {
            block_ordinal: 2,
            row_ordinal: 3,
        };

        let member = Tabled_Member("09-reference.md", Restored::GlossaryTerm, "Applicability", origin);

        assert_eq!(member.id, "GLS-APPLICABILITY");
        assert_eq!(member.name, "Applicability");
        assert_eq!(member.document, "09-reference.md");
        assert_eq!(member.alias, Some("Applicability".to_owned()));
        assert_eq!(member.origin, origin);
    }

    #[test]
    fn Test_Named_By_Should_Split_The_Domain_Models_Cell_And_Read_Others_Whole()
    {
        let split_row = TableRow {
            ordinal: 1,
            table_ordinal: 1,
            kind: RowKind::Content,
            cells: vec![
                "ModelUsageObservation and CostObservation".to_owned(),
                "Two models.".to_owned(),
            ],
            text: String::new(),
        };
        let whole_row = TableRow {
            ordinal: 2,
            table_ordinal: 1,
            kind: RowKind::Content,
            cells: vec!["Applicability".to_owned(), "One term.".to_owned()],
            text: String::new(),
        };

        assert_eq!(
            Named_By(&split_row, Restored::CanonicalDomainModel),
            vec!["ModelUsageObservation", "CostObservation"]
        );
        assert_eq!(Named_By(&whole_row, Restored::GlossaryTerm), vec!["Applicability"]);
    }

    #[test]
    fn Test_Models_In_Should_Split_On_Commas_And_The_Word_And()
    {
        assert_eq!(Models_In("Gate, Phase, Workflow"), vec!["Gate", "Phase", "Workflow"]);
        assert_eq!(
            Models_In("ModelUsageObservation and CostObservation"),
            vec!["ModelUsageObservation", "CostObservation"]
        );
        assert_eq!(Models_In("WorkspaceContext"), vec!["WorkspaceContext"]);
    }

    #[test]
    fn Test_First_Cell_Should_Skip_Empty_Cells_And_Return_The_First_One_With_Content()
    {
        let row = TableRow {
            ordinal: 1,
            table_ordinal: 1,
            kind: RowKind::Content,
            cells: vec![String::new(), "  ".to_owned(), "Applicability".to_owned()],
            text: String::new(),
        };
        assert_eq!(First_Cell(&row), Some("Applicability"));

        let empty = TableRow {
            ordinal: 2,
            table_ordinal: 1,
            kind: RowKind::Content,
            cells: vec![String::new(), "  ".to_owned()],
            text: String::new(),
        };
        assert_eq!(First_Cell(&empty), None);
    }

    #[test]
    fn Test_Has_Ancestor_Should_Find_A_Step_Anywhere_In_The_Path()
    {
        let path = vec!["Reference".to_owned(), GLOSSARY.to_owned(), "Extended operational terms".to_owned()];

        assert!(Has_Ancestor(&path, GLOSSARY));
        assert!(!Has_Ancestor(&path, SYSTEMS_HEADING));
    }

    #[test]
    fn Test_Milestone_Numbering_Should_Read_Foundation_And_Release_Titles()
    {
        assert_eq!(Milestone_Numbering("Foundation 0 — Protocol"), Some("F.0".to_owned()));
        assert_eq!(Milestone_Numbering("Release 7 — Advanced"), Some("R.7".to_owned()));
        assert_eq!(Milestone_Numbering("Release notes"), None);
    }

    #[test]
    fn Test_Numbering_In_Should_Require_The_Exact_Depth_Of_Numbers()
    {
        assert_eq!(Numbering_In("D.7 Profiles", 'D', 1), Some("D.7".to_owned()));
        assert_eq!(Numbering_In("D.7 Profiles", 'D', 2), None);
        assert_eq!(Numbering_In("D.7.1 atlas", 'D', 2), Some("D.7.1".to_owned()));
        assert_eq!(Numbering_In("Design notes", 'D', 1), None);
    }

    #[test]
    fn Test_Identify_Member_Should_Mint_An_Identifier_From_The_Family_Prefix_And_Slug()
    {
        assert_eq!(
            Identify_Member(Restored::Service, "Counterfactual Analysis Service"),
            "SVC-COUNTERFACTUAL-ANALYSIS-SERVICE"
        );
        assert_eq!(Identify_Member(Restored::GlossaryTerm, "Applicability"), "GLS-APPLICABILITY");
    }

    #[test]
    fn Test_Slug_Of_Should_Join_Alnum_Runs_With_A_Single_Dash()
    {
        assert_eq!(Slug_Of("WorkspaceContext"), "WORKSPACECONTEXT");
        assert_eq!(Slug_Of("Counterfactual Analysis Service"), "COUNTERFACTUAL-ANALYSIS-SERVICE");
        assert_eq!(Slug_Of("  Leading and trailing  "), "LEADING-AND-TRAILING");
    }

    fn Block(text: &str) -> SourceBlock
    {
        return SourceBlock {
            ordinal: 1,
            kind: BlockKind::Heading,
            heading_path: Vec::new(),
            text: text.to_owned(),
        };
    }
}
