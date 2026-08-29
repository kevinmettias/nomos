//! What materializing the `dependency.policy` fact produced.

use nomos_contracts::Finding;
use nomos_rules::SourceFile;

/// What materializing the `dependency.policy` fact produced: the sources a rule can judge
/// it under, and any finding the materialization itself already raised -- the identical
/// shape [`super::LintMaterialization`] already has for the identical reason.
pub struct PolicyMaterialization
{
    pub sources: Vec<SourceFile>,
    pub findings: Vec<Finding>,
}
