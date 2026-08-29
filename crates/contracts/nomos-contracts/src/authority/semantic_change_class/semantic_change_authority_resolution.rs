//! `AGT-018`: "Organization and repository policy shall resolve every applicable class
//! to an allowed authority level, required approver role, evidence obligations,
//! permitted mutation mode, rollback boundary, validation scope, and
//! autonomous-execution disposition. ... Nomos shall expose a user-visible projection of
//! the resolved decision that states the classified effects, governing policy, trust
//! evidence where used, required validation, approver roles, and the reason the
//! operation is autonomous, preview-only, proposal-only, review-required, or
//! prohibited."
//!
//! Same glossary entry (5.6): "An immutable, policy-resolved authorization record for an
//! agent-originated or agent-assisted mutation." `governing ChangeIntent` is
//! deliberately not a field: `ChangeIntent` itself is not built anywhere in this
//! workspace, so naming it here would type a field against an identity that does not
//! exist yet rather than reuse one.

use serde::{Deserialize, Serialize};

use super::SemanticChangeClass;
use crate::{AuthorityClass, BuildVariantId, MutationClass};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticChangeAuthorityResolution
{
    pub predicted_class: SemanticChangeClass,
    pub observed_class: Option<SemanticChangeClass>,
    pub required_authority: AuthorityClass,
    pub approver_role: Option<String>,
    pub permitted_mutation_mode: MutationClass,
    pub autonomous_execution: bool,
    pub evidence_obligations: Vec<String>,
    pub rollback_boundary: Option<String>,
    pub unresolved_effects: Vec<String>,
    pub scope_expansion: bool,
    pub decision_rationale: String,
    pub affected_build_variants: Vec<BuildVariantId>,
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Resolution_Carries_Exactly_What_It_Was_Given()
    {
        let resolution = SemanticChangeAuthorityResolution {
            predicted_class: SemanticChangeClass::LocalImplementation,
            observed_class: None,
            required_authority: AuthorityClass::Mutate,
            approver_role: None,
            permitted_mutation_mode: MutationClass::Apply,
            autonomous_execution: true,
            evidence_obligations: vec![],
            rollback_boundary: None,
            unresolved_effects: vec![],
            scope_expansion: false,
            decision_rationale: "no cross-boundary effect predicted".to_owned(),
            affected_build_variants: vec![],
        };

        assert_eq!(resolution.predicted_class, SemanticChangeClass::LocalImplementation);
        assert_eq!(resolution.required_authority, AuthorityClass::Mutate);
        assert!(resolution.autonomous_execution);
    }
}
