//! Every way a stored block disagrees with its source.

use crate::MismatchKind as BlockMismatchKind;

/// Why a recomputed block disagreed with the manifest, always naming the document.
///
/// Named per block rather than summed. A count tells you the segmenter is wrong; the
/// ordinal and the kind tell you where.
///
/// The document is the type's and not the kind's. Every disagreement is a disagreement
/// *about one document*, so a reader grouping a report by document reads `document`
/// without matching on a disagreement it does not otherwise care about, and a new kind of
/// disagreement cannot forget to say which document it is about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mismatch
{
    pub document: String,
    pub kind: BlockMismatchKind,
}

impl Mismatch
{
    #[must_use]
    pub fn Describe(&self) -> String
    {
        let document = &self.document;

        return match self.kind
        {
            BlockMismatchKind::DocumentMissing =>
            {
                format!("{document}: recorded in the manifest, not found in the source tree")
            }
            BlockMismatchKind::CountDiffers {
                recorded,
                recomputed,
            } => format!("{document}: v14 recorded {recorded} blocks, segmentation produced {recomputed}"),
            BlockMismatchKind::Block {
                ordinal,
                field,
                ref recorded,
                ref recomputed,
            } => format!(
                "{document}#{ordinal}: {} {recorded} recomputed as {recomputed}",
                field.Label()
            ),
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The manifest's count in the fixture below, which disagrees with — and is larger than —
    /// `RECOMPUTED_BLOCKS`, so a `Describe` that read them the wrong way round would say so.
    const RECORDED_BLOCKS: usize = 3;

    /// The source tree's count for the same document, recomputed from the markdown.
    const RECOMPUTED_BLOCKS: usize = 2;

    /// The ordinal of the one block the two trees disagree about, in the document at `c.md`.
    const DISAGREEING_ORDINAL: u32 = 4;

    #[test]
    fn Test_Describe_Should_Name_The_Document_For_Every_Kind_Of_Disagreement()
    {
        let missing = Mismatch {
            document: "a.md".to_owned(),
            kind: BlockMismatchKind::DocumentMissing,
        };
        let count_differs = Mismatch {
            document: "b.md".to_owned(),
            kind: BlockMismatchKind::CountDiffers {
                recorded: RECORDED_BLOCKS,
                recomputed: RECOMPUTED_BLOCKS,
            },
        };
        let block = Mismatch {
            document: "c.md".to_owned(),
            kind: BlockMismatchKind::Block {
                ordinal: DISAGREEING_ORDINAL,
                field: crate::Field::Kind,
                recorded: "heading".to_owned(),
                recomputed: "prose".to_owned(),
            },
        };

        assert!(missing.Describe().starts_with("a.md:"));
        assert!(count_differs.Describe().starts_with("b.md:"));
        assert!(block.Describe().starts_with("c.md#4:"));
    }
}
