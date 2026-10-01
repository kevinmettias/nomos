//! What an agent-assisted operation is handed before it starts.

use nomos_contracts::{CapabilityId, KnowledgeReferenceId, RuleId, SchemaId};
use nomos_model_package::EffortLevel;
use nomos_scope_verification::Territory;
use serde::{Deserialize, Serialize};

/// `AGT-001`: "Before agent work, Nomos shall produce a typed task envelope containing
/// goal, scope, authoritative knowledge context, applicable rules, prohibited changes,
/// available tools, and expected output schema."
///
/// Seven of the eight fields below are `AGT-001`'s own list, each a direct reuse of an
/// existing type: `scope` and `prohibited_changes` both reuse [`Territory`] -- the same
/// shape, at two different membership senses (what may be touched, what may not).
/// `knowledge_context` carries [`KnowledgeReferenceId`] citations, never a
/// `KnowledgeWorkbench` claim's own content -- the bounded-projection role that identity
/// already plays, and the one `ARC-ECOSYSTEM-001`'s ownership table gives Nomos for
/// borrowed knowledge.
///
/// `effort` is the eighth, added by `OD-CONTRACTS-004` for a separate requirement,
/// `MODEL-ROUTE-001`'s "agent task classes shall be able to reference a
/// `ModelExecutionProfile`": `TaskEnvelope` is this workspace's one existing
/// `Serialize`/`Deserialize` type for "what an agent-assisted operation is handed," so it
/// is the wire-crossing shape a peer executor would need to see the requested effort in,
/// not a value a caller could instead thread through only as an in-process function
/// argument. Reusing [`EffortLevel`] alone, not the fuller
/// `nomos_model_package::ModelExecutionProfile` (which also carries a `ModelSelector`):
/// `OD-PACKAGE-011` already found routing/selection premature while exactly one real
/// executor exists, so this record's own reader consumes only the one field that maps
/// onto something real today.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskEnvelope
{
    pub goal: String,
    pub scope: Territory,
    pub knowledge_context: Vec<KnowledgeReferenceId>,
    pub applicable_rules: Vec<RuleId>,
    pub prohibited_changes: Territory,
    pub available_tools: Vec<CapabilityId>,
    pub expected_output_schema: SchemaId,
    pub effort: EffortLevel,
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_An_Envelope_Carries_Exactly_What_It_Was_Given()
    {
        let envelope = TaskEnvelope {
            goal: "close AGT-001's gap".to_owned(),
            scope: Territory::Of_Files(["crates/agent/nomos-agent-contracts"]),
            knowledge_context: vec![KnowledgeReferenceId::New("kwb:decision:42")],
            applicable_rules: vec![RuleId::New("check-naming-convention")],
            prohibited_changes: Territory::Of_Files(["work/ledger.json"]),
            available_tools: vec![CapabilityId::New("nomos.cap.example.for_this_test_only")],
            expected_output_schema: SchemaId::New("nomos.agent.work_result.v1"),
            effort: EffortLevel::High,
        };

        assert_eq!(envelope.goal, "close AGT-001's gap");
        assert_eq!(envelope.scope.paths, ["crates/agent/nomos-agent-contracts"]);
        assert_eq!(envelope.knowledge_context.len(), 1);
        assert_eq!(envelope.applicable_rules, [RuleId::New("check-naming-convention")]);
        assert_eq!(envelope.prohibited_changes.paths, ["work/ledger.json"]);
        assert_eq!(envelope.available_tools.len(), 1);
        assert_eq!(envelope.expected_output_schema, SchemaId::New("nomos.agent.work_result.v1"));
        assert_eq!(envelope.effort, EffortLevel::High);
    }
}
