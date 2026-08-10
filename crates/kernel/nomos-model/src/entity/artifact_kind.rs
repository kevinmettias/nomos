use serde::{Deserialize, Serialize};

/// What kind of persisted object an [`Artifact`](super::Artifact) is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArtifactKind
{
    /// A source file.
    SourceFile,
    /// A project or build manifest.
    Manifest,
    /// A configuration file.
    Configuration,
    /// A schema or contract definition.
    Schema,
    /// A build output.
    Binary,
    /// An output produced by a generator, which a correction must retarget rather than
    /// edit.
    GeneratedOutput,
    /// Documentation.
    Document,
}
