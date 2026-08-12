//! The identity of a fact, and whether the store already holds it.
//!
//! A key is what makes a second run cheap and a wrong key is what makes it wrong, so the
//! digest inputs are gathered in one place rather than at each call site that needs one.

use super::{
    CapabilityId, Context, Digest128, Digest_Of_Parts, FactIdentity, FactKey, FactStore,
    GuaranteeDigest, InputDigest, ProviderId, Slice, SourceFile, SubjectId, surface,
};

impl Slice
{
    pub(super) fn Context(&self) -> Context
    {
        return Context {
            // The workspace as it is now. Not a key component — see OD-ANALYSIS-001 — so
            // it moves freely with the workspace without re-addressing a single fact.
            snapshot: self.snapshot,
            variant: self.variant,
            configuration: self.configuration,
            generation: self.generation,
        };
    }

    /// The semantic inputs of a syntax fact.
    ///
    /// Recomputed here rather than obtained from `nomos-lang-rust`, deliberately. If the
    /// two disagreed, the rollup's [`FactReader::Require`] would look up a key the
    /// provider never wrote and the run would degrade — loudly, in `RunReport::degraded`.
    /// A shared helper would make the two agree by construction and this whole class of
    /// mismatch would become untestable.
    pub(super) fn Syntax_Inputs(source: &str) -> InputDigest
    {
        return InputDigest::Of(&[source.as_bytes()]);
    }

    /// The rollup's semantic inputs: the digests of every member's inputs, in corpus order.
    ///
    /// A digest over the members rather than over the directory's name, so that editing
    /// any member changes the rollup's identity. Keying it on the name alone would leave a
    /// stale rollup matching a key that is still current, which is the failure mode a
    /// content-addressed key exists to prevent.
    pub(super) fn Surface_Inputs(members: &[&SourceFile]) -> InputDigest
    {
        let parts: Vec<[u8; Digest128::BYTE_LENGTH]> = members
            .iter()
            .map(|member| return *Self::Syntax_Inputs(&member.source).Digest().Bytes())
            .collect();
        let borrowed: Vec<&[u8]> = parts.iter().map(|part| return part.as_slice()).collect();

        return InputDigest::From_Digest(Digest_Of_Parts(&borrowed));
    }

    pub(super) fn Surface_Key(&self, group: SubjectId, members: &[&SourceFile]) -> FactKey
    {
        return FactKey {
            contract: CapabilityId::New(surface::CAPABILITY),
            contract_version: surface::CONTRACT_VERSION,
            subject: group,
            semantic_inputs: Self::Surface_Inputs(members),
            provider: ProviderId::New(surface::PROVIDER),
            provider_version: surface::CONTRACT_VERSION,
            guarantee: GuaranteeDigest::Of(&surface::Declared_Guarantee()),
            variant: self.variant,
            configuration: self.configuration,
        };
    }

    /// Whether the store already holds this fact at the current generation.
    pub(super) fn Held(&self, key: &FactKey) -> bool
    {
        let identity = FactIdentity {
            key: key.clone(),
            generation: self.generation,
        };

        return self.store.Current(&identity, self.generation).is_some();
    }
}
