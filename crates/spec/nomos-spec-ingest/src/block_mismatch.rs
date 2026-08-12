//! Every way a stored block disagrees with its source.

use crate::block_mismatch_kind::BlockMismatchKind;

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
pub struct BlockMismatch
{
    pub document: String,
    pub kind: BlockMismatchKind,
}

impl BlockMismatch
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
