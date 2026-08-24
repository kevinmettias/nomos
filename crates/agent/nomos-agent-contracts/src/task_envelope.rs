//! What an agent-assisted operation is handed before it starts.

use nomos_contracts::{CapabilityId, KnowledgeReferenceId, RuleId, SchemaId};
use nomos_ledger::Territory;
use serde::{Deserialize, Serialize};

/// `AGT-001`: "Before agent work, Nomos shall produce a typed task envelope containing
/// goal, scope, authoritative knowledge context, applicable rules, prohibited changes,
/// available tools, and expected output schema."
///
/// Seven fields, each a direct reuse of an existing type: `scope` and
/// `prohibited_changes` both reuse [`Territory`] -- the same shape, at two different
/// membership senses (what may be touched, what may not). `knowledge_context` carries
/// [`KnowledgeReferenceId`] citations, never a `KnowledgeWorkbench` claim's own content
/// -- the bounded-projection role that identity already plays, and the one
/// `ARC-ECOSYSTEM-001`'s ownership table gives Nomos for borrowed knowledge.
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
        };

        assert_eq!(envelope.goal, "close AGT-001's gap");
        assert_eq!(envelope.scope.paths, ["crates/agent/nomos-agent-contracts"]);
        assert_eq!(envelope.knowledge_context.len(), 1);
        assert_eq!(envelope.applicable_rules, [RuleId::New("check-naming-convention")]);
        assert_eq!(envelope.prohibited_changes.paths, ["work/ledger.json"]);
        assert_eq!(envelope.available_tools.len(), 1);
        assert_eq!(envelope.expected_output_schema, SchemaId::New("nomos.agent.work_result.v1"));
    }
}
