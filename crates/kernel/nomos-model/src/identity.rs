//! Composite identity: what makes two observations the same thing.

use crate::digest::Digest_Of_Parts;
use crate::entity::EntityId;
use nomos_contracts::Digest128;
use serde::{Deserialize, Serialize};

/// A hash of a symbol's shape, independent of its name and location.
///
/// The component that survives a rename. Without it, renaming a function looks like
/// deleting one and adding another, every suppression attached to it is orphaned, and
/// its history restarts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StructuralFingerprint(Digest128);

impl StructuralFingerprint
{
    /// Wraps a digest as a fingerprint.
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

/// How ambiguous declarations are resolved into distinct identities.
///
/// These are the cases where two declarations can legitimately share a qualified name,
/// and each needs a stated answer rather than whatever the first implementation
/// happened to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityPolicy
{
    /// Whether overloads sharing a name are distinguished by signature.
    pub distinguish_overloads: bool,
    /// Whether generated declarations are distinguished from hand-written ones.
    pub distinguish_generated: bool,
    /// Whether declarations under different conditional-compilation configurations are
    /// distinct.
    pub distinguish_conditional_compilation: bool,
}

impl IdentityPolicy
{
    /// The policy Nomos applies unless a language package states otherwise.
    ///
    /// All three on. Every one of them off produces a collision that presents as a
    /// finding attached to the wrong declaration, which is worse than a missing
    /// finding because it sends someone to read code that is fine.
    pub const STRICT: Self = Self {
        distinguish_overloads: true,
        distinguish_generated: true,
        distinguish_conditional_compilation: true,
    };
}

/// Everything that decides whether two observations denote the same declaration.
///
/// Note what is absent: a path and a line number. That combination was the prototype's
/// identity, and it made every comparative feature a join with no key — a reformatting
/// commit read as a total rewrite, and no finding could survive its subject moving down
/// three lines.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompositeIdentity
{
    /// The language whose rules gave this declaration its meaning.
    pub language: String,
    /// The provider's own identifier for this declaration.
    ///
    /// Opaque, and deliberately never part of a public contract: a Roslyn symbol key or
    /// a rust-analyzer revision means nothing to a peer and everything to the provider
    /// that issued it.
    pub provider_native: Option<String>,
    /// The fully qualified name, where the language has one.
    pub qualified_name: Option<String>,
    /// The signature, where the language has one.
    pub signature: Option<String>,
    /// The shape hash that survives renaming.
    pub structural: StructuralFingerprint,
    /// Where the declaration came from.
    pub provenance: SourceProvenance,
    /// How ambiguity was resolved.
    pub policy: IdentityPolicy,
}

impl CompositeIdentity
{
    /// The entity identity derived from these components.
    ///
    /// Every component that the policy says is significant participates. The parts are
    /// length-framed by [`Digest_Of_Parts`], so a qualified name ending where a
    /// signature begins cannot collide with a different split of the same characters.
    #[must_use]
    pub fn Entity_Id(&self) -> EntityId
    {
        let qualified = self.qualified_name.as_deref().unwrap_or_default();
        let signature = if self.policy.distinguish_overloads
        {
            self.signature.as_deref().unwrap_or_default()
        }
        else
        {
            ""
        };
        let generator = if self.policy.distinguish_generated
        {
            self.provenance.generator.as_deref().unwrap_or_default()
        }
        else
        {
            ""
        };

        return EntityId::From_Digest(Digest_Of_Parts(&[
            self.language.as_bytes(),
            qualified.as_bytes(),
            signature.as_bytes(),
            self.structural.Digest().Bytes(),
            self.provenance.repository.as_bytes(),
            generator.as_bytes(),
        ]));
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::digest::Content_Digest;

    fn Identity(qualified: &str, signature: &str) -> CompositeIdentity
    {
        return CompositeIdentity {
            language: "rust".to_owned(),
            provider_native: None,
            qualified_name: Some(qualified.to_owned()),
            signature: Some(signature.to_owned()),
            structural: StructuralFingerprint::From_Digest(Content_Digest(b"shape")),
            provenance: SourceProvenance {
                repository: "nomos".to_owned(),
                revision: "abc123".to_owned(),
                generator: None,
            },
            policy: IdentityPolicy::STRICT,
        };
    }

    #[test]
    fn Test_Identity_Should_Be_Stable_For_The_Same_Components()
    {
        assert_eq!(
            Identity("crate::foo", "(u32) -> bool").Entity_Id(),
            Identity("crate::foo", "(u32) -> bool").Entity_Id()
        );
    }

    /// The case that justifies `distinguish_overloads`. Two declarations sharing a
    /// qualified name and differing only in signature must not collide, or a finding on
    /// one attaches to the other.
    #[test]
    fn Test_Overloads_Should_Be_Distinct_Under_The_Strict_Policy()
    {
        assert_ne!(
            Identity("crate::foo", "(u32) -> bool").Entity_Id(),
            Identity("crate::foo", "(String) -> bool").Entity_Id()
        );
    }

    /// And the case that justifies making it a policy rather than a constant: a
    /// language without overloading gains nothing from signature disambiguation, and
    /// paying for it means an identity that churns whenever a parameter type is
    /// reformatted.
    #[test]
    fn Test_Overloads_Should_Collapse_When_The_Policy_Says_So()
    {
        let mut lenient = Identity("crate::foo", "(u32) -> bool");
        lenient.policy.distinguish_overloads = false;
        let mut other_signature = Identity("crate::foo", "(String) -> bool");
        other_signature.policy.distinguish_overloads = false;

        assert_eq!(lenient.Entity_Id(), other_signature.Entity_Id());
    }

    /// A moved declaration keeps its identity. This is the whole reason path is not a
    /// component: renaming a directory must not orphan every suppression beneath it.
    #[test]
    fn Test_Identity_Should_Not_Depend_On_Location()
    {
        let mut moved = Identity("crate::foo", "(u32) -> bool");
        moved.provenance.revision = "def456".to_owned();

        assert_eq!(
            moved.Entity_Id(),
            Identity("crate::foo", "(u32) -> bool").Entity_Id()
        );
    }

    /// A generated declaration and a hand-written one with the same shape are different
    /// things: a correction may rewrite one and must retarget the generator for the
    /// other.
    #[test]
    fn Test_Generated_Declarations_Should_Be_Distinct_From_Authored_Ones()
    {
        let mut generated = Identity("crate::foo", "(u32) -> bool");
        generated.provenance.generator = Some("protoc".to_owned());

        assert_ne!(
            generated.Entity_Id(),
            Identity("crate::foo", "(u32) -> bool").Entity_Id()
        );
    }
}
