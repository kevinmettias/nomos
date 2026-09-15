//! Materializing `review.finding` facts, which produces nothing on its own today.

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

/// Produces the `nomos.cap.review.finding` sources [`nomos_rules::Check_Review_Findings`]
/// can judge -- today, always none.
///
/// Unlike [`super::Materialize_Dependencies`], [`super::Materialize_Lint`] and
/// [`super::Materialize_Policy`], this capability is not "run one subprocess over the whole
/// workspace and materialize what it finds": `nomos-connector-coderabbit`'s one provider
/// answers about one already-identified external review comment (a repository and a comment
/// id), and nothing in an ordinary `nomos check` run over already-walked source names either
/// -- there is no field on [`crate::RunContext`], [`nomos_analysis::Context`], or any
/// [`SourceFile`] this crate's own composition can read one from, and inventing a discovery
/// mechanism (which pull request, which comment) is exactly the kind of address
/// `ARC-CONNECTOR-001` deliberately declines to decide in advance of a real caller needing
/// one. So this materialization step is honest about producing nothing today:
/// `Check_Review_Findings` is composed, selectable, and will judge real sources the moment a
/// future composition root supplies a repository and comment id to fetch -- which is
/// deliberately not invented here, ahead of a real caller needing one.
#[must_use]
pub fn Materialize_Review() -> ReviewMaterialization
{
    return ReviewMaterialization { sources: Vec::new(), findings: Vec::new() };
}
