use crate::RecordedBlock;
use crate::BlockLineage;
use crate::BlockField;
use crate::BlockMismatch;
use crate::BlockMismatchKind;
use nomos_spec_model::{ContentHash, Segment, SourceBlock};
use std::collections::BTreeMap;

/// The result of checking our segmentation against v14's manifest.
///
/// # What this gate does and does not establish
///
/// It compares **v14's segmentation** against itself. v14's block kinds are `prose`,
/// `heading` and `code` — there is no `table_row`, so a table is one prose block and a
/// row that vanishes from inside it violates no per-block disposition. That is how 282
/// rows were lost without the manifest noticing.
///
/// Reproducing these hashes therefore proves our reader agrees with v14 about what v14
/// saw. It does not prove the corpus is intact at the granularity Nomos needs, and the
/// finer segmentation that catches row-level loss will not match this manifest by
/// construction. Two segmentations, two checks; this one names which it is.
#[derive(Debug, Default)]
pub struct GateReport
{
    pub documents_checked: u32,
    pub blocks_checked: u32,
    pub discriminating_blocks: u32,
    pub mismatches: Vec<BlockMismatch>,
}

impl GateReport
{
    #[must_use]
    pub fn Is_Passing(&self) -> bool
    {
        return self.mismatches.is_empty() && self.blocks_checked > 0;
    }

    /// Whether the manifest actually exercised the normalizer.
    ///
    /// Only blocks whose `content_hash` and `normalized_hash` differ do. In v14's corpus
    /// there are thirty, all tables. A run that checked none of them has verified
    /// hashing and not normalization, and must not be reported as having verified both.
    #[must_use]
    pub fn Has_Exercised_The_Normalizer(&self) -> bool
    {
        return self.discriminating_blocks > 0;
    }

    #[must_use]
    pub fn Summary(&self) -> String
    {
        return format!(
            "{} documents, {} blocks, {} of which discriminate the normalizer; {} mismatch(es)",
            self.documents_checked,
            self.blocks_checked,
            self.discriminating_blocks,
            self.mismatches.len()
        );
    }
}

/// # Errors
///
/// Returns a message if the manifest cannot be parsed.
pub fn Parse_Block_Lineage(yaml: &str) -> Result<BlockLineage, String>
{
    return serde_yaml_ng::from_str(yaml).map_err(|error| format!("block lineage: {error}"));
}

/// Recomputes every recorded block and reports every disagreement.
///
/// `documents` maps a source document name to its markdown.
#[must_use]
pub fn Check_Against_Manifest(
    lineage: &BlockLineage,
    documents: &BTreeMap<String, String>,
) -> GateReport
{
    let mut report = GateReport::default();

    for (name, recorded) in By_Document(lineage)
    {
        let Some(markdown) = documents.get(name)
        else
        {
            report.mismatches.push(BlockMismatch {
                document: name.to_owned(),
                kind: BlockMismatchKind::DocumentMissing,
            });
            continue;
        };

        Check_Document(name, &recorded, markdown, &mut report);
    }

    return report;
}

/// The recorded blocks, grouped by the document each came from.
fn By_Document(lineage: &BlockLineage) -> BTreeMap<&str, Vec<&RecordedBlock>>
{
    let mut grouped: BTreeMap<&str, Vec<&RecordedBlock>> = BTreeMap::new();

    for block in &lineage.blocks
    {
        grouped.entry(&block.source_document).or_default().push(block);
    }

    return grouped;
}

/// Recomputes one document's blocks against what the manifest recorded.
///
/// A count that differs is reported and the walk continues over the shorter of the two,
/// because the first few blocks usually still line up and the ordinals where they stop
/// doing so are the useful part of the answer.
fn Check_Document(
    name: &str,
    recorded: &[&RecordedBlock],
    markdown: &str,
    report: &mut GateReport,
)
{
    report.documents_checked = report.documents_checked.saturating_add(1);
    let recomputed = Segment(markdown);
    if recomputed.len() != recorded.len()
    {
        report.mismatches.push(BlockMismatch {
            document: name.to_owned(),
            kind: BlockMismatchKind::CountDiffers {
                recorded: recorded.len(),
                recomputed: recomputed.len(),
            },
        });
    }

    for (want, got) in recorded.iter().zip(&recomputed)
    {
        report.blocks_checked = report.blocks_checked.saturating_add(1);
        if want.content_hash != want.normalized_hash
        {
            report.discriminating_blocks = report.discriminating_blocks.saturating_add(1);
        }
        Compare_Block(name, want, got, &mut report.mismatches);
    }
}

fn Compare_Block(
    document: &str,
    want: &RecordedBlock,
    got: &SourceBlock,
    mismatches: &mut Vec<BlockMismatch>,
)
{
    Compare_Kind(document, want, got, mismatches);
    Compare_Content(document, want, got, mismatches);
    Compare_Normalized(document, want, got, mismatches);
}

/// What the segmenter now calls the block.
fn Compare_Kind(
    document: &str,
    want: &RecordedBlock,
    got: &SourceBlock,
    mismatches: &mut Vec<BlockMismatch>,
)
{
    let kind = nomos_spec_store::Kind_Label(got.kind);
    if kind != want.block_kind
    {
        mismatches.push(BlockMismatch {
            document: document.to_owned(),
            kind: BlockMismatchKind::Block {
                ordinal: want.block_ordinal,
                field: BlockField::Kind,
                recorded: want.block_kind.clone(),
                recomputed: kind.to_owned(),
            },
        });
    }
}

/// The block's text exactly as it stands.
fn Compare_Content(
    document: &str,
    want: &RecordedBlock,
    got: &SourceBlock,
    mismatches: &mut Vec<BlockMismatch>,
)
{
    let content = ContentHash::Of(&got.text);
    if content.As_String_Slice() != want.content_hash
    {
        mismatches.push(BlockMismatch {
            document: document.to_owned(),
            kind: BlockMismatchKind::Block {
                ordinal: want.block_ordinal,
                field: BlockField::ContentHash,
                recorded: want.content_hash.clone(),
                recomputed: content.As_String_Slice().to_owned(),
            },
        });
    }
}

/// The normalized hash on its own.
///
/// It is checked separately from the content hash because the two disagreeing is the whole
/// signal: a block whose content moved but whose normalization did not is a reformatting,
/// and one where both moved is an edit.
fn Compare_Normalized(
    document: &str,
    want: &RecordedBlock,
    got: &SourceBlock,
    mismatches: &mut Vec<BlockMismatch>,
)
{
    let normalized = ContentHash::Of_Normalized(&got.text);
    if normalized.As_String_Slice() != want.normalized_hash
    {
        mismatches.push(BlockMismatch {
            document: document.to_owned(),
            kind: BlockMismatchKind::Block {
                ordinal: want.block_ordinal,
                field: BlockField::NormalizedHash,
                recorded: want.normalized_hash.clone(),
                recomputed: normalized.As_String_Slice().to_owned(),
            },
        });
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Parsed_Manifest(yaml: &str) -> BlockLineage
    {
        return Parse_Block_Lineage(yaml).expect("valid");
    }

    const DOC: &str = "# Title\n\nOne.\n";

    fn Documents() -> BTreeMap<String, String>
    {
        return BTreeMap::from([("a.md".to_owned(), DOC.to_owned())]);
    }

    fn Recorded() -> String
    {
        let heading = ContentHash::Of("# Title");
        let prose = ContentHash::Of("One.");
        return format!(
            "blocks:\n\
             - source_document: a.md\n  block_ordinal: 1\n  block_kind: heading\n  \
             content_hash: {heading}\n  normalized_hash: {heading}\n\
             - source_document: a.md\n  block_ordinal: 2\n  block_kind: prose\n  \
             content_hash: {prose}\n  normalized_hash: {prose}\n"
        );
    }

    #[test]
    fn Test_A_Matching_Corpus_Should_Pass()
    {
        let report = Check_Against_Manifest(&Parsed_Manifest(&Recorded()), &Documents());

        assert!(report.Is_Passing(), "{:?}", report.mismatches);
        assert_eq!(report.blocks_checked, 2);
    }

    /// An empty manifest must not pass. A gate that checked nothing and reported clean is
    /// the defect this whole project exists to prevent.
    #[test]
    fn Test_An_Empty_Manifest_Should_Not_Pass()
    {
        let report = Check_Against_Manifest(&Parsed_Manifest("blocks: []"), &Documents());

        assert!(!report.Is_Passing(), "checking zero blocks is not a pass");
    }

    #[test]
    fn Test_A_Changed_Block_Should_Be_Named_Not_Counted()
    {
        let documents = BTreeMap::from([("a.md".to_owned(), "# Title\n\nAltered.\n".to_owned())]);

        let report = Check_Against_Manifest(&Parsed_Manifest(&Recorded()), &documents);

        assert!(!report.Is_Passing());
        assert!(
            report
                .mismatches
                .iter()
                .any(|mismatch| mismatch.Describe().contains("a.md#2")),
            "the mismatch must name the block: {:?}",
            report.mismatches
        );
    }

    #[test]
    fn Test_A_Missing_Document_Should_Be_Reported()
    {
        let report = Check_Against_Manifest(&Parsed_Manifest(&Recorded()), &BTreeMap::new());

        assert!(!report.Is_Passing());
        assert!(matches!(
            report.mismatches.first(),
            Some(BlockMismatch {
                kind: BlockMismatchKind::DocumentMissing,
                ..
            })
        ));
    }

    /// A manifest with no discriminating block verifies hashing and not normalization,
    /// and must say so rather than reporting an unqualified pass.
    #[test]
    fn Test_A_Manifest_Without_Tables_Should_Not_Claim_To_Exercise_The_Normalizer()
    {
        let report = Check_Against_Manifest(&Parsed_Manifest(&Recorded()), &Documents());

        assert!(report.Is_Passing());
        assert!(!report.Has_Exercised_The_Normalizer());
    }

    #[test]
    fn Test_A_Discriminating_Block_Should_Be_Counted()
    {
        let table = "| a | b |\n| --- | --- |\n| 1 | 2 |";
        let documents = BTreeMap::from([("t.md".to_owned(), format!("{table}\n"))]);
        let yaml = format!(
            "blocks:\n- source_document: t.md\n  block_ordinal: 1\n  block_kind: prose\n  \
             content_hash: {}\n  normalized_hash: {}\n",
            ContentHash::Of(table),
            ContentHash::Of_Normalized(table)
        );

        let report = Check_Against_Manifest(&Parsed_Manifest(&yaml), &documents);

        assert!(report.Is_Passing(), "{:?}", report.mismatches);
        assert_eq!(report.discriminating_blocks, 1);
        assert!(report.Has_Exercised_The_Normalizer());
    }
}
