use crate::phases::IngestError;
use nomos_spec_model::{ContentHash, Parse_Record, Segment};
use nomos_spec_store::{SpecificationStore, StoreError};
use core::fmt::Write as _;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The three v14 artifact families v15.0 was supposed to carry forward.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Family
{
    Requirement,
    Story,
    Acceptance,
}

impl Family
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Requirement => "requirement",
            Self::Story => "story",
            Self::Acceptance => "acceptance",
        };
    }

    /// The v14 directory each family is authored in.
    #[must_use]
    pub const fn Directory(self) -> &'static str
    {
        return match self
        {
            Self::Requirement => "requirements",
            Self::Story => "stories",
            Self::Acceptance => "acceptance",
        };
    }

    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[Self::Requirement, Self::Story, Self::Acceptance];
    }
}

/// One authored v14 artifact, reduced to what reconciliation compares.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Artifact
{
    pub id: String,
    pub family: Family,
    /// How many criteria the artifact declared. Zero for requirements and stories.
    pub criteria: usize,
    pub statement: String,
}

#[derive(Deserialize)]
struct ArtifactFrontMatter
{
    id: String,
    #[serde(default)]
    statement: String,
    /// Acceptance artifacts carry a list instead of one statement. Reconciliation needs
    /// one comparable text per identifier, so the criteria are joined in declared order —
    /// which also means a criterion silently dropped from the list changes the hash.
    #[serde(default)]
    criteria: Vec<Criterion>,
}

#[derive(Deserialize)]
struct Criterion
{
    id: String,
    #[serde(default)]
    statement: String,
}

/// What became of one v14 identifier in v15.0.
///
/// Three arms, and `Absent` is not a variant of `Reworded`. A family that vanished and a
/// family whose wording drifted are different failures, and collapsing them is how a
/// disappearance reads as an edit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Disposition
{
    Preserved,
    Reworded
    {
        v14: ContentHash,
        v15: ContentHash,
    },
    Absent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentifierOutcome
{
    pub id: String,
    pub family: Family,
    pub disposition: Disposition,
}

/// One row per v14 identifier, never a count.
///
/// The plan requires differences reported per identifier and never summarised, so the
/// report holds every outcome and the counts are queries over it. A report that stored
/// totals could not answer "which ones", which is the only question worth asking.
#[derive(Clone, Debug, Default)]
pub struct ReconciliationReport
{
    pub outcomes: Vec<IdentifierOutcome>,
}

impl ReconciliationReport
{
    #[must_use]
    pub fn Absent(&self) -> Vec<&IdentifierOutcome>
    {
        return self
            .outcomes
            .iter()
            .filter(|outcome| outcome.disposition == Disposition::Absent)
            .collect();
    }

    #[must_use]
    pub fn Reworded(&self) -> Vec<&IdentifierOutcome>
    {
        return self
            .outcomes
            .iter()
            .filter(|outcome| matches!(outcome.disposition, Disposition::Reworded { .. }))
            .collect();
    }

    #[must_use]
    pub fn Preserved_In(&self, family: Family) -> usize
    {
        return self
            .outcomes
            .iter()
            .filter(|outcome| outcome.family == family && outcome.disposition == Disposition::Preserved)
            .count();
    }

    #[must_use]
    pub fn Declared_In(&self, family: Family) -> usize
    {
        return self.outcomes.iter().filter(|outcome| outcome.family == family).count();
    }

    #[must_use]
    pub fn Absent_In(&self, family: Family) -> Vec<&str>
    {
        return self
            .Absent()
            .into_iter()
            .filter(|outcome| outcome.family == family)
            .map(|outcome| outcome.id.as_str())
            .collect();
    }

    /// Names families and identifiers, never a bare total.
    #[must_use]
    pub fn Summary(&self) -> String
    {
        let mut lines = Vec::new();
        for family in Family::All()
        {
            let absent = self.Absent_In(*family);
            let mut line = format!(
                "{}: {} of {} preserved",
                family.Label(),
                self.Preserved_In(*family),
                self.Declared_In(*family)
            );
            if !absent.is_empty()
            {
                let named: Vec<&str> = absent.iter().take(5).copied().collect();
                let _ = write!(
                    line,
                    ", {} absent ({}{})",
                    absent.len(),
                    named.join(", "),
                    if absent.len() > named.len() { ", …" } else { "" }
                );
            }
            lines.push(line);
        }

        return lines.join("\n");
    }
}

/// Reads a v14 artifact's declared identity and statement.
///
/// A lighter reader than [`Parse_Record`] on purpose: an artifact's heading is
/// `# ID - title`, which is not its title, so the record reader refuses it for naming
/// itself twice. That check is right for records and wrong here.
///
/// # Errors
///
/// Returns [`IngestError::Parse`] if the front matter is absent or unreadable.
pub fn Parse_Artifact(markdown: &str, family: Family) -> Result<Artifact, IngestError>
{
    let yaml = Front_Matter(markdown)
        .ok_or_else(|| return IngestError::Parse("artifact has no front matter".to_owned()))?;

    let front: ArtifactFrontMatter = serde_yaml_ng::from_str(yaml)
        .map_err(|error| IngestError::Parse(format!("artifact front matter: {error}")))?;

    let statement = if front.statement.trim().is_empty()
    {
        front
            .criteria
            .iter()
            .map(|criterion| return format!("{} {}", criterion.id, criterion.statement))
            .collect::<Vec<String>>()
            .join("\n")
    }
    else
    {
        front.statement
    };

    if statement.trim().is_empty()
    {
        return Err(IngestError::Parse(format!(
            "{} declares neither a statement nor any criteria, so there is nothing to reconcile",
            front.id
        )));
    }

    return Ok(Artifact {
        id: front.id,
        family,
        criteria: front.criteria.len(),
        statement,
    });
}

/// The front matter body, tolerating a byte order mark exactly as v14's readers did.
///
/// See D-131. The mark belongs to the opening fence and leaves with it.
fn Front_Matter(markdown: &str) -> Option<&str>
{
    let text = markdown.strip_prefix('\u{feff}').unwrap_or(markdown);
    let after = text.strip_prefix("---\n").or_else(|| return text.strip_prefix("---\r\n"))?;

    let mut offset = 0_usize;
    for line in after.split_inclusive('\n')
    {
        if line.trim_end() == "---"
        {
            return after.get(..offset);
        }
        offset = offset.checked_add(line.len())?;
    }

    return None;
}

/// Every `{#ID} statement` v15 carries inline, by identifier.
///
/// v15 dissolved the per-artifact documents into section prose, so an identifier's text
/// is a span inside a line rather than a file. Reconciliation has to read it where it
/// actually is.
#[must_use]
pub fn Statements_In(markdown: &str) -> BTreeMap<String, String>
{
    let mut found = BTreeMap::new();

    for line in markdown.split('\n')
    {
        let Some((before, after)) = line.split_once("{#")
        else
        {
            continue;
        };
        let _ = before;
        let Some((id, rest)) = after.split_once('}')
        else
        {
            continue;
        };
        if id.is_empty() || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            continue;
        }

        found.insert(id.to_owned(), rest.trim().to_owned());
    }

    return found;
}

/// Compares every v14 identifier against what v15 carries.
#[must_use]
pub fn Reconcile(v14: &[Artifact], v15: &BTreeMap<String, String>) -> ReconciliationReport
{
    let mut outcomes: Vec<IdentifierOutcome> = Vec::new();

    for artifact in v14
    {
        // Normalized, because v14 stores the statement as folded YAML and v15 stores it
        // as one line of prose. Comparing raw text would report every identifier reworded
        // on a difference no reader could see.
        let mine = ContentHash::Of_Normalized(&artifact.statement);
        let disposition = match v15.get(&artifact.id)
        {
            None => Disposition::Absent,
            Some(theirs) =>
            {
                let theirs = ContentHash::Of_Normalized(theirs);
                if theirs == mine
                {
                    Disposition::Preserved
                }
                else
                {
                    Disposition::Reworded {
                        v14: mine,
                        v15: theirs,
                    }
                }
            }
        };

        outcomes.push(IdentifierOutcome {
            id: artifact.id.clone(),
            family: artifact.family,
            disposition,
        });
    }

    outcomes.sort_by(|left, right| return left.id.cmp(&right.id));

    return ReconciliationReport { outcomes };
}

/// Prose that says a section exists without saying what it says.
///
/// The first two are v14's own blocklist, ported from `validate_bundle.py`. They match
/// nothing in v15.0 — v15's filler is differently worded — and are kept because the
/// recorded blocklist is what the corpus declared, not what one revision happened to
/// need. The rest were measured in v15.0 and account for 405 blocks across 99 files.
pub const FILLER_PATTERNS: &[&str] = &[
    "This section groups related specification material",
    "Detailed statements appear below with stable IDs and graph and projection tooling",
    "This section preserves the reference or explanatory material for",
    "See the normative content above; this schema heading makes the retained architecture record",
    "The requirements below are the normative detail; the owning domain source supplies",
    "This decision preserves the product/domain boundary and behavior described by the owning",
    "Alternatives include retaining the preceding architecture, duplicating the mechanism locally",
    "The accepted decision is the normative direction stated in this record",
    "Implementations and projections shall conform to the accepted decision and expose incompatibility",
];

/// The pattern a block matches, or `None`.
///
/// Returns which pattern rather than a bool, so a lineage row can name why a block was
/// judged filler instead of asserting it.
#[must_use]
pub fn Is_Filler(text: &str) -> Option<&'static str>
{
    return FILLER_PATTERNS
        .iter()
        .find(|pattern| text.contains(**pattern))
        .copied();
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OverlayReport
{
    pub documents: u32,
    pub blocks: u32,
    /// Blocks judged filler, each with the pattern that judged it and the v14 heading it
    /// displaced. Named, never counted.
    pub filler: Vec<FillerBlock>,
    pub records: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FillerBlock
{
    pub document: String,
    pub ordinal: u32,
    pub pattern: &'static str,
    /// The v14 heading this block's document corresponds to, when one does.
    pub displaced: Option<String>,
}

/// I4 — ingests one v15 document, recording filler as filler.
///
/// A filler block is stored, not discarded: a stub is evidence of what was lost, and
/// dropping it would leave the regression report with nothing to point at. What makes it
/// filler is the lineage row, which names the pattern that judged it and the v14 heading
/// it displaced.
///
/// # Errors
///
/// Returns [`IngestError`] on any store failure.
pub fn Ingest_Overlay_Document(
    store: &mut SpecificationStore,
    path: &str,
    markdown: &str,
    v14_headings: &BTreeMap<String, i64>,
    report: &mut OverlayReport,
) -> Result<(), IngestError>
{
    let document_uid = store.Put_Source_Document(path, "v15.0", markdown)?;
    let blocks = Segment(markdown);
    store.Put_Source_Blocks(document_uid, &blocks)?;

    let heading = blocks
        .first()
        .filter(|block| block.kind == nomos_spec_model::BlockKind::Heading)
        .map(|block| return block.text.clone());
    let displaced = heading.as_ref().filter(|text| v14_headings.contains_key(*text));

    for block in &blocks
    {
        report.blocks = report.blocks.saturating_add(1);

        let Some(pattern) = Is_Filler(&block.text)
        else
        {
            continue;
        };

        Record_Filler(store, document_uid, block.ordinal, pattern)?;
        report.filler.push(FillerBlock {
            document: path.to_owned(),
            ordinal: block.ordinal,
            pattern,
            displaced: displaced.cloned(),
        });
    }

    report.documents = report.documents.saturating_add(1);

    return Ok(());
}

fn Record_Filler(
    store: &mut SpecificationStore,
    document_uid: i64,
    ordinal: u32,
    pattern: &str,
) -> Result<(), StoreError>
{
    store.Connection().execute(
        "INSERT OR IGNORE INTO lineage (source_block_uid, disposition)
         SELECT uid, ?3 FROM source_blocks WHERE document_uid = ?1 AND ordinal = ?2",
        rusqlite::params![document_uid, ordinal, format!("regression-filler: {pattern}")],
    )?;

    return Ok(());
}

/// The records that exist only in v15, ingested as authored nodes.
///
/// They have to be in the store before the validator they govern is trusted, which is
/// why they are ingested rather than referenced.
///
/// # Errors
///
/// Returns [`IngestError::Parse`] if a record does not read.
pub fn Ingest_v15_Record(
    store: &mut SpecificationStore,
    path: &str,
    markdown: &str,
) -> Result<String, IngestError>
{
    let record = Parse_Record(markdown)
        .map_err(|error| IngestError::Parse(format!("{path}: {error}")))?;

    store.Upsert_Node(
        &record.front_matter.id,
        &record.front_matter.kind,
        &record.front_matter.authority,
        "document",
        &record.front_matter.title,
    )?;

    let document_uid = store.Put_Source_Document(path, "v15.0", markdown)?;
    store.Put_Source_Blocks(document_uid, &Segment(&record.body))?;

    return Ok(record.front_matter.id);
}

#[cfg(test)]
mod tests
{
    use super::*;

    const ARTIFACT: &str = "\u{feff}---\nid: MODEL-001\nkind: requirement\n\
                            statement: MODEL-001 Artifact represents persisted objects.\n\
                            ---\n\n# MODEL-001 - Artifact\n";

    #[test]
    fn Test_An_Artifact_Should_Read_Through_Its_Byte_Order_Mark()
    {
        let artifact = Parse_Artifact(ARTIFACT, Family::Requirement).expect("reads");

        assert_eq!(artifact.id, "MODEL-001");
        assert_eq!(artifact.statement, "MODEL-001 Artifact represents persisted objects.");
    }

    #[test]
    fn Test_An_Artifact_With_No_Statement_Should_Be_Refused()
    {
        let empty = ARTIFACT.replace("statement: MODEL-001 Artifact represents persisted objects.\n", "");

        assert!(Parse_Artifact(&empty, Family::Requirement).is_err());
    }

    #[test]
    fn Test_Inline_Identifiers_Should_Be_Read_Where_v15_Put_Them()
    {
        let found = Statements_In(
            "> **Requirement:** {#MODEL-001} MODEL-001 Artifact represents persisted objects.\n\
             Some prose with no identifier.\n",
        );

        assert_eq!(found.len(), 1);
        assert_eq!(
            found.get("MODEL-001").map(String::as_str),
            Some("MODEL-001 Artifact represents persisted objects.")
        );
    }

    #[test]
    fn Test_A_Preserved_Identifier_Should_Reconcile()
    {
        let v14 = vec![Parse_Artifact(ARTIFACT, Family::Requirement).expect("reads")];
        let mut v15 = BTreeMap::new();
        v15.insert(
            "MODEL-001".to_owned(),
            "MODEL-001  Artifact represents\npersisted objects.".to_owned(),
        );

        let report = Reconcile(&v14, &v15);

        assert_eq!(report.Preserved_In(Family::Requirement), 1, "{}", report.Summary());
        assert!(report.Absent().is_empty());
    }

    #[test]
    fn Test_A_Missing_Identifier_Should_Be_Absent_Not_Reworded()
    {
        let v14 = vec![Parse_Artifact(ARTIFACT, Family::Requirement).expect("reads")];

        let report = Reconcile(&v14, &BTreeMap::new());

        assert_eq!(report.Absent_In(Family::Requirement), vec!["MODEL-001"]);
        assert!(report.Reworded().is_empty(), "a disappearance was reported as an edit");
        assert!(report.Summary().contains("MODEL-001"), "the summary counts without naming");
    }

    #[test]
    fn Test_A_Changed_Statement_Should_Carry_Both_Hashes()
    {
        let v14 = vec![Parse_Artifact(ARTIFACT, Family::Requirement).expect("reads")];
        let mut v15 = BTreeMap::new();
        v15.insert("MODEL-001".to_owned(), "MODEL-001 Something else entirely.".to_owned());

        let report = Reconcile(&v14, &v15);

        let reworded = report.Reworded();
        let Some(outcome) = reworded.first()
        else
        {
            panic!("a changed statement must be reported: {}", report.Summary());
        };
        assert!(matches!(outcome.disposition, Disposition::Reworded { .. }));
    }

    /// Acceptance artifacts declare a list, not a statement, and joining it in declared
    /// order is what makes a silently dropped criterion change the hash.
    #[test]
    fn Test_An_Acceptance_Artifact_Should_Reconcile_Through_Its_Criteria()
    {
        const ACCEPTANCE: &str = concat!(
            "---
",
            "id: US-A-001-AC
",
            "kind: acceptance_criteria
",
            "criteria:
",
            "- id: US-A-001-AC-01
",
            "  statement: One holds
",
            "- id: US-A-001-AC-02
",
            "  statement: Two holds
",
            "---

# X
"
        );

        let artifact = Parse_Artifact(ACCEPTANCE, Family::Acceptance).expect("reads");

        assert_eq!(artifact.criteria, 2);
        assert!(artifact.statement.contains("US-A-001-AC-01 One holds"));
        assert!(artifact.statement.contains("US-A-001-AC-02 Two holds"));

        let dropped = ACCEPTANCE.replace("- id: US-A-001-AC-02
  statement: Two holds
", "");
        let shorter = Parse_Artifact(&dropped, Family::Acceptance).expect("reads");
        assert_ne!(
            shorter.statement, artifact.statement,
            "dropping a criterion did not change the text reconciliation hashes"
        );
    }

    #[test]
    fn Test_Filler_Should_Name_The_Pattern_That_Judged_It()
    {
        let judged = Is_Filler(
            "This section preserves the reference or explanatory material for Contents.",
        );

        assert_eq!(
            judged,
            Some("This section preserves the reference or explanatory material for")
        );
        assert_eq!(Is_Filler("Nomos uses a small identity kernel."), None);
    }
}
