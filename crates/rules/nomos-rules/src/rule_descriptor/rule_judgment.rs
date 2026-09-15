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
pub type RuleJudgment = fn(&[SourceFile], &mut dyn FactReader) -> Vec<Finding>;
