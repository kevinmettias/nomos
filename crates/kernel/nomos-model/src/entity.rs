//! The three canonical entity kinds.

use nomos_contracts::Digest128;
use serde::{Deserialize, Serialize};

/// Identity of a canonical entity, independent of any snapshot.
///
/// Answers "which thing is this" across time. Pair it with a snapshot to get a
/// [`crate::SnapshotEntity`], which answers "which thing is this, as it was then".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EntityId(Digest128);

impl EntityId
{
    /// Wraps a digest as an entity identity.
    #[must_use]
    pub const fn From_Digest(digest: Digest128) -> Self
    {
        return Self(digest);
    }

    /// The underlying digest.
    #[must_use]
    pub const fn Digest(&self) -> Digest128
    {
        return self.0;
    }
}

impl core::fmt::Display for EntityId
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return self.0.fmt(formatter);
    }
}

/// What kind of persisted object an [`Artifact`] is.
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

/// What kind of declaration a [`Symbol`] is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind
{
    /// A module or namespace.
    Module,
    /// A type declaration.
    Type,
    /// A callable.
    Function,
    /// A field or property.
    Field,
    /// A constant or static.
    Constant,
    /// A trait, interface or protocol.
    Interface,
    /// An anonymous callable.
    Closure,
    /// A block or region within a callable.
    Block,
}

/// A language-semantic declaration or executable unit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Symbol
{
    /// Stable identity.
    pub id: EntityId,
    /// Composite identity, from which [`Symbol::id`] is derived.
    pub identity: crate::CompositeIdentity,
    /// What kind of declaration this is.
    pub kind: SymbolKind,
    /// The enclosing symbol, if any.
    pub container: Option<EntityId>,
    /// The artifact this symbol is declared in.
    pub declared_in: EntityId,
}

/// What kind of non-code entity a [`Resource`] is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceKind
{
    /// A network endpoint.
    Endpoint,
    /// A message channel, topic or queue.
    Channel,
    /// A database or table.
    DataStore,
    /// A file treated as data rather than as source.
    DataFile,
    /// A device.
    Device,
    /// A service outside the workspace.
    ExternalService,
}

/// A non-code entity a rule or metric can address.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource
{
    /// Stable identity.
    pub id: EntityId,
    /// What kind of resource this is.
    pub kind: ResourceKind,
    /// How the resource is addressed, in its own domain's terms.
    pub locator: String,
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::digest::Content_Digest;

    #[test]
    fn Test_Entity_Id_Should_Render_As_Its_Digest()
    {
        let digest = Content_Digest(b"symbol");
        let id = EntityId::From_Digest(digest);

        assert_eq!(id.to_string(), digest.to_string());
    }

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
