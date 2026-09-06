//! What materializing `review.finding` facts produced.

use nomos_contracts::Finding;
use nomos_rules::SourceFile;

/// What materializing `review.finding` facts produced: the sources a rule can judge them
/// under, and any finding the materialization itself already raised -- the identical shape
/// [`super::LintMaterialization`] already has for the identical reason.
pub struct ReviewMaterialization
{
    pub sources: Vec<SourceFile>,
    pub findings: Vec<Finding>,
}
