//! The one shape every rule's own judgment is called through.
//!
//! Its own file rather than a second type beside [`super::RuleDescriptor`]: the one-public-type
//! rule wants one file-home type per file, and the callable is what keeps [`super::DESCRIPTORS`]
//! a `const` table rather than a list of identifiers needing a second table to carry the code.

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_contracts::Finding;

/// A rule's own judgment, in the one shape every rule can be called through.
///
/// A plain `fn` pointer rather than a trait object or a closure, because [`super::DESCRIPTORS`] is
/// a `const` and a `fn` pointer is the only callable a `const` can hold. That is what makes
/// the table below the whole declaration of a rule rather than half of one, and it is what
/// lets `nomos_check_orchestration` derive its run from this list instead of writing a
/// second copy of it by hand -- `OD-RULES-027`.
///
/// A named type with a method rather than a bare `fn` alias, and the pointer stays the only
/// thing it holds. The alias named the decision and stopped there: the two parameters stayed
/// positions a caller could transpose, and there was nowhere to state what a rule's judgment
/// may do. [`Self::Judges`] is that somewhere, and it is the one place a run calls a rule
/// through -- `nomos_check_orchestration`'s `run_context` names it, and nothing else may.
///
/// Both parameters are taken by every rule and read by most. A rule that judges only source
/// text is widened here with a closure that ignores the reader, and the two rules whose
/// whole subject arrives through the reader are widened with one that ignores the sources;
/// neither wrapper decides anything, which is why they are spelled inline in the table
/// rather than given names of their own.
///
/// The sources a rule is handed are not always the walked ones. Six rules read a capability
/// family's own materialized slice instead, and which six is `nomos_check_orchestration`'s
/// to say, not this table's: a capability slice is an orchestration concept, and a
/// descriptor naming one would be a lower band describing an upper band's shape.
/// `OD-RULES-027` decided that split and why the mapping that remains there is a
/// declaration rather than the demand planner `OD-RULES-009` declines.
#[derive(Clone, Copy, Debug)]
pub struct RuleJudgment(fn(&[SourceFile], &mut dyn FactReader) -> Vec<Finding>);

impl RuleJudgment
{
    /// A rule's judgment, carried as the one pointer shape a `const` table can hold.
    ///
    /// `const` so that [`super::DESCRIPTORS`] stays a table rather than becoming a run-time
    /// build step: a constructor that allocated would be this same declaration with the
    /// property that made it useful taken away -- `OD-RULES-027`.
    #[must_use]
    pub const fn New(judge: fn(&[SourceFile], &mut dyn FactReader) -> Vec<Finding>) -> Self
    {
        return Self(judge);
    }

    /// Runs this rule's judgment over `sources`, reading whatever facts it needs through `reader`.
    ///
    /// The rule's only call path. Both arguments are handed to every rule whether it reads them
    /// or not, which is what lets one signature serve a fact-free rule, a source-free rule and
    /// a rule that reads both -- see the module doc for why that widening has no decision in it.
    #[must_use]
    pub fn Judges(&self, sources: &[SourceFile], reader: &mut dyn FactReader) -> Vec<Finding>
    {
        return (self.0)(sources, reader);
    }
}
