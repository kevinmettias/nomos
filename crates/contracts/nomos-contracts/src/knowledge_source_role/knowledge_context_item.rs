//! `AGT-010`: "Every knowledge item included in a `TaskEnvelope`, `PrepareChangeContext`,
//! `ContextualizedFinding`, composite agent-guidance projection, or agent-facing
//! explanation shall carry a `KnowledgeSourceRole`, source identity/version, authority
//! scope, applicable snapshot or environment, freshness, provenance, and permitted-use
//! classification. Missing role or authority metadata shall be represented as unresolved
//! and shall not be treated as normative."
//!
//! A citation, not the referenced content -- `provenance` and every identity field here
//! name what an external knowledge system said and how, the same bounded-projection
//! role [`KnowledgeReferenceId`] already carries, never the claim's substance itself.
//! `role` and `authority_scope` are `Option` specifically so a missing one can be
//! represented rather than defaulted; [`KnowledgeContextItem::Is_Unresolved`] is the
//! corpus's own "shall not be treated as normative" test.

use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use super::KnowledgeSourceRole;
use crate::{BuildVariantId, KnowledgeReferenceId, SnapshotId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeContextItem
{
    pub source: Option<KnowledgeReferenceId>,
    pub source_version: Option<String>,
    pub role: Option<KnowledgeSourceRole>,
    pub authority_scope: Option<String>,
    pub applicable_snapshot: Option<SnapshotId>,
    pub applicable_build_variant: Option<BuildVariantId>,
    pub freshness: Option<String>,
    pub provenance: String,
    pub permitted_use: Option<String>,
    pub contradiction_links: Vec<KnowledgeReferenceId>,
}

impl KnowledgeContextItem
{
    /// `AGT-010`: "Missing role or authority metadata shall be represented as
    /// unresolved and shall not be treated as normative."
    #[must_use]
    pub const fn Is_Unresolved(&self) -> bool
    {
        return self.role.is_none() || self.authority_scope.is_none();
    }
}

#[cfg(test)]
mod tests
{
    use alloc::borrow::ToOwned;
    use alloc::vec;
    use super::*;

    #[test]
    fn Test_Is_Unresolved_Should_Be_True_When_Role_Or_Authority_Scope_Is_Missing()
    {
        let missing_role = Knowledge_Item(None, Some("repository"));
        let missing_authority = Knowledge_Item(Some(KnowledgeSourceRole::CodeOrTestEvidence), None);
        let resolved = Knowledge_Item(Some(KnowledgeSourceRole::CodeOrTestEvidence), Some("repository"));

        assert!(missing_role.Is_Unresolved(), "a missing role is unresolved");
        assert!(missing_authority.Is_Unresolved(), "a missing authority scope is unresolved");
        assert!(!resolved.Is_Unresolved(), "both present is what resolved means");
    }

    /// One knowledge item, carrying everything but the two fields this test varies.
    fn Knowledge_Item(role: Option<KnowledgeSourceRole>, authority_scope: Option<&str>) -> KnowledgeContextItem
    {
        return KnowledgeContextItem {
            source: None,
            source_version: None,
            role,
            authority_scope: authority_scope.map(ToOwned::to_owned),
            applicable_snapshot: None,
            applicable_build_variant: None,
            freshness: None,
            provenance: "test".to_owned(),
            permitted_use: None,
            contradiction_links: vec![],
        };
    }
}
