use nomos_contracts::Digest128;
use serde::{Deserialize, Serialize};

use super::{ArtifactKind, EntityId};

/// A persisted or generated repository object.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifact
{
    /// Stable identity.
    pub id: EntityId,
    /// Repository-relative path. Navigation, never identity — a moved file is the same
    /// artifact, and a path that became the identity would say otherwise.
    pub path: String,
    /// What kind of object this is.
    pub kind: ArtifactKind,
    /// Digest of the content.
    pub content: Digest128,
    /// The generator that owns this artifact, when it is generated. Present means a
    /// correction targets the generator, not this file.
    pub generated_by: Option<String>,
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::digest::Content_Digest;

    /// A generated artifact must be distinguishable from a hand-written one, because a
    /// correction that edits generated output is overwritten the next time the
    /// generator runs and the finding comes straight back.
    #[test]
    fn Test_Generated_Artifacts_Should_Name_Their_Generator()
    {
        let generated = Artifact {
            id: EntityId::From_Digest(Content_Digest(b"api.g.rs")),
            path: "src/api.g.rs".to_owned(),
            kind: ArtifactKind::GeneratedOutput,
            content: Content_Digest(b"contents"),
            generated_by: Some("protoc".to_owned()),
        };

        assert_eq!(generated.generated_by.as_deref(), Some("protoc"));
    }
}
