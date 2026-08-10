//! Every way a stored block disagrees with its source.

/// Why a recomputed block disagreed with the manifest.
///
/// Named per block rather than summed. A count tells you the segmenter is wrong; the
/// ordinal and the kind tell you where.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockMismatch
{
    CountDiffers
    {
        document: String,
        recorded: usize,
        recomputed: usize,
    },
    Kind
    {
        document: String,
        ordinal: u32,
        recorded: String,
        recomputed: String,
    },
    ContentHash
    {
        document: String,
        ordinal: u32,
        recorded: String,
        recomputed: String,
    },
    NormalizedHash
    {
        document: String,
        ordinal: u32,
        recorded: String,
        recomputed: String,
    },
    DocumentMissing
    {
        document: String,
    },
}

impl BlockMismatch
{
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::CountDiffers {
                document,
                recorded,
                recomputed,
            } => format!("{document}: v14 recorded {recorded} blocks, segmentation produced {recomputed}"),
            Self::Kind {
                document,
                ordinal,
                recorded,
                recomputed,
            } => format!("{document}#{ordinal}: kind {recorded} recomputed as {recomputed}"),
            Self::ContentHash {
                document,
                ordinal,
                recorded,
                recomputed,
            } => format!("{document}#{ordinal}: content_hash {recorded} recomputed as {recomputed}"),
            Self::NormalizedHash {
                document,
                ordinal,
                recorded,
                recomputed,
            } => format!("{document}#{ordinal}: normalized_hash {recorded} recomputed as {recomputed}"),
            Self::DocumentMissing { document } => {
                format!("{document}: recorded in the manifest, not found in the source tree")
            }
        };
    }
}
