use super::StoreError;

#[derive(Debug)]
pub enum IngestError
{
    Store(StoreError),
    Parse(String),
    /// The I1 gate found a disagreement. Ingest stops here.
    GateFailed
    {
        summary: String,
        first: Vec<String>,
    },
}

impl core::fmt::Display for IngestError
{
    // `fmt` is the method name `core::fmt::Display` mandates; implementing the trait means
    // matching its signature exactly, so this is not a style choice available to rename.
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Store(error) => write!(formatter, "{error}"),
            Self::Parse(cause) => write!(formatter, "parse: {cause}"),
            Self::GateFailed { summary, first } => write!(
                formatter,
                "the source-truth gate failed ({summary}). Our segmentation disagrees with \
                 v14's manifest, so every hash computed downstream is untrustworthy:\n  {}",
                first.join("\n  ")
            ),
        };
    }
}

impl std::error::Error for IngestError
{}

impl From<StoreError> for IngestError
{
    fn from(error: StoreError) -> Self
    {
        return Self::Store(error);
    }
}
