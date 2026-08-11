//! Every way a stored block disagrees with its source.

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

/// Which of the three disagreements it was.
///
/// The three per-block comparisons are one variant carrying a [`BlockField`] rather than
/// three variants, because they differ only in which field disagreed. Each of them read
/// the same ordinal and the same recorded-against-recomputed pair, and the ordinal is what
/// a reader needs first from every one of them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockMismatchKind
{
    /// A document the manifest records and the source tree does not have.
    DocumentMissing,
    /// A document whose block count moved, so no ordinal is comparable past the shorter.
    CountDiffers
    {
        recorded: usize,
        recomputed: usize,
    },
    /// One block at one ordinal, disagreeing in one field.
    Block
    {
        ordinal: u32,
        field: BlockField,
        recorded: String,
        recomputed: String,
    },
}

/// Which field of a block disagreed.
///
/// The two hashes are separate answers rather than one, because the two disagreeing is
/// the whole signal: a block whose content moved but whose normalization did not is a
/// reformatting, and one where both moved is an edit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockField
{
    Kind,
    ContentHash,
    NormalizedHash,
}

impl BlockField
{
    /// The field's name as the manifest spells it.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Kind => "kind",
            Self::ContentHash => "content_hash",
            Self::NormalizedHash => "normalized_hash",
        };
    }
}
