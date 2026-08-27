//! Reading a revision into sections, and counting what repeats across them.

use super::{SourceBlock, BTreeSet, BTreeMap, Is_Filler, SHARED_BY, Segment, BlockKind, Table_Rows, RowKind, TableRow, Models_In};

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
        let path = path.0;
        let title = title.0;
        let mut strongest: Option<Body> = None;
        for block in Keyable(body)
        {
            strongest = Some(self.Fold_Block(path, title, block, strongest));
        }

        self.authored.entry(title.to_owned()).or_default().push(Position::Heading {
            document: path.to_owned(),
            body: strongest,
        });
    }

    /// Judges one block's shape, tallies it into the section's block/filler counts, and
    /// folds it into the strongest body seen so far for that section.
    fn Fold_Block(&mut self, path: &str, title: &str, block: &SourceBlock, strongest: Option<Body>) -> Body
    {
        let shape = self.Shape(SectionText(&block.text), SectionTitle(title));
        let counted = self.bodies.entry(path.to_owned()).or_default();
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
        let text = text.0;
        let title = title.0;
        let declared = Is_Filler(text);
        let key = Template_Key(SectionText(text), SectionTitle(title));
        let shared = self
            .templates
            .get(&key)
            .map_or(0, |repetition| return repetition.sections);

        if declared.is_some() || shared >= SHARED_BY
        {
            return Body::Template {
                shared_with: shared,
                declared,
            };
        }

        return Body::Narrative;
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

        let heading = Title(&block);
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
        for block in Keyable(body)
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
    if Is_Filler(&block.text).is_some()
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

pub(super) fn Keyable(body: &[SourceBlock]) -> Vec<&SourceBlock>
{
    return body
        .iter()
        .filter(|block| return !block.text.trim().is_empty())
        .filter(|block| return !Is_Navigation(&block.text))
        .collect();
}

pub(super) fn Is_Navigation(text: &str) -> bool
{
    let mut lines = text.lines().filter(|line| return !line.trim().is_empty()).peekable();
    if lines.peek().is_none()
    {
        return false;
    }

    return lines.all(|line| {
        let item = line.trim().trim_start_matches(['-', '*', '+']).trim();
        return line.trim().starts_with(['-', '*', '+'])
            && item.starts_with('[')
            && item.ends_with(')');
    });
}

pub(super) fn Template_Key(text: SectionText<'_>, title: SectionTitle<'_>) -> String
{
    let text = text.0;
    let title = title.0;
    let flattened = text.split_whitespace().collect::<Vec<&str>>().join(" ");
    let elided = title.split_whitespace().collect::<Vec<&str>>().join(" ");

    if elided.is_empty()
    {
        return flattened;
    }

    return flattened.replace(&elided, "{}");
}

pub(super) fn Title(block: &SourceBlock) -> String
{
    return block.text.trim_start_matches('#').trim().to_owned();
}
