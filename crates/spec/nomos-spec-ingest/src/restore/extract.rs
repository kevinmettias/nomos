//! Recognising a family member in a heading or a table row.

use super::{
    BlockKind, BTreeMap, Collision, DOMAIN_MODEL, GLOSSARY, IngestError, Member, Origin, Restored, RowKind, Segment,
    SourceBlock, Table_Rows, TableRow,
};
use nomos_spec_store::DocumentPath;

/// Visible to [`super`] as well as here because the numbering readings are what
/// `restore::tests` drives directly.
pub(super) mod numbering;

use numbering::{Identify_Member, Recognized_In_Heading};

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

/// A heading block carrying the given text, for the tests in this file and in [`numbering`].
///
/// It sits at module scope rather than inside either test module because both need it, and
/// a private item here is visible to both.
#[cfg(test)]
fn Block(text: &str) -> SourceBlock
{
    return SourceBlock {
        ordinal: 1,
        kind: BlockKind::Heading,
        heading_path: Vec::new(),
        text: text.to_owned(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The ordinals the fixtures below give the blocks and rows they build.
    ///
    /// A fixture that builds one block and then a second beside it needs the second to carry
    /// a different ordinal, or the two read as the same position. Each is named for the role
    /// its own fixture gives it, so it is clear which block is the domain-model one and
    /// which is the one that names no family at all.

    /// The row `Colliding_Members` gives its second member, so the two colliding members are
    /// recorded as coming from two different rows rather than from one row twice.
    const SECOND_MEMBERS_ROW: u32 = 2;

    /// The ordinal of the one table block `Test_From_Rows` mints from.
    const THE_TABLE_BLOCK: u32 = 5;

    /// The block `Test_Tabled_Family` puts the glossary heading on, beside the domain-model
    /// block's first.
    const THE_GLOSSARY_BLOCK: u32 = 2;

    /// And the block carrying a heading that names neither tabled family.
    const THE_UNRELATED_BLOCK: u32 = 3;

    /// The block `Test_Tabled_Member` builds its origin from, and the row within it.
    const THE_ORIGIN_BLOCK: u32 = 2;
    const THE_ORIGIN_ROW: u32 = 3;

    /// The ordinal of the whole-name row in `Test_Named_By`, beside the split row's first.
    const THE_WHOLE_NAME_ROW: u32 = 2;

    /// The ordinal of the empty row in `Test_First_Cell`, beside the content row's first.
    const THE_EMPTY_ROW: u32 = 2;

    #[test]
    fn Test_Extract_Members_Should_Collect_Heading_And_Row_Members()
    {
        let markdown = "# Core\n\n## 5. Canonical domain model\n\n\
                        | Model | Responsibility |\n| --- | --- |\n\
                        | WorkspaceContext | Repository. |\n\n\
                        ## 6. Systems and subsystem responsibilities\n\n\
                        ### 6.1 Change reasoning\n\n\
                        #### Counterfactual Analysis Service\n\nEvaluates proposals.\n";

        let members = Extract_Members(DocumentPath("02-core.md"), markdown)
            .expect("the fixture markdown is a whole document, so segmenting it finds members");

        assert_eq!(
            members.iter().filter(|member| return member.family == Restored::CanonicalDomainModel).count(),
            1
        );
        assert_eq!(members.iter().filter(|member| return member.family == Restored::Service).count(), 1);
    }

    #[test]
    fn Test_Refuse_Collisions_Should_Reject_A_Repeated_Identifier()
    {
        let CollidingMembers { first, second } = Colliding_Members();

        let refusal = Refuse_Collisions(&[first, second]).expect_err("must refuse");

        assert!(format!("{refusal}").contains("GLS-APPLICABILITY"), "{refusal}");
    }

    /// Two members that took one identifier from two spellings of the same name.
    struct CollidingMembers
    {
        first: Member,
        second: Member,
    }

    fn Colliding_Members() -> CollidingMembers
    {
        let first = Member {
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
        let second = Member {
            name: "applicability".to_owned(),
            origin: Origin::Row {
                block_ordinal: 1,
                row_ordinal: SECOND_MEMBERS_ROW,
            },
            ..first.clone()
        };

        return CollidingMembers { first, second };
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
    fn Test_From_Rows_Should_Mint_A_Member_Per_Content_Row_In_Its_Volume()
    {
        let block = SourceBlock {
            ordinal: THE_TABLE_BLOCK,
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
            ordinal: THE_GLOSSARY_BLOCK,
            kind: BlockKind::Prose,
            heading_path: vec![GLOSSARY.to_owned()],
            text: String::new(),
        };
        let neither = SourceBlock {
            ordinal: THE_UNRELATED_BLOCK,
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
            block_ordinal: THE_ORIGIN_BLOCK,
            row_ordinal: THE_ORIGIN_ROW,
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
            ordinal: THE_WHOLE_NAME_ROW,
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
            ordinal: THE_EMPTY_ROW,
            table_ordinal: 1,
            kind: RowKind::Content,
            cells: vec![String::new(), "  ".to_owned()],
            text: String::new(),
        };
        assert_eq!(First_Cell(&empty), None);
    }

}
