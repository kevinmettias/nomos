//! Reading a revision into sections, and counting what repeats across them.

use super::{SourceBlock, BTreeSet, BTreeMap, Get_Filler_Pattern, Segment, BlockKind, Table_Rows, RowKind, TableRow, Models_In};

mod blocks;

use blocks::{Body_Of, Keyable_Blocks, Template_Key, Title_Of};

/// Where a section sits: which document, under what heading.
#[derive(Clone, Copy)]
pub(super) struct SectionPath<'a>(&'a str);

/// The heading a section is filed under, distinct from [`SectionPath`] and [`SectionText`]
/// even though all three are `&str` and sit adjacent at more than one call site here.
#[derive(Clone, Copy)]
pub(super) struct SectionTitle<'a>(&'a str);

/// A block's or a document's own text, as opposed to the [`SectionPath`] or [`SectionTitle`]
/// it is judged against.
#[derive(Clone, Copy)]
pub(super) struct SectionText<'a>(&'a str);

/// Every section of every document, as a path, a heading, and the blocks under it.
///
/// It is a list rather than a map because two documents may carry the same heading, and
/// that they do is the very repetition the census is looking for.
pub(super) type Sections = Vec<(String, String, Vec<SourceBlock>)>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Body
{
    Narrative,
    Template
    {
        shared_with: u32,
        declared: Option<&'static str>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Position
{
    Heading
    {
        document: String,
        body: Option<Body>,
    },
    Row
    {
        document: String,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Bodies
{
    pub(super) blocks: u32,
    pub(super) filler: u32,
}

#[derive(Clone, Debug, Default)]
pub(super) struct Repetition
{
    pub(super) sections: u32,
    pub(super) documents: BTreeSet<String>,
}

pub(super) struct Later
{
    pub(super) authored: BTreeMap<String, Vec<Position>>,
    pub(super) named_in_row: BTreeMap<String, String>,
    pub(super) templates: BTreeMap<String, Repetition>,
    pub(super) declared: BTreeSet<String>,
    pub(super) bodies: BTreeMap<String, Bodies>,
}

impl Later
{
    /// Reads every document into the index, in three passes that cannot be merged.
    ///
    /// A block's shape depends on how many *other* sections repeat it, so nothing can be
    /// judged until every section has been counted. The passes are: cut the documents into
    /// sections, count what repeats, then judge each section against the counts.
    pub(super) fn Read(documents: &BTreeMap<String, String>) -> Self
    {
        let mut later = Self {
            authored: BTreeMap::new(),
            named_in_row: BTreeMap::new(),
            templates: BTreeMap::new(),
            declared: BTreeSet::new(),
            bodies: BTreeMap::new(),
        };

        let sections = Sections_Of(documents, &mut later);
        later.templates = Repetitions_In(&sections);
        for (path, title, body) in &sections
        {
            later.Note_Section(SectionPath(path), SectionTitle(title), body);
        }

        return later;
    }

    /// What one section contributes: how much of it is filler, and where its heading stands.
    fn Note_Section(&mut self, path: SectionPath<'_>, title: SectionTitle<'_>, body: &[SourceBlock])
    {
        let mut strongest: Option<Body> = None;
        for block in Keyable_Blocks(body)
        {
            strongest = Some(self.Fold_Block(path, title, block, strongest));
        }

        self.authored.entry(title.0.to_owned()).or_default().push(Position::Heading {
            document: path.0.to_owned(),
            body: strongest,
        });
    }

    /// Judges one block's shape, tallies it into the section's block/filler counts, and
    /// folds it into the strongest body seen so far for that section.
    fn Fold_Block(&mut self, path: SectionPath<'_>, title: SectionTitle<'_>, block: &SourceBlock, strongest: Option<Body>) -> Body
    {
        let shape = self.Shape(SectionText(&block.text), title);
        let counted = self.bodies.entry(path.0.to_owned()).or_default();
        counted.blocks = counted.blocks.saturating_add(1);
        if matches!(shape, Body::Template { .. })
        {
            counted.filler = counted.filler.saturating_add(1);
        }

        return match (strongest, shape)
        {
            (Some(Body::Narrative), _) | (_, Body::Narrative) => Body::Narrative,
            (_, other) => other,
        };
    }

    fn Shape(&self, text: SectionText<'_>, title: SectionTitle<'_>) -> Body
    {
        let declared = Get_Filler_Pattern(text.0);
        let key = Template_Key(text, title);
        let shared = self.Sections_Sharing(&key);

        return Body_Of(declared, shared, &key);
    }

    /// How many sections already carry this key.
    fn Sections_Sharing(&self, key: &str) -> u32
    {
        return self.templates.get(key).map_or(0, |repetition| return repetition.sections);
    }
}

/// Cuts every document at its headings, noting what the non-heading blocks say on the way.
///
/// The index is filled here rather than afterwards because a block is read once and both
/// passes want it — the sections for repetition counting, the index for what the rows name.
pub(super) fn Sections_Of(documents: &BTreeMap<String, String>, later: &mut Later) -> Sections
{
    let mut sections: Sections = Vec::new();

    for (path, markdown) in documents
    {
        Cut_At_Headings(SectionPath(path), SectionText(markdown), later, &mut sections);
    }

    return sections;
}

/// Cuts one document at its headings, noting what its non-heading blocks say on the way.
///
/// A block before the first heading belongs to no section and is indexed but not kept: it
/// is front matter, and keying it against an empty title would make every document's front
/// matter look like a repetition of every other's.
pub(super) fn Cut_At_Headings(path: SectionPath<'_>, markdown: SectionText<'_>, later: &mut Later, sections: &mut Sections)
{
    let path = path.0;
    let markdown = markdown.0;
    let mut title: Option<String> = None;
    let mut body: Vec<SourceBlock> = Vec::new();

    for block in Segment(markdown)
    {
        if block.kind != BlockKind::Heading
        {
            Note_Block(later, path, &block);
            body.push(block);
            continue;
        }

        let heading = Title_Of(&block);
        Close_Section(path, title.replace(heading), &mut body, sections);
    }

    Close_Section(path, title, &mut body, sections);
}

/// Emits the section a heading just ended, if a heading had opened one.
///
/// The same call closes the last section at the end of the document, because "another
/// heading arrived" and "the document ran out" end a section for the same reason. Written
/// out twice, the two were one edit away from disagreeing about what gets emitted.
pub(super) fn Close_Section(
    path: &str,
    title: Option<String>,
    body: &mut Vec<SourceBlock>,
    sections: &mut Sections,
)
{
    let Some(title) = title
    else
    {
        return;
    };

    sections.push((path.to_owned(), title, core::mem::take(body)));
}

/// How many sections, and how many documents, each keyed block appears in.
pub(super) fn Repetitions_In(sections: &[(String, String, Vec<SourceBlock>)]) -> BTreeMap<String, Repetition>
{
    let mut templates: BTreeMap<String, Repetition> = BTreeMap::new();

    for (path, title, body) in sections
    {
        for block in Keyable_Blocks(body)
        {
            let key = Template_Key(SectionText(&block.text), SectionTitle(title));
            let repetition = templates.entry(key).or_default();
            repetition.sections = repetition.sections.saturating_add(1);
            repetition.documents.insert(path.clone());
        }
    }

    return templates;
}

/// What one non-heading block contributes to the index: whether its document declares
/// filler, and every subject its content rows name.
pub(super) fn Note_Block(later: &mut Later, path: &str, block: &SourceBlock)
{
    if Get_Filler_Pattern(&block.text).is_some()
    {
        later.declared.insert(path.to_owned());
    }

    for row in Table_Rows(block).iter().filter(|row| return row.kind == RowKind::Content)
    {
        Note_Row(later, path, row);
    }
}

/// What one content row names.
///
/// The subject is the row's first non-empty cell, and a row with none is not a statement
/// about anything — a table's alignment padding is not a member of the domain.
pub(super) fn Note_Row(later: &mut Later, path: &str, row: &TableRow)
{
    let subject = row
        .cells
        .iter()
        .map(|cell| return cell.trim())
        .find(|cell| return !cell.is_empty());
    let Some(cell) = subject
    else
    {
        return;
    };

    later.authored.entry(cell.to_owned()).or_default().push(Position::Row {
        document: path.to_owned(),
    });
    for name in Models_In(cell)
    {
        later
            .named_in_row
            .entry(name.to_owned())
            .or_insert_with(|| return path.to_owned());
    }
}

/// A prose block carrying the given text, for the tests in this file and in [`blocks`].
///
/// It sits at module scope rather than inside either test module because both need it, and
/// a private item here is visible to both.
#[cfg(test)]
fn Test_Block(text: &str) -> SourceBlock
{
    return SourceBlock {
        ordinal: 1,
        kind: BlockKind::Prose,
        heading_path: Vec::new(),
        text: text.to_owned(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// How many documents each repetition fixture below writes, and so how many positions
    /// the heading they share must be indexed under.
    const SHARED_DOCUMENTS: usize = 3;

    /// The same three documents where `Body::Template` carries the count: `shared_with` is a
    /// `u32`, so a match pattern over it cannot compare against the `usize` above, and the
    /// one count is therefore named once in each width rather than written twice as a bare 3.
    const SHARED_WITH_TEMPLATE: u32 = SHARED_DOCUMENTS as u32;

    /// The sections the one document below cuts into: it carries two `#` headings, and front
    /// matter would be a third only if `Cut_At_Headings` opened a section for it.
    const SECTIONS_IN_THE_TWO_HEADING_DOCUMENT: usize = 2;

    /// The sections `Repetitions_In` folds under a single key when two documents carry the
    /// same title and the same body.
    const SECTIONS_UNDER_ONE_KEY: u32 = 2;

    #[test]
    fn Test_Read_Should_Index_Headings_As_Positions_And_Judge_Repeated_Bodies()
    {
        let documents = Documents_By_Path(&[
            ("a.md", "# Shared\n\nRefer to the owning domain volume for this material.\n"),
            ("b.md", "# Shared\n\nRefer to the owning domain volume for this material.\n"),
            ("c.md", "# Shared\n\nRefer to the owning domain volume for this material.\n"),
        ]);

        let later = Later::Read(&documents);

        let positions = later.authored.get("Shared").expect("the heading is indexed");
        assert_eq!(positions.len(), SHARED_DOCUMENTS);
        assert!(positions.iter().all(|position| return matches!(
            position,
            Position::Heading { body: Some(Body::Template { shared_with: SHARED_WITH_TEMPLATE, declared: None }), .. }
        )));
    }

    /// A repeated body under the floor leaves its section narrative rather than hollowed.
    ///
    /// The falsifier for the floor being applied here as well as in the census and the rule.
    /// A section whose body is a connective repeated across the corpus has not been hollowed
    /// by a form letter, and reporting it as `Template` is what sent `OD-SPEC-004` version 3
    /// looking at 102 of them.
    #[test]
    fn Test_A_Repeated_Body_Below_The_Floor_Should_Read_As_Narrative()
    {
        let documents = Documents_By_Path(&[
            ("a.md", "# Shared\n\nShort shared line.\n"),
            ("b.md", "# Shared\n\nShort shared line.\n"),
            ("c.md", "# Shared\n\nShort shared line.\n"),
        ]);

        let later = Later::Read(&documents);

        let positions = later.authored.get("Shared").expect("the heading is indexed");
        assert_eq!(positions.len(), SHARED_DOCUMENTS, "all three sections are indexed under one heading");
        assert!(
            positions.iter().all(|position| return matches!(
                position,
                Position::Heading { body: Some(Body::Narrative), .. }
            )),
            "a body under the floor repeated three times is narrative, not a template: {positions:?}"
        );
    }

    #[test]
    fn Test_Sections_Of_Should_Cut_Every_Document_Into_Its_Own_Sections()
    {
        let documents = Documents_By_Path(&[("a.md", "# One\n\nFirst body.\n\n# Two\n\nSecond body.\n")]);
        let mut later = Empty_Later();

        let sections = Sections_Of(&documents, &mut later);

        assert_eq!(sections.len(), SECTIONS_IN_THE_TWO_HEADING_DOCUMENT);
        assert_eq!(sections.first().expect("the document cuts into two sections").0, "a.md");
        assert_eq!(sections.first().expect("the document cuts into two sections").1, "One");
        assert_eq!(sections.get(1).expect("the document cuts into two sections").1, "Two");
    }

    #[test]
    fn Test_Cut_At_Headings_Should_Not_Emit_A_Section_For_Front_Matter_Before_The_First_Heading()
    {
        let mut later = Empty_Later();
        let mut sections: Sections = Vec::new();

        Cut_At_Headings(SectionPath("a.md"), SectionText("Front matter.\n\n# One\n\nBody.\n"), &mut later, &mut sections);

        assert_eq!(sections.len(), 1, "front matter must not open a section of its own");
        assert_eq!(sections.first().expect("the assertion above confirms exactly one section").1, "One");
    }

    #[test]
    fn Test_Close_Section_Should_Emit_Nothing_When_No_Heading_Has_Opened()
    {
        let mut sections: Sections = Vec::new();
        let mut body = vec![Test_Block("Text.")];

        Close_Section("a.md", None, &mut body, &mut sections);

        assert!(sections.is_empty());
        assert_eq!(body.len(), 1, "the body is left as-is for a caller who has not opened a section yet");
    }

    #[test]
    fn Test_Close_Section_Should_Emit_The_Section_A_Heading_Opened()
    {
        let mut sections: Sections = Vec::new();
        let mut body = vec![Test_Block("Text.")];

        Close_Section("a.md", Some("Title".to_owned()), &mut body, &mut sections);

        assert_eq!(sections.len(), 1);

        let (path, title, blocks) =
            sections.first().expect("the assertion above confirms exactly one section");

        assert_eq!(path, "a.md");
        assert_eq!(title, "Title");
        assert_eq!(blocks, &vec![Test_Block("Text.")]);
        assert!(body.is_empty());
    }

    #[test]
    fn Test_Repetitions_In_Should_Count_Sections_And_Distinct_Documents_Per_Key()
    {
        let sections: Sections = vec![
            ("a.md".to_owned(), "Title".to_owned(), vec![Test_Block("Shared body.")]),
            ("b.md".to_owned(), "Title".to_owned(), vec![Test_Block("Shared body.")]),
        ];

        let templates = Repetitions_In(&sections);

        let key = Template_Key(SectionText("Shared body."), SectionTitle("Title"));
        let repetition = templates.get(&key).expect("the shared block is keyed");
        assert_eq!(repetition.sections, SECTIONS_UNDER_ONE_KEY);
        assert_eq!(repetition.documents, BTreeSet::from(["a.md".to_owned(), "b.md".to_owned()]));
    }

    #[test]
    fn Test_Note_Block_Should_Flag_Declared_Filler_And_Index_Table_Subjects()
    {
        let mut later = Empty_Later();

        Note_Block(&mut later, "a.md", &Test_Block("This section groups related specification material for X.\n"));
        Note_Block(
            &mut later,
            "b.md",
            &Test_Block("| Subject | Detail |\n| --- | --- |\n| Widget | Detail text. |\n"),
        );

        assert!(later.declared.contains("a.md"));
        assert!(!later.declared.contains("b.md"));
        assert!(later.authored.contains_key("Widget"));
    }

    #[test]
    fn Test_Note_Row_Should_Record_The_Rows_Subject_As_An_Authored_Position()
    {
        let mut later = Empty_Later();
        let row = TableRow {
            ordinal: 1,
            table_ordinal: 1,
            kind: RowKind::Content,
            cells: vec!["  ".to_owned(), "Widget".to_owned()],
            text: "|  | Widget |".to_owned(),
        };

        Note_Row(&mut later, "a.md", &row);

        let positions = later.authored.get("Widget").expect("the subject is indexed");
        assert_eq!(positions.len(), 1);
        assert!(matches!(
            positions.first().expect("the assertion above confirms exactly one position"),
            Position::Row { document } if document == "a.md"
        ));
        assert_eq!(later.named_in_row.get("Widget"), Some(&"a.md".to_owned()));
    }

    fn Documents_By_Path(pairs: &[(&str, &str)]) -> BTreeMap<String, String>
    {
        return pairs.iter().map(|(path, text)| return ((*path).to_owned(), (*text).to_owned())).collect();
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
