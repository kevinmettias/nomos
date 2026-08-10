#![allow(clippy::missing_errors_doc)]

use crate::template::Template;
use crate::filler_census::FillerCensus;
use crate::hollow::Hollow;
use crate::fate::Fate;
use crate::relocation::Relocation;
use crate::document_fate::DocumentFate;
use crate::member_fate::MemberFate;
use crate::regression_report::RegressionReport;
use crate::archive::Archive;
use crate::overlay::Is_Filler;
use crate::phases::IngestError;
use crate::restore::Extract;
use crate::member::Member;
use crate::restore::Models_In;
use crate::restored::Restored;
use crate::revisions::Fingerprint_Of;
use crate::pair_change::PairChange;
use crate::revisions::RevisionFingerprint;
use crate::revisions::Walk;
use crate::revisions::Within;
use crate::revisions::DOMAIN_VOLUMES;
use nomos_spec_model::{BlockKind, RowKind, Segment, SourceBlock, TableRow, Table_Rows};
use std::collections::{BTreeMap, BTreeSet};

pub const SHARED_BY: u32 = 3;

/// Every section of every document, as a path, a heading, and the blocks under it.
///
/// It is a list rather than a map because two documents may carry the same heading, and
/// that they do is the very repetition the census is looking for.
type Sections = Vec<(String, String, Vec<SourceBlock>)>;

pub struct Revision
{
    pub label: String,
    pub documents: BTreeMap<String, String>,
}

impl Revision
{
    pub fn Read(archive: &mut Archive, label: &str) -> Result<Self, IngestError>
    {
        let mut documents = BTreeMap::new();

        for entry in archive.Ending_With(".md")
        {
            let text = archive
                .Read_Text(&entry)
                .map_err(|error| return IngestError::Parse(error.to_string()))?;
            documents.insert(Within(&entry), text);
        }

        if documents.is_empty()
        {
            return Err(IngestError::Parse(format!(
                "{label} holds no markdown, so a regression report over it would find every \
                 family missing from a revision that was never read"
            )));
        }

        return Ok(Self {
            label: label.to_owned(),
            documents,
        });
    }

    pub fn Fingerprint(&self) -> Result<RevisionFingerprint, IngestError>
    {
        return Fingerprint_Of(&self.label, &self.documents);
    }

    #[must_use]
    pub fn Volumes(&self) -> BTreeMap<String, String>
    {
        return self
            .documents
            .iter()
            .filter(|(path, _)| return path.contains(DOMAIN_VOLUMES))
            .map(|(path, text)| {
                let name = path.rsplit_once('/').map_or(path.as_str(), |(_, name)| return name);
                return (name.to_owned(), text.clone());
            })
            .collect();
    }
}

pub fn Regression(from: &Revision, to: &Revision) -> Result<RegressionReport, IngestError>
{
    let volumes = Volumes_Of(from)?;
    let earlier = from.Fingerprint()?;
    let later_print = to.Fingerprint()?;
    let pair = One_Pair(&earlier, &later_print)?;
    let later = Later::Read(&to.documents);
    let members = Judged(&volumes, &later, &to.documents)?;

    return Ok(RegressionReport {
        from: from.label.clone(),
        to: to.label.clone(),
        documents: Relocations(&pair, &earlier, &later_print),
        members,
        filler: Census(&later),
    });
}

/// The domain volumes a revision carries, refusing one that carries none.
///
/// Without the refusal, a revision that never held a family reports every member of every
/// family gone — which reads as catastrophic loss rather than as the wrong input.
fn Volumes_Of(from: &Revision) -> Result<BTreeMap<String, String>, IngestError>
{
    let volumes = from.Volumes();
    if volumes.is_empty()
    {
        return Err(IngestError::Parse(format!(
            "{} has no {DOMAIN_VOLUMES}, so there is no family to ask after. Refusing to \
             report every member gone from a revision that never carried one",
            from.label
        )));
    }

    return Ok(volumes);
}

/// Every member the earlier volumes declare, each with what became of it.
fn Judged(
    volumes: &BTreeMap<String, String>,
    later: &Later,
    documents: &BTreeMap<String, String>,
) -> Result<Vec<MemberFate>, IngestError>
{
    let mut members = Vec::new();
    for (document, markdown) in volumes
    {
        for member in Extract(document, markdown)?
        {
            let judged = Judge(&member, later, documents);
            members.push(judged);
        }
    }

    return Ok(members);
}

fn One_Pair(
    from: &RevisionFingerprint,
    to: &RevisionFingerprint,
) -> Result<PairChange, IngestError>
{
    let walk = Walk(&[from.clone(), to.clone()]);

    return walk.into_iter().next().ok_or_else(|| {
        return IngestError::Parse(format!(
            "{} and {} are not a pair the walk recognises",
            from.label, to.label
        ));
    });
}

#[must_use]
pub fn Relocations(
    pair: &PairChange,
    from: &RevisionFingerprint,
    to: &RevisionFingerprint,
) -> DocumentFate
{
    let origins = Origins_By_Content(pair, from);
    let mut fate = DocumentFate {
        changed: pair.changed.clone(),
        ..DocumentFate::default()
    };
    let moved = Place_Appeared(pair, to, &origins, &mut fate);

    fate.disappeared = pair
        .disappeared
        .iter()
        .filter(|path| return !moved.contains(path.as_str()))
        .cloned()
        .collect();

    return fate;
}

/// The paths that disappeared, indexed by the content they held.
///
/// A move is only recognisable as one because the content survived under another name, so
/// content is the key and the old paths are what it answers with.
fn Origins_By_Content<'a>(
    pair: &'a PairChange,
    from: &'a RevisionFingerprint,
) -> BTreeMap<&'a str, Vec<String>>
{
    let mut origins: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for path in &pair.disappeared
    {
        if let Some(hash) = from.documents.get(path)
        {
            origins.entry(hash.as_str()).or_default().push(path.clone());
        }
    }

    return origins;
}

/// Sorts each appeared path into a relocation or a genuine arrival, and says which origins
/// were accounted for — those are the ones the caller must not also report as disappeared.
fn Place_Appeared(
    pair: &PairChange,
    to: &RevisionFingerprint,
    origins: &BTreeMap<&str, Vec<String>>,
    fate: &mut DocumentFate,
) -> BTreeSet<String>
{
    let mut moved: BTreeSet<String> = BTreeSet::new();

    for path in &pair.appeared
    {
        let origin = to.documents.get(path).and_then(|hash| return origins.get(hash.as_str()));
        match origin
        {
            Some(from_paths) =>
            {
                moved.extend(from_paths.iter().cloned());
                fate.relocated.push(Relocation {
                    to: path.clone(),
                    from: from_paths.clone(),
                });
            }
            None => fate.appeared.push(path.clone()),
        }
    }

    return moved;
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Body
{
    Narrative,
    Template
    {
        shared_with: u32,
        declared: Option<&'static str>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Position
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
struct Bodies
{
    blocks: u32,
    filler: u32,
}

#[derive(Clone, Debug, Default)]
struct Repetition
{
    sections: u32,
    documents: BTreeSet<String>,
}

struct Later
{
    authored: BTreeMap<String, Vec<Position>>,
    named_in_row: BTreeMap<String, String>,
    templates: BTreeMap<String, Repetition>,
    declared: BTreeSet<String>,
    bodies: BTreeMap<String, Bodies>,
}

impl Later
{
    /// Reads every document into the index, in three passes that cannot be merged.
    ///
    /// A block's shape depends on how many *other* sections repeat it, so nothing can be
    /// judged until every section has been counted. The passes are: cut the documents into
    /// sections, count what repeats, then judge each section against the counts.
    fn Read(documents: &BTreeMap<String, String>) -> Self
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
            later.Note_Section(path, title, body);
        }

        return later;
    }

    /// What one section contributes: how much of it is filler, and where its heading stands.
    fn Note_Section(&mut self, path: &str, title: &str, body: &[SourceBlock])
    {
        let mut strongest: Option<Body> = None;
        for block in Keyable(body)
        {
            let shape = self.Shape(&block.text, title);
            let counted = self.bodies.entry(path.to_owned()).or_default();
            counted.blocks = counted.blocks.saturating_add(1);
            if matches!(shape, Body::Template { .. })
            {
                counted.filler = counted.filler.saturating_add(1);
            }
            strongest = Some(match (strongest.take(), shape)
            {
                (Some(Body::Narrative), _) | (_, Body::Narrative) => Body::Narrative,
                (_, other) => other,
            });
        }

        self.authored.entry(title.to_owned()).or_default().push(Position::Heading {
            document: path.to_owned(),
            body: strongest,
        });
    }

    fn Shape(&self, text: &str, title: &str) -> Body
    {
        let declared = Is_Filler(text);
        let key = Template_Key(text, title);
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
fn Sections_Of(documents: &BTreeMap<String, String>, later: &mut Later) -> Sections
{
    let mut sections: Sections = Vec::new();

    for (path, markdown) in documents
    {
        Cut_At_Headings(path, markdown, later, &mut sections);
    }

    return sections;
}

/// Cuts one document at its headings, noting what its non-heading blocks say on the way.
///
/// A block before the first heading belongs to no section and is indexed but not kept: it
/// is front matter, and keying it against an empty title would make every document's front
/// matter look like a repetition of every other's.
fn Cut_At_Headings(path: &str, markdown: &str, later: &mut Later, sections: &mut Sections)
{
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
fn Close_Section(
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
fn Repetitions_In(sections: &[(String, String, Vec<SourceBlock>)]) -> BTreeMap<String, Repetition>
{
    let mut templates: BTreeMap<String, Repetition> = BTreeMap::new();

    for (path, title, body) in sections
    {
        for block in Keyable(body)
        {
            let key = Template_Key(&block.text, title);
            let repetition = templates.entry(key).or_default();
            repetition.sections = repetition.sections.saturating_add(1);
            repetition.documents.insert(path.clone());
        }
    }

    return templates;
}

/// What one non-heading block contributes to the index: whether its document declares
/// filler, and every subject its content rows name.
fn Note_Block(later: &mut Later, path: &str, block: &SourceBlock)
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
fn Note_Row(later: &mut Later, path: &str, row: &TableRow)
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

fn Keyable(body: &[SourceBlock]) -> Vec<&SourceBlock>
{
    return body
        .iter()
        .filter(|block| return !block.text.trim().is_empty())
        .filter(|block| return !Is_Navigation(&block.text))
        .collect();
}

fn Is_Navigation(text: &str) -> bool
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

fn Template_Key(text: &str, title: &str) -> String
{
    let flattened = text.split_whitespace().collect::<Vec<&str>>().join(" ");
    let elided = title.split_whitespace().collect::<Vec<&str>>().join(" ");

    if elided.is_empty()
    {
        return flattened;
    }

    return flattened.replace(&elided, "{}");
}

fn Title(block: &SourceBlock) -> String
{
    return block.text.trim_start_matches('#').trim().to_owned();
}

fn Judge(member: &Member, later: &Later, documents: &BTreeMap<String, String>) -> MemberFate
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
fn Still(member: &Member, later: &Later) -> Option<Fate>
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
fn Named_In_Row(member: &Member, later: &Later) -> Option<Fate>
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
fn At(position: &Position) -> Fate
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

fn Mentions(name: &str, documents: &BTreeMap<String, String>) -> Fate
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

fn Census(later: &Later) -> FillerCensus
{
    return FillerCensus {
        declared: later.declared.iter().cloned().collect(),
        templates: Shared_Templates(later),
        stubs: Stubs(later),
    };
}

/// The blocks repeated widely enough to be template rather than prose, widest first.
///
/// Ordering is by reach and then by text, so the report opens on the thing worth deleting
/// and two runs over the same corpus agree on what to print.
fn Shared_Templates(later: &Later) -> Vec<Template>
{
    let mut templates: Vec<Template> = later
        .templates
        .iter()
        .filter(|(_, repetition)| return repetition.sections >= SHARED_BY)
        .map(|(key, repetition)| return Described(key, repetition))
        .collect();
    templates.sort_by(|first, second| {
        return second
            .sections
            .cmp(&first.sections)
            .then_with(|| return first.text.cmp(&second.text));
    });

    return templates;
}

/// One repeated block as the census reports it.
fn Described(key: &str, repetition: &Repetition) -> Template
{
    return Template {
        text: key.to_owned(),
        sections: repetition.sections,
        documents: repetition.documents.iter().cloned().collect(),
        declared: Is_Filler(key),
    };
}

/// The documents carrying nothing but filler.
///
/// A document with no blocks at all is not one of them: it is empty, which is a different
/// complaint and one the reader can already see.
fn Stubs(later: &Later) -> Vec<String>
{
    let mut stubs: Vec<String> = Vec::new();

    for (path, bodies) in &later.bodies
    {
        if bodies.blocks > 0 && bodies.filler == bodies.blocks
        {
            stubs.push(path.clone());
        }
    }

    return stubs;
}

#[cfg(test)]
mod tests
{
    use super::*;

    const CORE: &str = "# Core\n\n## 5. Canonical domain model\n\n\
                        | Model | Responsibility |\n| --- | --- |\n\
                        | WorkspaceContext | Repository, branch, configuration. |\n\
                        | ModelUsageObservation and CostObservation | Tokens against money. |\n\n\
                        ## 6. Systems and subsystem responsibilities\n\n\
                        ### 6.1 Change reasoning\n\n\
                        #### Counterfactual Analysis Service\n\nEvaluates proposals.\n";

    const DECLARED: &str = "This section groups related specification material for the domain.";

    fn Documents(pairs: &[(&str, &str)]) -> BTreeMap<String, String>
    {
        return pairs
            .iter()
            .map(|(path, text)| return ((*path).to_owned(), (*text).to_owned()))
            .collect();
    }

    fn Earlier() -> Revision
    {
        return Revision {
            label: "v14.36".to_owned(),
            documents: Documents(&[("01_authoring/domain_volumes/02-core-architecture.md", CORE)]),
        };
    }

    fn Earlier_With(path: &str, text: &str) -> Revision
    {
        let mut revision = Earlier();
        revision.documents.insert(path.to_owned(), text.to_owned());

        return revision;
    }

    fn Later_Than(documents: &[(&str, &str)]) -> Revision
    {
        return Revision {
            label: "v15.0".to_owned(),
            documents: Documents(documents),
        };
    }

    fn Fate_Of(report: &RegressionReport, name: &str) -> Fate
    {
        return report
            .Named(name)
            .unwrap_or_else(|| panic!("{name} is not a member"))
            .fate
            .clone();
    }

    fn Reported(later: &[(&str, &str)]) -> RegressionReport
    {
        return Regression(&Earlier(), &Later_Than(later)).expect("reports");
    }

    #[test]
    fn Test_A_Heading_Over_A_Repeated_Paragraph_Should_Be_Hollowed()
    {
        let report = Reported(&[
            ("a.md", "# Counterfactual Analysis Service\n\nRefer to the owning domain.\n"),
            ("b.md", "# Something else\n\nRefer to the owning domain.\n"),
            ("c.md", "# A third\n\nRefer to the owning domain.\n"),
        ]);

        assert_eq!(
            Fate_Of(&report, "Counterfactual Analysis Service"),
            Fate::Hollowed {
                document: "a.md".to_owned(),
                evidence: Hollow::Template {
                    shared_with: 3,
                    declared: None,
                },
            }
        );
    }

    #[test]
    fn Test_A_Paragraph_Two_Sections_Share_Should_Not_Be_A_Template()
    {
        let report = Reported(&[
            ("a.md", "# Counterfactual Analysis Service\n\nRefer to the owning domain.\n"),
            ("b.md", "# Something else\n\nRefer to the owning domain.\n"),
        ]);

        assert_eq!(
            Fate_Of(&report, "Counterfactual Analysis Service"),
            Fate::Preserved {
                document: "a.md".to_owned(),
            }
        );
    }

    #[test]
    fn Test_A_Heading_Over_Nothing_Should_Be_Hollowed_With_No_Body()
    {
        let report = Reported(&[("a.md", "# Counterfactual Analysis Service\n")]);

        assert_eq!(
            Fate_Of(&report, "Counterfactual Analysis Service"),
            Fate::Hollowed {
                document: "a.md".to_owned(),
                evidence: Hollow::NoBody,
            }
        );
    }

    #[test]
    fn Test_A_Heading_Over_Its_Own_Link_List_Should_Not_Count_As_A_Body()
    {
        let report = Reported(&[(
            "a.md",
            "# Counterfactual Analysis Service\n\n- [One](one.md)\n- [Two](two.md)\n",
        )]);

        assert_eq!(
            Fate_Of(&report, "Counterfactual Analysis Service"),
            Fate::Hollowed {
                document: "a.md".to_owned(),
                evidence: Hollow::NoBody,
            }
        );
    }

    #[test]
    fn Test_A_Name_In_Prose_Should_Be_Mentioned_Rather_Than_Preserved()
    {
        let report = Reported(&[("a.md", "# Elsewhere\n\nThe WorkspaceContext is discussed.\n")]);

        assert_eq!(
            Fate_Of(&report, "WorkspaceContext"),
            Fate::Mentioned {
                documents: vec!["a.md".to_owned()],
            }
        );
    }

    #[test]
    fn Test_A_Name_Occurring_Nowhere_Should_Be_Gone()
    {
        let report = Reported(&[("a.md", "# Elsewhere\n\nNothing of the kind.\n")]);

        assert_eq!(Fate_Of(&report, "WorkspaceContext"), Fate::Gone);
        assert_eq!(report.Tally(Restored::CanonicalDomainModel).gone, 3);
    }

    #[test]
    fn Test_A_Model_Sharing_A_Row_Should_Be_Found_In_That_Row()
    {
        let report = Reported(&[(
            "a.md",
            "# Models\n\n| Model | Responsibility |\n| --- | --- |\n\
             | ModelUsageObservation and CostObservation | Tokens against money. |\n",
        )]);

        assert_eq!(
            Fate_Of(&report, "CostObservation"),
            Fate::Preserved {
                document: "a.md".to_owned(),
            },
            "a model reads as absent from a table it is in, because the cell names two"
        );
    }

    #[test]
    fn Test_A_Moved_Document_Should_Be_Relocated_Rather_Than_Both_Sets()
    {
        let earlier = Earlier_With("old/record.md", "# Record\n\nA decision.\n");
        let later = Later_Than(&[("new/record.md", "# Record\n\nA decision.\n")]);

        let report = Regression(&earlier, &later).expect("reports");

        assert_eq!(report.documents.relocated.len(), 1);
        assert_eq!(
            report.documents.relocated.first().map(|moved| return moved.to.clone()),
            Some("new/record.md".to_owned())
        );
        assert_eq!(
            report.documents.relocated.first().map(|moved| return moved.from.clone()),
            Some(vec!["old/record.md".to_owned()])
        );
        assert!(report.documents.appeared.is_empty());
        assert!(!report.documents.disappeared.iter().any(|path| return path == "old/record.md"));
    }

    #[test]
    fn Test_A_Relocation_With_Two_Origins_Should_Name_Both()
    {
        let mut earlier = Earlier_With("old/one.md", "# Record\n\nA decision.\n");
        earlier
            .documents
            .insert("old/two.md".to_owned(), "# Record\n\nA decision.\n".to_owned());
        let later = Later_Than(&[("new/record.md", "# Record\n\nA decision.\n")]);

        let report = Regression(&earlier, &later).expect("reports");

        assert_eq!(
            report.documents.relocated.first().map(|moved| return moved.from.clone()),
            Some(vec!["old/one.md".to_owned(), "old/two.md".to_owned()])
        );
    }

    #[test]
    fn Test_An_Edited_Move_Should_Not_Be_A_Relocation()
    {
        let earlier = Earlier_With("old/record.md", "# Record\n\nA decision.\n");
        let later = Later_Than(&[("new/record.md", "# Record\n\nA different decision.\n")]);

        let report = Regression(&earlier, &later).expect("reports");

        assert!(report.documents.relocated.is_empty());
        assert_eq!(report.documents.appeared, vec!["new/record.md".to_owned()]);
        assert!(report.documents.disappeared.contains(&"old/record.md".to_owned()));
    }

    #[test]
    fn Test_A_Revision_Without_The_Volumes_Should_Be_Refused()
    {
        let earlier = Revision {
            label: "v15.0".to_owned(),
            documents: Documents(&[("records/one.md", "# Record\n\nA decision.\n")]),
        };

        let refusal = Regression(&earlier, &Later_Than(&[("a.md", "# A\n\nText.\n")]))
            .expect_err("must refuse");

        assert!(format!("{refusal}").contains("no family to ask after"), "{refusal}");
    }

    #[test]
    fn Test_A_Revision_With_No_Markdown_Should_Be_Refused()
    {
        let empty: BTreeMap<String, String> = BTreeMap::new();

        assert!(Fingerprint_Of("v15.0", &empty).is_err());
    }

    #[test]
    fn Test_Declared_Filler_Should_Be_Named_As_Declared()
    {
        let first = format!("# Counterfactual Analysis Service\n\n{DECLARED}\n");
        let second = format!("# Something else\n\n{DECLARED}\n");
        let third = format!("# A third\n\n{DECLARED}\n");
        let report = Reported(&[("a.md", &first), ("b.md", &second), ("c.md", &third)]);

        assert!(
            matches!(
                Fate_Of(&report, "Counterfactual Analysis Service"),
                Fate::Hollowed {
                    evidence: Hollow::Template {
                        declared: Some(_), ..
                    },
                    ..
                }
            ),
            "{:?}",
            Fate_Of(&report, "Counterfactual Analysis Service")
        );
        assert_eq!(report.filler.declared.len(), 3);
        assert!(report.filler.Widest_Undeclared().is_none());
        assert_eq!(report.filler.stubs.len(), 3);
    }

    #[test]
    fn Test_Declared_Filler_Should_Not_Need_The_Threshold()
    {
        let only = format!("# Counterfactual Analysis Service\n\n{DECLARED}\n");
        let report = Reported(&[("a.md", &only)]);

        assert!(matches!(
            Fate_Of(&report, "Counterfactual Analysis Service"),
            Fate::Hollowed { .. }
        ));
    }

    #[test]
    fn Test_A_Template_Naming_Its_Own_Section_Should_Read_As_One_Template()
    {
        let report = Reported(&[
            (
                "a.md",
                "# Counterfactual Analysis Service\n\nRead Counterfactual Analysis Service \
                 within the owning contract.\n",
            ),
            ("b.md", "# Second\n\nRead Second within the owning contract.\n"),
            ("c.md", "# Third\n\nRead Third within the owning contract.\n"),
        ]);

        assert!(
            matches!(Fate_Of(&report, "Counterfactual Analysis Service"), Fate::Hollowed { .. }),
            "three sections of one form letter read as three distinct paragraphs"
        );
        assert_eq!(
            report.filler.Widest_Undeclared().map(|template| return template.sections),
            Some(3)
        );
    }

    #[test]
    fn Test_The_Summary_Should_Name_Members_Rather_Than_Only_Count_Them()
    {
        let report = Reported(&[("a.md", "# A\n\nText.\n")]);

        assert!(report.Summary().contains("WorkspaceContext"), "{}", report.Summary());
    }
}
