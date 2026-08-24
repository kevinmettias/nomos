//! The shared envelope a routing-configuration validation diagnostic carries.

use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-028`: "Validation results shall be machine-readable, stable for
/// identical pinned inputs, and projected consistently through configuration
/// editors, CLI/API/MCP validation, workflow publication, run preview, and audit
/// views. Each diagnostic shall state severity, blocking disposition, affected
/// operations, evidence, winning or missing candidates, and a concrete remediation
/// such as remove, narrow, retarget, install, authorize, weaken explicitly, or add a
/// satisfiable fallback."
///
/// Six fields, directly naming the "each diagnostic shall state A, B, C, D, E, F"
/// clause -- no proper noun is given for this type, the same naming pattern
/// [`crate::EffortLevel`] itself used (the corpus closes the value/field list, this
/// workspace supplies the name). `severity` stays a raw `String`: no closed value set
/// is given anywhere in this requirement. `remediation` stays a raw `String` rather
/// than a closed enum: "such as" marks the remediation list as illustrative, not
/// exhaustive, unlike every other closed list built this session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationDiagnostic
{
    pub severity: String,
    pub blocking: bool,
    pub affected_operations: Vec<String>,
    pub evidence: String,
    pub candidates: CandidateOutcome,
    pub remediation: String,
}

/// `MODEL-ROUTE-028`: "winning or missing candidates."
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateOutcome
{
    Winning(Vec<String>),
    Missing(Vec<String>),
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Diagnostic_Carries_Exactly_What_It_Was_Given()
    {
        let diagnostic = ValidationDiagnostic {
            severity: "blocking".to_owned(),
            blocking: true,
            affected_operations: vec!["judge-finding".to_owned()],
            evidence: "no candidate satisfies the pinned effort constraint".to_owned(),
            candidates: CandidateOutcome::Missing(vec!["gpt-5-mini".to_owned()]),
            remediation: "weaken the effort constraint explicitly".to_owned(),
        };

        assert!(diagnostic.blocking);
        assert_eq!(diagnostic.candidates, CandidateOutcome::Missing(vec!["gpt-5-mini".to_owned()]));
    }
}
