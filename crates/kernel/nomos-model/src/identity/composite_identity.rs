use serde::{Deserialize, Serialize};

use super::{IdentityPolicy, SourceProvenance, StructuralFingerprint};
use crate::EntityId;

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
        use crate::Digest_Of_Parts;

        let qualified = self.qualified_name.as_deref().unwrap_or_default();
        let signature = self.Significant_Signature();
        let generator = self.Significant_Generator();

        return EntityId::From_Digest(Digest_Of_Parts(&[
            self.language.as_bytes(),
            qualified.as_bytes(),
            signature.as_bytes(),
            self.structural.Digest().Bytes(),
            self.provenance.repository.as_bytes(),
            generator.as_bytes(),
        ]));
    }

    /// The signature, when the policy says overloads must be distinguished by it --
    /// empty otherwise, so a language without overloading pays nothing for it.
    fn Significant_Signature(&self) -> &str
    {
        if self.policy.distinguish_overloads
        {
            return self.signature.as_deref().unwrap_or_default();
        }

        return "";
    }

    /// The generator, when the policy says a generated declaration must be distinguished
    /// from a hand-written one -- empty otherwise.
    fn Significant_Generator(&self) -> &str
    {
        if self.policy.distinguish_generated
        {
            return self.provenance.generator.as_deref().unwrap_or_default();
        }

        return "";
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Content_Digest;

    struct Qualified<'a>(&'a str);
    struct Signature<'a>(&'a str);

    #[test]
    fn Test_Identity_Should_Be_Stable_For_The_Same_Components()
    {
        assert_eq!(
            Identity_From_Components(Qualified("crate::foo"), Signature("(u32) -> bool")).Entity_Id(),
            Identity_From_Components(Qualified("crate::foo"), Signature("(u32) -> bool")).Entity_Id()
        );
    }

    /// The case that justifies `distinguish_overloads`. Two declarations sharing a
    /// qualified name and differing only in signature must not collide, or a finding on
    /// one attaches to the other.
    #[test]
    fn Test_Overloads_Should_Be_Distinct_Under_The_Strict_Policy()
    {
        assert_ne!(
            Identity_From_Components(Qualified("crate::foo"), Signature("(u32) -> bool")).Entity_Id(),
            Identity_From_Components(Qualified("crate::foo"), Signature("(String) -> bool")).Entity_Id()
        );
    }

    /// And the case that justifies making it a policy rather than a constant: a
    /// language without overloading gains nothing from signature disambiguation, and
    /// paying for it means an identity that churns whenever a parameter type is
    /// reformatted.
    #[test]
    fn Test_Overloads_Should_Collapse_When_The_Policy_Says_So()
    {
        let mut lenient = Identity_From_Components(Qualified("crate::foo"), Signature("(u32) -> bool"));
        lenient.policy.distinguish_overloads = false;
        let mut other_signature = Identity_From_Components(Qualified("crate::foo"), Signature("(String) -> bool"));
        other_signature.policy.distinguish_overloads = false;

        assert_eq!(lenient.Entity_Id(), other_signature.Entity_Id());
    }

    /// A moved declaration keeps its identity. This is the whole reason path is not a
    /// component: renaming a directory must not orphan every suppression beneath it.
    #[test]
    fn Test_Identity_Should_Not_Depend_On_Location()
    {
        let mut moved = Identity_From_Components(Qualified("crate::foo"), Signature("(u32) -> bool"));
        moved.provenance.revision = "def456".to_owned();

        assert_eq!(
            moved.Entity_Id(),
            Identity_From_Components(Qualified("crate::foo"), Signature("(u32) -> bool")).Entity_Id()
        );
    }

    /// A generated declaration and a hand-written one with the same shape are different
    /// things: a correction may rewrite one and must retarget the generator for the
    /// other.
    #[test]
    fn Test_Generated_Declarations_Should_Be_Distinct_From_Authored_Ones()
    {
        let mut generated = Identity_From_Components(Qualified("crate::foo"), Signature("(u32) -> bool"));
        generated.provenance.generator = Some("protoc".to_owned());

        assert_ne!(
            generated.Entity_Id(),
            Identity_From_Components(Qualified("crate::foo"), Signature("(u32) -> bool")).Entity_Id()
        );
    }

    fn Identity_From_Components(qualified: Qualified<'_>, signature: Signature<'_>) -> CompositeIdentity
    {
        return CompositeIdentity {
            language: "rust".to_owned(),
            provider_native: None,
            qualified_name: Some(qualified.0.to_owned()),
            signature: Some(signature.0.to_owned()),
            structural: StructuralFingerprint::From_Digest(Content_Digest(b"shape")),
            provenance: SourceProvenance {
                repository: "nomos".to_owned(),
                revision: "abc123".to_owned(),
                generator: None,
            },
            policy: IdentityPolicy::STRICT,
        };
    }
}
