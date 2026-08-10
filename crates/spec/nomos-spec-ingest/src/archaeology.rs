#![allow(clippy::missing_errors_doc)]

use crate::archive::Archive;
use crate::overlay::Is_Filler;
use crate::phases::IngestError;
use crate::restore::{Extract, Member, Models_In, Restored};
use crate::revisions::{Fingerprint_Of, PairChange, RevisionFingerprint, Walk, Within, DOMAIN_VOLUMES};
use nomos_spec_model::{BlockKind, RowKind, Segment, SourceBlock, Table_Rows};
use core::fmt::Write as _;
use std::collections::{BTreeMap, BTreeSet};

pub const SHARED_BY: u32 = 3;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fate
{
    Preserved
    {
        document: String,
    },
    Hollowed
    {
        document: String,
        evidence: Hollow,
    },
    Mentioned
    {
        documents: Vec<String>,
    },
    Gone,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Hollow
{
    Template
    {
        shared_with: u32,
        declared: Option<&'static str>,
    },
    NoBody,
}

impl Fate
{
    #[must_use]
    pub const fn Label(&self) -> &'static str
    {
        return match self
        {
            Self::Preserved { .. } => "preserved",
            Self::Hollowed { .. } => "hollowed",
            Self::Mentioned { .. } => "mentioned",
            Self::Gone => "gone",
        };
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberFate
{
    pub id: String,
    pub family: Restored,
    pub name: String,
    pub was: String,
    pub fate: Fate,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally
{
    pub preserved: u32,
    pub hollowed: u32,
    pub mentioned: u32,
    pub gone: u32,
}

impl Tally
{
    #[must_use]
    pub const fn Total(&self) -> u32
    {
        return self
            .preserved
            .saturating_add(self.hollowed)
            .saturating_add(self.mentioned)
            .saturating_add(self.gone);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Relocation
{
    pub to: String,
    pub from: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DocumentFate
{
    pub appeared: Vec<String>,
    pub disappeared: Vec<String>,
    pub changed: Vec<String>,
    pub relocated: Vec<Relocation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Template
{
    pub text: String,
    pub sections: u32,
    pub documents: Vec<String>,
    pub declared: Option<&'static str>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FillerCensus
{
    pub templates: Vec<Template>,
    pub declared: Vec<String>,
    pub stubs: Vec<String>,
}

impl FillerCensus
{
    #[must_use]
    pub fn Undeclared(&self) -> Vec<&Template>
    {
        return self
            .templates
            .iter()
            .filter(|template| return template.declared.is_none())
            .collect();
    }

    #[must_use]
    pub fn Widest_Undeclared(&self) -> Option<&Template>
    {
        return self.Undeclared().into_iter().next();
    }
}

#[derive(Clone, Debug, Default)]
pub struct RegressionReport
{
    pub from: String,
    pub to: String,
    pub documents: DocumentFate,
    pub members: Vec<MemberFate>,
    pub filler: FillerCensus,
}

impl RegressionReport
{
    #[must_use]
    pub fn In(&self, family: Restored) -> Vec<&MemberFate>
    {
        return self
            .members
            .iter()
            .filter(|member| return member.family == family)
            .collect();
    }

    #[must_use]
    pub fn Tally(&self, family: Restored) -> Tally
    {
        let mut tally = Tally::default();

        for member in self.In(family)
        {
            let counter = match member.fate
            {
                Fate::Preserved { .. } => &mut tally.preserved,
                Fate::Hollowed { .. } => &mut tally.hollowed,
                Fate::Mentioned { .. } => &mut tally.mentioned,
                Fate::Gone => &mut tally.gone,
            };
            *counter = counter.saturating_add(1);
        }

        return tally;
    }

    #[must_use]
    pub fn Named(&self, name: &str) -> Option<&MemberFate>
    {
        return self
            .members
            .iter()
            .find(|member| return member.name == name || member.id == name);
    }

    #[must_use]
    pub fn Summary(&self) -> String
    {
        let mut lines = vec![
            format!("{} -> {}", self.from, self.to),
            format!(
                "  documents: {} appeared, {} disappeared, {} changed in place, {} relocated",
                self.documents.appeared.len(),
                self.documents.disappeared.len(),
                self.documents.changed.len(),
                self.documents.relocated.len()
            ),
        ];

        for family in Restored::All()
        {
            let tally = self.Tally(*family);
            if tally.Total() == 0
            {
                continue;
            }

            let named: Vec<&str> = self
                .In(*family)
                .iter()
                .filter(|member| return !matches!(member.fate, Fate::Preserved { .. }))
                .take(3)
                .map(|member| return member.name.as_str())
                .collect();

            let mut line = format!(
                "  {}: {} preserved, {} hollowed, {} mentioned, {} gone",
                family.Label(),
                tally.preserved,
                tally.hollowed,
                tally.mentioned,
                tally.gone
            );
            if !named.is_empty()
            {
                let _ = write!(line, " ({})", named.join(", "));
            }
            lines.push(line);
        }

        let mut filler = format!(
            "  filler: {} documents the blocklist matches, {} carrying nothing but filler",
            self.filler.declared.len(),
            self.filler.stubs.len()
        );
        if let Some(widest) = self.filler.Widest_Undeclared()
        {
            let _ = write!(
                filler,
                "\n  undeclared: {} sections across {} documents stand on \"{}\"",
                widest.sections,
                widest.documents.len(),
                widest.text.chars().take(72).collect::<String>()
            );
        }
        lines.push(filler);

        return lines.join("\n");
    }
}

pub fn Regression(from: &Revision, to: &Revision) -> Result<RegressionReport, IngestError>
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

    let pair = One_Pair(&from.Fingerprint()?, &to.Fingerprint()?)?;
    let later = Later::Read(&to.documents);

    let mut members = Vec::new();
    for (document, markdown) in &volumes
    {
        for member in Extract(document, markdown)?
        {
            let judged = Judge(&member, &later, &to.documents);
            members.push(judged);
        }
    }

    return Ok(RegressionReport {
        from: from.label.clone(),
        to: to.label.clone(),
        documents: Relocations(&pair, &from.Fingerprint()?, &to.Fingerprint()?),
        members,
        filler: Census(&later),
    });
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
    let mut origins: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for path in &pair.disappeared
    {
        if let Some(hash) = from.documents.get(path)
        {
            origins.entry(hash.as_str()).or_default().push(path.clone());
        }
    }

    let mut fate = DocumentFate {
        changed: pair.changed.clone(),
        ..DocumentFate::default()
    };
    let mut moved: BTreeSet<&str> = BTreeSet::new();

    for path in &pair.appeared
    {
        match to.documents.get(path).and_then(|hash| return origins.get(hash.as_str()))
        {
            Some(from_paths) =>
            {
                moved.extend(from_paths.iter().map(String::as_str));
                fate.relocated.push(Relocation {
                    to: path.clone(),
                    from: from_paths.clone(),
                });
            }
            None => fate.appeared.push(path.clone()),
        }
    }

    fate.disappeared = pair
        .disappeared
        .iter()
        .filter(|path| return !moved.contains(path.as_str()))
        .cloned()
        .collect();

    return fate;
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
    fn Read(documents: &BTreeMap<String, String>) -> Self
    {
        let mut sections: Vec<(String, String, Vec<SourceBlock>)> = Vec::new();
        let mut templates: BTreeMap<String, Repetition> = BTreeMap::new();
        let mut later = Self {
            authored: BTreeMap::new(),
            named_in_row: BTreeMap::new(),
            templates: BTreeMap::new(),
            declared: BTreeSet::new(),
            bodies: BTreeMap::new(),
        };

        for (path, markdown) in documents
        {
            let mut title: Option<String> = None;
            let mut body: Vec<SourceBlock> = Vec::new();

            for block in Segment(markdown)
            {
                if block.kind != BlockKind::Heading
                {
                    Note_Block(&mut later, path, &block);
                    body.push(block);
                    continue;
                }

                let heading = Title(&block);
                if let Some(previous) = title.replace(heading)
                {
                    sections.push((path.clone(), previous, core::mem::take(&mut body)));
                }
            }

            if let Some(last) = title
            {
                sections.push((path.clone(), last, body));
            }
        }

        for (path, title, body) in &sections
        {
            for block in Keyable(body)
            {
                let key = Template_Key(&block.text, title);
                let repetition = templates.entry(key).or_default();
                repetition.sections = repetition.sections.saturating_add(1);
                repetition.documents.insert(path.clone());
            }
        }
        later.templates = templates;

        for (path, title, body) in &sections
        {
            let mut strongest: Option<Body> = None;
            for block in Keyable(body)
            {
                let shape = later.Shape(&block.text, title);
                let counted = later.bodies.entry(path.clone()).or_default();
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

            later
                .authored
                .entry(title.clone())
                .or_default()
                .push(Position::Heading {
                    document: path.clone(),
                    body: strongest,
                });
        }

        return later;
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
        let subject = row
            .cells
            .iter()
            .map(|cell| return cell.trim())
            .find(|cell| return !cell.is_empty());
        let Some(cell) = subject
        else
        {
            continue;
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

fn Still(member: &Member, later: &Later) -> Option<Fate>
{
    if member.family == Restored::CanonicalDomainModel
    {
        if let Some(document) = later.named_in_row.get(&member.name)
        {
            return Some(Fate::Preserved {
                document: document.clone(),
            });
        }
    }

    let positions = later.authored.get(&member.name)?;
    let mut hollow: Option<Fate> = None;

    for position in positions
    {
        match position
        {
            Position::Row { document } =>
            {
                return Some(Fate::Preserved {
                    document: document.clone(),
                })
            }
            Position::Heading { document, body } => match body
            {
                Some(Body::Narrative) =>
                {
                    return Some(Fate::Preserved {
                        document: document.clone(),
                    })
                }
                Some(Body::Template {
                    shared_with,
                    declared,
                }) =>
                {
                    hollow.get_or_insert(Fate::Hollowed {
                        document: document.clone(),
                        evidence: Hollow::Template {
                            shared_with: *shared_with,
                            declared: *declared,
                        },
                    });
                }
                None =>
                {
                    hollow.get_or_insert(Fate::Hollowed {
                        document: document.clone(),
                        evidence: Hollow::NoBody,
                    });
                }
            },
        }
    }

    return hollow;
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
    let mut census = FillerCensus {
        declared: later.declared.iter().cloned().collect(),
        ..FillerCensus::default()
    };

    for (key, repetition) in &later.templates
    {
        if repetition.sections < SHARED_BY
        {
            continue;
        }
        census.templates.push(Template {
            text: key.clone(),
            sections: repetition.sections,
            documents: repetition.documents.iter().cloned().collect(),
            declared: Is_Filler(key),
        });
    }
    census.templates.sort_by(|first, second| {
        return second
            .sections
            .cmp(&first.sections)
            .then_with(|| return first.text.cmp(&second.text));
    });

    for (path, bodies) in &later.bodies
    {
        if bodies.blocks > 0 && bodies.filler == bodies.blocks
        {
            census.stubs.push(path.clone());
        }
    }

    return census;
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
