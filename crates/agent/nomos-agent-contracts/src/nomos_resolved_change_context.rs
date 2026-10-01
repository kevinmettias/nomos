//! The Nomos-resolved half of `AGT-007`'s `PrepareChangeContext` operation.

use std::collections::BTreeMap;

use nomos_contracts::{Applicability, BuildVariantId, CapabilityId, RuleId, SnapshotId};
use nomos_scope_verification::{Territory, VerificationPredicate};
use serde::{Deserialize, Serialize};

mod change_context_subject;

pub use change_context_subject::ChangeContextSubject;

/// `AGT-007`: "Nomos shall expose a read-only `PrepareChangeContext` operation...
/// Given a pinned repository snapshot plus a symbol, selected scope, or task
/// description, it shall combine `KnowledgeWorkbench` architecture context, claims,
/// design decisions, examples, contradictions, and historical evidence with
/// Nomos-resolved applicability, permitted scope, required verification, prohibited
/// actions, evidence gaps, approved capabilities, and task-envelope inputs."
///
/// Only the Nomos-resolved half. The `KnowledgeWorkbench`-sourced half (architecture
/// context, claims, decisions, examples, contradictions, historical evidence) belongs
/// to the sibling product per `ARC-ECOSYSTEM-001` -- a composing layer pairs this type
/// with that other, separately-owned half to form the full composite result. Every
/// item in that other half must stay attributed to `KnowledgeWorkbench`'s own
/// namespace and must never be represented as this type's content.
///
/// `evidence_gaps` is deliberately omitted: `AGT-007` names it as one of the seven
/// Nomos-resolved ingredients, but the only place the corpus gives `EvidenceGap` a
/// concrete shape is the unrelated `OBS-002` evidence cluster, a different
/// requirement's territory that is itself unbuilt anywhere in this workspace.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NomosResolvedChangeContext
{
    /// "a pinned repository snapshot" (`AGT-007`).
    pub snapshot: SnapshotId,
    /// "for a pinned snapshot and build variant" (`PrepareChangeContext`'s canonical
    /// model, `02-core-architecture-identity-and-configuration.md`).
    pub variant: BuildVariantId,
    /// "a symbol, selected scope, or task description" (`AGT-007`) -- the request
    /// this context was resolved for, echoed back in the result.
    pub requested: ChangeContextSubject,
    /// "Nomos-resolved applicability" (`AGT-007`) / "effective applicability"
    /// (canonical model) -- per matched rule, for the requested scope.
    pub applicability: BTreeMap<RuleId, Applicability>,
    /// "permitted scope" (`AGT-007`) / "Nomos-resolved scope" (canonical model).
    pub permitted_scope: Territory,
    /// "required verification" (`AGT-007`, canonical model, and workflow 9.21).
    pub required_verification: Vec<VerificationPredicate>,
    /// "prohibited actions" (`AGT-007`, canonical model, 9.21). Free-text: the only
    /// concrete shape the corpus gives this field is `ChangeIntent`'s worked example
    /// ("Prohibited effects: removing existing strategies; changing workspace
    /// topology; ..."), a sentence rather than a closed enum.
    pub prohibited_actions: Vec<String>,
    /// "approved capabilities" (`AGT-007`) / "permitted capabilities" (canonical
    /// model).
    pub approved_capabilities: Vec<CapabilityId>,
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::Digest128;

    /// The byte every snapshot digest in these tests is filled with. Distinct from
    /// [`BUILD_VARIANT_SEED`] so a context that swapped the two identities could not
    /// still compare equal.
    const SNAPSHOT_SEED: u8 = 3;

    /// The byte every build-variant digest in these tests is filled with; see
    /// [`SNAPSHOT_SEED`].
    const BUILD_VARIANT_SEED: u8 = 4;

    #[test]
    fn Test_A_Context_Carries_Exactly_What_It_Was_Given()
    {
        let mut applicability = BTreeMap::new();
        applicability.insert(RuleId::New("check-naming-convention"), Applicability::Supported);

        let context = NomosResolvedChangeContext {
            snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([SNAPSHOT_SEED; Digest128::BYTE_LENGTH])),
            variant: BuildVariantId::From_Digest(Digest128::From_Bytes([BUILD_VARIANT_SEED; Digest128::BYTE_LENGTH])),
            requested: ChangeContextSubject::TaskDescription("close AGT-007's gap".to_owned()),
            applicability,
            permitted_scope: Territory::Of_Files(["crates/agent/nomos-agent-contracts"]),
            required_verification: vec![VerificationPredicate::From_String_Arguments(vec!["cargo".to_owned(), "test".to_owned(), "-p".to_owned(), "nomos-agent-contracts".to_owned()])],
            prohibited_actions: vec!["removing existing strategies".to_owned()],
            approved_capabilities: vec![CapabilityId::New("nomos.cap.example.change_context_test_only")],
        };

        assert_eq!(context.applicability.len(), 1);
        assert_eq!(context.permitted_scope.paths, ["crates/agent/nomos-agent-contracts"]);
        assert_eq!(context.required_verification.len(), 1);
        assert_eq!(context.prohibited_actions, ["removing existing strategies"]);
        assert_eq!(context.approved_capabilities.len(), 1);
    }
}
