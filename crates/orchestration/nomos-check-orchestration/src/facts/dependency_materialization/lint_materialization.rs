//! What materializing `lint.diagnostics` facts produced.

use nomos_contracts::Finding;
use nomos_rules::SourceFile;

/// What materializing `lint.diagnostics` facts produced: the sources a rule can judge them
/// under, and any finding the materialization itself already raised -- the identical shape
/// [`super::DependencyMaterialization`] already has for the identical reason.
pub struct LintMaterialization
{
    pub sources: Vec<SourceFile>,
    pub findings: Vec<Finding>,
}
