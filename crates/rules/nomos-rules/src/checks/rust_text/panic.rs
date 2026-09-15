//! `panics-are-justified-documented-and-validated`: a panic primitive with no local note.
//!
//! Split out of [`super`], which states the family's shared reasoning.

use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::Finding;

use super::comment_block::{Comment_Text_Of, Previous_Comment_Block_Has};
use super::{
    ConstructDetector, Detector, JustificationDetector, Message, Rule, Unjustified_Construct_Findings_In,
    PANICS_ARE_JUSTIFIED_DOCUMENTED_AND_VALIDATED,
};

/// Reports explicit panic primitives that do not carry a local panic/invariant note.
#[must_use]
pub fn Check_Panics_Are_Justified_Documented_And_Validated(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE)
        {
            findings.extend(Panic_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Panic_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    return Unjustified_Construct_Findings_In(
        source,
        Rule(PANICS_ARE_JUSTIFIED_DOCUMENTED_AND_VALIDATED),
        Message("uses a panic primitive without a local panic or invariant note"),
        Detector { has_construct: ConstructDetector(Has_Panic_Primitive), has_local_justification: JustificationDetector(Has_Local_Panic_Justification) },
    );
}

fn Has_Panic_Primitive(code: &str) -> bool
{
    return code.contains("panic!(")
        || code.contains("todo!(")
        || code.contains("unimplemented!(")
        || code.contains("unreachable!(");
}

fn Has_Local_Panic_Justification(lines: &[&str], index: usize) -> bool
{
    if lines.get(index).is_some_and(|line| return Comment_Has_Panic_Reason(line))
    {
        return true;
    }

    return Previous_Comment_Block_Has(lines, index, Comment_Has_Panic_Reason);
}

fn Comment_Has_Panic_Reason(line: &str) -> bool
{
    let Some(comment) = Comment_Text_Of(line)
    else
    {
        return false;
    };

    let lower = comment.to_ascii_lowercase();
    return lower.contains("panic:")
        || lower.contains("panics:")
        || lower.contains("invariant:")
        || lower.contains("# panics");
}
