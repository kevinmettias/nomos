use serde::{Deserialize, Serialize};

/// Where a declaration came from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceProvenance
{
    /// The repository the declaration lives in.
    pub repository: String,
    /// The revision it was read at.
    pub revision: String,
    /// The generator that produced it, if it is generated.
    pub generator: Option<String>,
}
