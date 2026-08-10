//! I5 — the families v15.0 destroyed, restored from v14.36 as resolvable nodes.
//!
//! Every member is minted from the document rather than from a list kept here. A hand-kept
//! inventory of 170 members is a second authority that drifts from the corpus the moment
//! either changes, and the whole point of the restoration is that the corpus is the source.
//! So a family declares which volume it lives in and how its members are recognised, and
//! the members are whatever that recognition finds.
//!
//! Identifiers are minted from what the document already says — its own numbering for the
//! appendices and the roadmap, the authored name for a service, a glossary term or a
//! canonical domain model. A name the corpus did not give would be an identity this build
//! invented, and re-running the restoration against a corrected corpus would silently mint
//! a second one. Two members minting the same identifier is refused rather than merged.
//!
//! Counts are not asserted here. They live in `tests/corpus/families/counts.json` with the
//! extraction that produced each one, per D-132.

use crate::phases::IngestError;
use nomos_spec_model::{BlockKind, RowKind, Segment, SourceBlock, TableRow, Table_Rows};
use nomos_spec_store::{SpecificationStore, StoreError};
use core::fmt::Write as _;
use std::collections::BTreeMap;

/// The heading whose leaves are the service descriptions.
const SERVICES: &str = "6. Systems and subsystem responsibilities";

/// The heading whose table is the canonical domain model.
const DOMAIN_MODEL: &str = "5. Canonical domain model";

const GLOSSARY: &str = "Glossary";

const EXTENDED_TERMS: &str = "Extended operational terms";

/// A family v15.0 dropped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Restored
{
    RoadmapMilestone,
    Scenario,
    Service,
    AppendixD,
    AppendixH,
    HeadlessInventory,
    IdeProfile,
    GlossaryTerm,
    CanonicalDomainModel,
}

impl Restored
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::RoadmapMilestone => "roadmap_milestone",
            Self::Scenario => "scenario",
            Self::Service => "service",
            Self::AppendixD => "appendix_d",
            Self::AppendixH => "appendix_h",
            Self::HeadlessInventory => "headless_inventory",
            Self::IdeProfile => "ide_profile",
            Self::GlossaryTerm => "glossary_term",
            Self::CanonicalDomainModel => "canonical_domain_model",
        };
    }

    /// The node kind a restored member takes.
    #[must_use]
    pub const fn Node_Kind(self) -> &'static str
    {
        return match self
        {
            Self::RoadmapMilestone => "release",
            Self::Scenario => "scenario",
            Self::Service => "service",
            Self::AppendixD => "schema",
            Self::AppendixH => "section",
            Self::HeadlessInventory => "inventory",
            Self::IdeProfile => "projection_profile",
            Self::GlossaryTerm => "glossary_term",
            Self::CanonicalDomainModel => "concept",
        };
    }

    #[must_use]
    pub const fn Prefix(self) -> &'static str
    {
        return match self
        {
            Self::RoadmapMilestone => "RMAP",
            Self::Scenario => "SCEN",
            Self::Service => "SVC",
            Self::AppendixD => "APX-D",
            Self::AppendixH => "APX-H",
            Self::HeadlessInventory => "HLS",
            Self::IdeProfile => "IDE",
            Self::GlossaryTerm => "GLS",
            Self::CanonicalDomainModel => "CDM",
        };
    }

    /// The volume a family lives in, by filename stem.
    ///
    /// Scoped rather than searched tree-wide so a family's membership is the same set the
    /// register measured. A recognition that matched across every volume would count
    /// whatever else happened to be shaped like it.
    #[must_use]
    pub const fn Volume(self) -> &'static str
    {
        return match self
        {
            Self::RoadmapMilestone => "08-roadmap",
            Self::Scenario | Self::AppendixD | Self::GlossaryTerm => "09-reference",
            Self::Service | Self::CanonicalDomainModel => "02-core",
            Self::AppendixH => "06-agents",
            Self::HeadlessInventory | Self::IdeProfile => "07-clients",
        };
    }

    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[
            Self::RoadmapMilestone,
            Self::Scenario,
            Self::Service,
            Self::AppendixD,
            Self::AppendixH,
            Self::HeadlessInventory,
            Self::IdeProfile,
            Self::GlossaryTerm,
            Self::CanonicalDomainModel,
        ];
    }
}

/// Where a restored member came from.
///
/// A row is addressed as a row and not as the block around it. Thirty concepts all
/// pointing at one table block is not a lineage, and it is the property the restoration
/// exists to establish.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin
{
    Block
    {
        ordinal: u32,
    },
    Row
    {
        block_ordinal: u32,
        row_ordinal: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member
{
    pub id: String,
    pub family: Restored,
    /// The name as the document gives it.
    pub name: String,
    pub document: String,
    pub origin: Origin,
    /// The name an alias resolves, where the document names the member rather than
    /// numbering it. `None` for a heading whose title is a sentence.
    pub alias: Option<String>,
}

/// Two members that would take one identifier.
///
/// Refused rather than merged. Merging would give one node two origins and make "which row
/// did this come from" unanswerable, which is the question the restoration is for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Collision
{
    pub id: String,
    pub first: String,
    pub second: String,
}

#[derive(Clone, Debug, Default)]
pub struct RestorationReport
{
    /// Every member, named. Counts are queries over this.
    pub members: Vec<Member>,
    /// Names more than one restored member claims, so none of them takes it.
    ///
    /// The corpus really does define `Capability` twice — once as a canonical domain model
    /// and once as a glossary term — and they are two nodes. Handing the bare name to
    /// whichever volume was read first would make resolution depend on directory order and
    /// answer confidently with one of two right answers.
    pub ambiguous_names: Vec<String>,
    /// Aliases something outside the restoration already owned, per alias rather than
    /// counted. Left where they are: a restoration may not repoint another authority's
    /// name.
    pub contested_aliases: Vec<String>,
}

impl RestorationReport
{
    #[must_use]
    pub fn In(&self, family: Restored) -> Vec<&Member>
    {
        return self
            .members
            .iter()
            .filter(|member| return member.family == family)
            .collect();
    }

    #[must_use]
    pub fn Named(&self, name: &str) -> Option<&Member>
    {
        return self
            .members
            .iter()
            .find(|member| return member.name == name || member.id == name);
    }

    /// Names families and their members, never a bare total.
    #[must_use]
    pub fn Summary(&self) -> String
    {
        let mut lines = Vec::new();
        for family in Restored::All()
        {
            let members = self.In(*family);
            let named: Vec<&str> = members
                .iter()
                .take(3)
                .map(|member| return member.name.as_str())
                .collect();
            let mut line = format!("{}: {} restored", family.Label(), members.len());
            if !named.is_empty()
            {
                let _ = write!(
                    line,
                    " ({}{})",
                    named.join(", "),
                    if members.len() > named.len() { ", …" } else { "" }
                );
            }
            lines.push(line);
        }

        return lines.join("\n");
    }
}

/// Reads one volume's family members without touching a store.
///
/// Pure, so the recognition can be tested against a fixture rather than only against the
/// corpus, and so a caller can see what a restoration would mint before it mints it.
///
/// # Errors
///
/// Returns [`IngestError::Parse`] naming both members when two would take one identifier.
pub fn Extract(document: &str, markdown: &str) -> Result<Vec<Member>, IngestError>
{
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

fn Refuse_Collisions(members: &[Member]) -> Result<(), IngestError>
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

impl core::fmt::Display for Collision
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(
            formatter,
            "{} would identify both \"{}\" and \"{}\". Refusing to mint one node for two \
             members, because the second's origin would be unrecoverable",
            self.id, self.first, self.second
        );
    }
}

fn From_Heading(document: &str, block: &SourceBlock, members: &mut Vec<Member>)
{
    let depth = block.text.chars().take_while(|character| return *character == '#').count();
    let title = block.text.trim_start_matches('#').trim();
    let path = &block.heading_path;
    let origin = Origin::Block {
        ordinal: block.ordinal,
    };

    // The key rather than the identity, because every caller minted the identity from the
    // family it was already passing. Naming the family twice was an invitation to name two
    // different ones.
    let mut push = |family: Restored, key: &str, alias: Option<String>| {
        if !document.starts_with(family.Volume())
        {
            return;
        }
        members.push(Member {
            id: Identify(family, key),
            family,
            name: title.to_owned(),
            document: document.to_owned(),
            origin,
            alias,
        });
    };

    if depth == 3
    {
        if let Some(numbering) = Milestone(title)
        {
            push(Restored::RoadmapMilestone, &numbering, None);
        }
        if let Some(numbering) = Numbering(title, 'G', 1)
        {
            if title.contains("End-to-end scenario:")
            {
                push(Restored::Scenario, &numbering, None);
            }
        }
        if let Some(numbering) = Numbering(title, 'D', 1)
        {
            push(Restored::AppendixD, &numbering, None);
        }
        if let Some(numbering) = Numbering(title, 'H', 1)
        {
            push(Restored::AppendixH, &numbering, None);
        }
    }

    if depth == 4
    {
        if let Some(numbering) = Numbering(title, 'D', 2)
        {
            push(Restored::AppendixD, &numbering, None);
        }
        if let Some(numbering) = Numbering(title, 'E', 2)
        {
            push(Restored::HeadlessInventory, &numbering, None);
        }
        if let Some(numbering) = Numbering(title, 'F', 2)
        {
            push(Restored::IdeProfile, &numbering, None);
        }
        if Under(path, SERVICES) && title.split_whitespace().any(|word| return word == "Service")
        {
            push(Restored::Service, title, None);
        }
        if Under(path, EXTENDED_TERMS)
        {
            push(Restored::GlossaryTerm, title, Some(title.to_owned()));
        }
    }
}

fn From_Rows(document: &str, block: &SourceBlock, members: &mut Vec<Member>)
{
    let under = block.heading_path.last().map(String::as_str);
    let family = match under
    {
        Some(DOMAIN_MODEL) => Restored::CanonicalDomainModel,
        Some(GLOSSARY) => Restored::GlossaryTerm,
        _ => return,
    };

    if !document.starts_with(family.Volume())
    {
        return;
    }

    for row in Table_Rows(block).iter().filter(|row| return row.kind == RowKind::Content)
    {
        let Some(cell) = First_Cell(row)
        else
        {
            continue;
        };

        let names = match family
        {
            Restored::CanonicalDomainModel => Models_In(cell),
            _ => vec![cell],
        };

        for name in names
        {
            members.push(Member {
                id: Identify(family, name),
                family,
                name: name.to_owned(),
                document: document.to_owned(),
                origin: Origin::Row {
                    block_ordinal: block.ordinal,
                    row_ordinal: row.ordinal,
                },
                alias: Some(name.to_owned()),
            });
        }
    }
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
fn First_Cell(row: &TableRow) -> Option<&str>
{
    return row
        .cells
        .iter()
        .map(|cell| return cell.trim())
        .find(|cell| return !cell.is_empty());
}

fn Under(path: &[String], heading: &str) -> bool
{
    return path.iter().any(|step| return step == heading);
}

/// `Foundation 0` and `Release 3`, as the roadmap numbers itself.
///
/// Both are milestones; only seven of the eight are Releases, which is why the family is
/// named for the milestone and not for the release.
fn Milestone(title: &str) -> Option<String>
{
    let mut words = title.split_whitespace();
    let word = words.next()?;
    let number = words.next()?;

    if word != "Foundation" && word != "Release"
    {
        return None;
    }
    if number.is_empty() || !number.bytes().all(|byte| return byte.is_ascii_digit())
    {
        return None;
    }

    let initial = word.get(..1)?;
    return Some(format!("{initial}.{number}"));
}

/// `D.7.1` from `D.7.1 atlas-profile.build-cost`, when it has exactly `parts` numbers.
fn Numbering(title: &str, letter: char, parts: usize) -> Option<String>
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
    if !segments
        .iter()
        .skip(1)
        .all(|segment| return !segment.is_empty() && segment.bytes().all(|b| return b.is_ascii_digit()))
    {
        return None;
    }

    return Some(format!("{letter}{numbering}"));
}

/// The identifier, minted from the family's prefix and what the document already says.
fn Identify(family: Restored, name: &str) -> String
{
    return format!("{}-{}", family.Prefix(), Slug(name));
}

fn Slug(name: &str) -> String
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

/// I5 — restores every family across the corpus into a store that already holds it.
///
/// Takes the whole document set rather than one volume at a time, because both things
/// that have to be unique are corpus-wide. A minted identifier is stable across databases,
/// so two members colliding is a collision wherever they live; and a bare name resolving
/// to one node is a claim about the corpus, not about a file. Restoring volume by volume
/// would make both answers depend on the order the directory was read in.
///
/// Every document must already be ingested. Restoring from text the store never saw would
/// make the source-truth gate optional for exactly the content the restoration cares
/// about, and a node whose lineage points at a block nobody hashed is a node with a
/// provenance nothing checked.
///
/// # Errors
///
/// Returns [`IngestError::Parse`] if a document is not in the store at that revision or
/// two members collide, and [`IngestError::Store`] on any store failure.
pub fn Restore(
    store: &mut SpecificationStore,
    revision: &str,
    documents: &BTreeMap<String, String>,
) -> Result<RestorationReport, IngestError>
{
    let mut report = RestorationReport::default();
    let mut located: Vec<(i64, Member)> = Vec::new();

    for (document, markdown) in documents
    {
        let document_uid = Document_Uid(store, revision, document)?;
        for member in Extract(document, markdown)?
        {
            located.push((document_uid, member));
        }
    }

    let members: Vec<Member> = located.iter().map(|(_, member)| return member.clone()).collect();
    Refuse_Collisions(&members)?;
    report.ambiguous_names = Ambiguous(&members);

    for (document_uid, member) in located
    {
        let node_uid = store.Upsert_Node(
            &member.id,
            member.family.Node_Kind(),
            "canonical",
            "record",
            &member.name,
        )?;

        match member.origin
        {
            Origin::Block { ordinal } => Dispose_Block(store, document_uid, ordinal, node_uid)?,
            Origin::Row {
                block_ordinal,
                row_ordinal,
            } =>
            {
                let row_uid = store
                    .Table_Row_Uid(document_uid, block_ordinal, row_ordinal)?
                    .ok_or_else(|| {
                        return IngestError::Parse(format!(
                            "{} block {block_ordinal} row {row_ordinal} is not in the store, \
                             so {} would trace to nothing",
                            member.document, member.id
                        ));
                    })?;
                store.Put_Row_Lineage(row_uid, "preserved-verbatim", Some(node_uid))?;
            }
        }

        if let Some(alias) = &member.alias
        {
            if !report.ambiguous_names.contains(alias) && !Alias(store, alias, node_uid)?
            {
                report.contested_aliases.push(alias.clone());
            }
        }

        report.members.push(member);
    }

    return Ok(report);
}

/// Names claimed by more than one restored member.
///
/// Reported and withheld rather than resolved by a rule such as "the domain model wins".
/// Two nodes really do carry the name; picking one is an answer the corpus does not give.
fn Ambiguous(members: &[Member]) -> Vec<String>
{
    let mut claims: BTreeMap<&str, u32> = BTreeMap::new();
    for alias in members.iter().filter_map(|member| return member.alias.as_deref())
    {
        let counter = claims.entry(alias).or_insert(0);
        *counter = counter.saturating_add(1);
    }

    return claims
        .into_iter()
        .filter(|(_, claimed)| return *claimed > 1)
        .map(|(alias, _)| return alias.to_owned())
        .collect();
}

fn Document_Uid(
    store: &SpecificationStore,
    revision: &str,
    document: &str,
) -> Result<i64, IngestError>
{
    return store
        .Connection()
        .query_row(
            "SELECT uid FROM source_documents WHERE path = ?1 AND revision = ?2",
            rusqlite::params![document, revision],
            |row| row.get(0),
        )
        .map_err(|_| {
            return IngestError::Parse(format!(
                "{document} is not in the store at {revision}, so there is nothing for a \
                 restored node to trace back to"
            ));
        });
}

fn Dispose_Block(
    store: &mut SpecificationStore,
    document_uid: i64,
    ordinal: u32,
    node_uid: i64,
) -> Result<(), IngestError>
{
    let disposed = store.Connection().execute(
        "INSERT OR IGNORE INTO lineage (source_block_uid, disposition, target_node_uid)
         SELECT uid, 'preserved-verbatim', ?3 FROM source_blocks
         WHERE document_uid = ?1 AND ordinal = ?2",
        rusqlite::params![document_uid, ordinal, node_uid],
    );
    Sql(disposed)?;

    return Ok(());
}

/// Whether the alias now resolves to this node.
///
/// `false` where something else already owns it. Reported rather than ignored: an alias
/// silently pointing at another node makes "resolve by name" answer confidently and
/// wrongly, which is worse than not resolving at all.
fn Alias(store: &SpecificationStore, alias: &str, node_uid: i64) -> Result<bool, IngestError>
{
    let inserted = store.Connection().execute(
        "INSERT OR IGNORE INTO node_aliases (alias, node_uid) VALUES (?1, ?2)",
        rusqlite::params![alias, node_uid],
    );
    Sql(inserted)?;

    let selected_owner = store.Connection().query_row(
        "SELECT node_uid FROM node_aliases WHERE alias = ?1",
        rusqlite::params![alias],
        |row| row.get(0),
    );
    let owner: i64 = Sql(selected_owner)?;

    return Ok(owner == node_uid);
}

/// Resolves an identifier or an authored name to a node.
///
/// The alias table is what makes `WorkspaceContext` answer as well as `CDM-WORKSPACECONTEXT`.
/// Without it a restored concept resolves only by the identifier this build minted, and the
/// name the corpus actually uses would find nothing.
///
/// # Errors
///
/// Returns [`IngestError::Store`] on any store failure.
pub fn Resolve(store: &SpecificationStore, name: &str) -> Result<Option<i64>, IngestError>
{
    if let Some(uid) = store.Node_Uid(name)?
    {
        return Ok(Some(uid));
    }

    return Sql(store
        .Connection()
        .query_row(
            "SELECT node_uid FROM node_aliases WHERE alias = ?1",
            rusqlite::params![name],
            |row| row.get(0),
        )
        .map(Some)
        .or_else(|error| {
            return match error
            {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(other),
            };
        }));
}

fn Sql<T>(result: rusqlite::Result<T>) -> Result<T, IngestError>
{
    return result.map_err(|error| IngestError::Store(StoreError::Sql(error.to_string())));
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::phases::Ingest_Source_Document;

    const CORE: &str = "# Core\n\n## 5. Canonical domain model\n\n\
                        | Model | Responsibility |\n| --- | --- |\n\
                        | WorkspaceContext | Repository, branch, configuration. |\n\
                        | MetricTradeoffProjection | Cost against benefit. |\n\n\
                        ## 6. Systems and subsystem responsibilities\n\n\
                        ### 6.1 Change reasoning\n\n\
                        #### Counterfactual Analysis Service\n\nEvaluates proposals.\n\n\
                        #### CodeStewardshipAssignment\n\nA model, not a service.\n";

    const REFERENCE: &str = "# Reference\n\n## Glossary\n\n\
                             | Term | Definition |\n| --- | --- |\n\
                             | Applicability | Whether a rule can run. |\n\n\
                             ### Extended operational terms\n\n#### SavedView\n\nA saved query.\n\n\
                             ## Scenarios\n\n\
                             ### G.1 Scenario catalog and coverage\n\nA catalog.\n\n\
                             ### G.2 End-to-end scenario: add a strategy\n\nA scenario.\n";

    fn Corpus(documents: &[(&str, &str)]) -> (SpecificationStore, BTreeMap<String, String>)
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let mut set = BTreeMap::new();

        for (document, markdown) in documents
        {
            Ingest_Source_Document(&mut store, document, "v14.36", markdown).expect("ingests");
            set.insert((*document).to_owned(), (*markdown).to_owned());
        }

        return (store, set);
    }

    fn Core() -> (SpecificationStore, BTreeMap<String, String>)
    {
        return Corpus(&[("02-core.md", CORE)]);
    }

    #[test]
    fn Test_A_Domain_Model_Row_Should_Become_A_Concept_Named_After_Itself()
    {
        let members = Extract("02-core.md", CORE).expect("extracts");
        let models: Vec<&Member> = members
            .iter()
            .filter(|member| return member.family == Restored::CanonicalDomainModel)
            .collect();

        assert_eq!(models.len(), 2, "the header or the delimiter was minted as a model");
        assert_eq!(models.first().map(|member| member.id.as_str()), Some("CDM-WORKSPACECONTEXT"));
        assert_eq!(models.first().map(|member| member.name.as_str()), Some("WorkspaceContext"));
        assert!(matches!(models.first().map(|member| member.origin), Some(Origin::Row { .. })));
    }

    /// The reason the header kind exists. Without it the column titles mint a concept
    /// named after the column names.
    #[test]
    fn Test_The_Column_Titles_Should_Not_Become_A_Concept()
    {
        let members = Extract("02-core.md", CORE).expect("extracts");

        assert!(
            !members.iter().any(|member| return member.name == "Model"),
            "the header row was restored as a domain model"
        );
    }

    #[test]
    fn Test_Only_A_Leaf_Naming_A_Service_Should_Become_One()
    {
        let members = Extract("02-core.md", CORE).expect("extracts");
        let services: Vec<&str> = members
            .iter()
            .filter(|member| return member.family == Restored::Service)
            .map(|member| return member.name.as_str())
            .collect();

        assert_eq!(services, vec!["Counterfactual Analysis Service"]);
    }

    /// A section that catalogs the scenarios is not a scenario.
    #[test]
    fn Test_A_Catalog_Section_Should_Not_Become_A_Scenario()
    {
        let members = Extract("09-reference.md", REFERENCE).expect("extracts");
        let scenarios: Vec<&str> = members
            .iter()
            .filter(|member| return member.family == Restored::Scenario)
            .map(|member| return member.id.as_str())
            .collect();

        assert_eq!(scenarios, vec!["SCEN-G-2"]);
    }

    #[test]
    fn Test_The_Glossary_Should_Restore_Both_Of_Its_Shapes()
    {
        let members = Extract("09-reference.md", REFERENCE).expect("extracts");
        let terms: Vec<&str> = members
            .iter()
            .filter(|member| return member.family == Restored::GlossaryTerm)
            .map(|member| return member.id.as_str())
            .collect();

        assert_eq!(terms, vec!["GLS-APPLICABILITY", "GLS-SAVEDVIEW"]);
    }

    /// A family recognised in the wrong volume would count whatever happened to be shaped
    /// like it.
    #[test]
    fn Test_A_Family_Should_Not_Be_Recognised_Outside_Its_Volume()
    {
        assert!(Extract("07-clients.md", CORE).expect("extracts").is_empty());
    }

    #[test]
    fn Test_Two_Members_Taking_One_Identifier_Should_Be_Refused()
    {
        let doubled = "# X\n\n## Glossary\n\n| Term | Definition |\n| --- | --- |\n\
                       | Applicability | One. |\n| applicability | Two. |\n";

        let refusal = Extract("09-reference.md", doubled).expect_err("must refuse");

        assert!(format!("{refusal}").contains("GLS-APPLICABILITY"), "{refusal}");
    }

    #[test]
    fn Test_Milestones_Should_Number_Themselves_As_The_Roadmap_Does()
    {
        assert_eq!(Milestone("Foundation 0 — Protocol"), Some("F.0".to_owned()));
        assert_eq!(Milestone("Release 7 — Advanced"), Some("R.7".to_owned()));
        assert_eq!(Milestone("Release notes"), None);
        assert_eq!(Milestone("11.1 First usable product boundary"), None);
    }

    #[test]
    fn Test_Numbering_Should_Require_Exactly_Its_Depth()
    {
        assert_eq!(Numbering("D.7 Profiles", 'D', 1), Some("D.7".to_owned()));
        assert_eq!(Numbering("D.7 Profiles", 'D', 2), None);
        assert_eq!(Numbering("D.7.1 atlas", 'D', 2), Some("D.7.1".to_owned()));
        assert_eq!(Numbering("Design notes", 'D', 1), None);
    }

    #[test]
    fn Test_A_Restored_Concept_Should_Trace_To_Its_Own_Row()
    {
        let (mut store, documents) = Core();

        let report = Restore(&mut store, "v14.36", &documents).expect("restores");

        assert!(report.contested_aliases.is_empty(), "{:?}", report.contested_aliases);
        let traced: String = store
            .Connection()
            .query_row(
                "SELECT r.text FROM lineage l
                 JOIN source_table_rows r ON r.uid = l.source_table_row_uid
                 JOIN nodes n ON n.uid = l.target_node_uid
                 WHERE n.node_id = 'CDM-METRICTRADEOFFPROJECTION'",
                [],
                |row| row.get(0),
            )
            .expect("the concept traces to no row");

        assert!(traced.contains("MetricTradeoffProjection"), "traced to {traced}");
        assert!(!traced.contains("WorkspaceContext"), "traced to the whole table");
    }

    #[test]
    fn Test_A_Restored_Concept_Should_Resolve_By_Its_Authored_Name()
    {
        let (mut store, documents) = Core();
        Restore(&mut store, "v14.36", &documents).expect("restores");

        assert!(Resolve(&store, "CDM-WORKSPACECONTEXT").expect("resolves").is_some());
        assert!(
            Resolve(&store, "WorkspaceContext").expect("resolves").is_some(),
            "the name the corpus uses resolves to nothing"
        );
        assert!(Resolve(&store, "NoSuchModel").expect("resolves").is_none());
    }

    /// A name two members carry belongs to neither. Handing it to whichever volume was
    /// read first makes resolution depend on directory order and answer confidently with
    /// one of two right answers.
    #[test]
    fn Test_A_Name_Two_Members_Carry_Should_Resolve_To_Neither()
    {
        const SHARED: &str = "# Reference\n\n## Glossary\n\n\
                              | Term | Definition |\n| --- | --- |\n\
                              | WorkspaceContext | The term, not the model. |\n";

        let (mut store, documents) = Corpus(&[("02-core.md", CORE), ("09-reference.md", SHARED)]);

        let report = Restore(&mut store, "v14.36", &documents).expect("restores");

        assert_eq!(report.ambiguous_names, vec!["WorkspaceContext".to_owned()]);
        assert!(
            Resolve(&store, "WorkspaceContext").expect("resolves").is_none(),
            "an ambiguous name answered anyway"
        );
        assert!(
            Resolve(&store, "CDM-WORKSPACECONTEXT").expect("resolves").is_some(),
            "both nodes must still resolve by identifier"
        );
        assert!(Resolve(&store, "GLS-WORKSPACECONTEXT").expect("resolves").is_some());
    }

    #[test]
    fn Test_Restoring_Twice_Should_Change_Nothing()
    {
        let (mut store, documents) = Core();

        let first = Restore(&mut store, "v14.36", &documents).expect("restores");
        let nodes = store.Count(nomos_spec_store::Table::Nodes).expect("counts");
        let lineage = store.Count(nomos_spec_store::Table::Lineage).expect("counts");
        let second = Restore(&mut store, "v14.36", &documents).expect("restores again");

        assert_eq!(first.members, second.members);
        assert_eq!(store.Count(nomos_spec_store::Table::Nodes).expect("counts"), nodes);
        assert_eq!(store.Count(nomos_spec_store::Table::Lineage).expect("counts"), lineage);
        assert!(second.contested_aliases.is_empty(), "re-running contested its own aliases");
    }

    /// Restoring from text the store never saw would make the source-truth gate optional
    /// for exactly the content the restoration cares about.
    #[test]
    fn Test_Restoring_A_Document_The_Store_Does_Not_Hold_Should_Be_Refused()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let mut documents = BTreeMap::new();
        documents.insert("02-core.md".to_owned(), CORE.to_owned());

        let refusal = Restore(&mut store, "v14.36", &documents).expect_err("must refuse");

        assert!(format!("{refusal}").contains("not in the store"), "{refusal}");
    }

    /// A restoration may not repoint a name another authority already owns.
    #[test]
    fn Test_A_Contested_Alias_Should_Be_Reported_Rather_Than_Silently_Repointed()
    {
        let (mut store, documents) = Core();
        let other = store
            .Upsert_Node("OTHER-001", "concept", "canonical", "record", "Something else")
            .expect("mints");
        store
            .Connection()
            .execute(
                "INSERT INTO node_aliases (alias, node_uid) VALUES ('WorkspaceContext', ?1)",
                rusqlite::params![other],
            )
            .expect("takes the alias");

        let report = Restore(&mut store, "v14.36", &documents).expect("restores");

        assert_eq!(report.contested_aliases, vec!["WorkspaceContext".to_owned()]);
        assert_eq!(
            Resolve(&store, "WorkspaceContext").expect("resolves"),
            Some(other),
            "the contested alias was silently repointed"
        );
    }

    #[test]
    fn Test_The_Summary_Should_Name_Members_Rather_Than_Only_Count_Them()
    {
        let (mut store, documents) = Core();
        let report = Restore(&mut store, "v14.36", &documents).expect("restores");

        assert!(report.Summary().contains("WorkspaceContext"), "{}", report.Summary());
    }
}
