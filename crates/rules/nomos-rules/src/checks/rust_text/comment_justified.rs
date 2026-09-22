//! Two rules whose justification is any adjacent comment: no marker is required.
//!
//! Split out of [`super`], which states the family's shared reasoning.
//! `every-allow-carries-a-justification` and `inline-always-requires-justification` differ only
//! in the attribute they match and both accept a plain non-empty comment, unlike the panic and
//! smart-pointer rules' `panic:`/`smart-pointer: allow:` keywords.

use crate::checks::Is_Test_Or_Example_Source;
use crate::{RUST_LANGUAGE, SourceFile};
use nomos_analysis::FactReader;
use nomos_contracts::Finding;

use super::comment_block::{Is_A_Non_Empty_Comment, Has_A_Previous_Comment_Block};
use super::{
    ConstructDetector, Detector, Is_Own_Implementation_File, JustificationDetector, Message, Rule,
    Unjustified_Construct_Findings_In, EVERY_ALLOW_CARRIES_A_JUSTIFICATION, INLINE_ALWAYS_JUSTIFICATION,
};

/// Reports `#[allow(...)]`/`#![allow(...)]` attributes with no adjacent explanatory comment.
///
/// A test or example source is not judged, the same exemption
/// [`Check_Unwrap_Expect_Discipline`] above already reads. A test suppresses a lint to
/// construct the shape it is testing -- a deliberately wrong call, an unused binding held to
/// prove a drop -- and the justification the rule asks for is the test's own name. The
/// exemption is deliberately not extended to the sibling rules in this file: `unsafe` and
/// `#[inline(always)]` mean the same thing wherever they are written, and
/// `a-disabled-test-states-why` would be deleted outright by it, since a disabled test is in
/// a test source by construction.
#[must_use]
pub fn Check_Every_Allow_Carries_A_Justification(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let declared = crate::checks::Resolve_Declared_Fixture_Locations(facts);
    let mut findings = Vec::new();

    for source in sources
    {
        let is_judged_rust_source = source.Is_Written_In(RUST_LANGUAGE)
            && !Is_Test_Or_Example_Source(source, &declared)
            && !Is_Own_Implementation_File(source);
        if is_judged_rust_source
        {
            findings.extend(Allow_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Allow_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    return Unjustified_Construct_Findings_In(
        source,
        Rule(EVERY_ALLOW_CARRIES_A_JUSTIFICATION),
        Message("carries an #[allow(...)] with no adjacent comment explaining why"),
        Detector { has_construct: ConstructDetector(Has_Allow_Attribute), has_local_justification: JustificationDetector(Has_Local_Allow_Justification) },
    );
}
/// Reports `#[inline(always)]` attributes with no adjacent explanatory comment.
#[must_use]
pub fn Check_Inline_Always_Justification(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE) && !Is_Own_Implementation_File(source)
        {
            findings.extend(Inline_Always_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Inline_Always_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    return Unjustified_Construct_Findings_In(
        source,
        Rule(INLINE_ALWAYS_JUSTIFICATION),
        Message("carries #[inline(always)] with no adjacent comment explaining why"),
        Detector { has_construct: ConstructDetector(Has_Inline_Always_Attribute), has_local_justification: JustificationDetector(Has_Local_Inline_Always_Justification) },
    );
}
pub(crate) fn Has_Allow_Attribute(code: &str) -> bool
{
    return code.contains("#[allow(") || code.contains("#![allow(");
}

fn Has_Local_Allow_Justification(lines: &[&str], index: usize) -> bool
{
    if lines.get(index).is_some_and(|line| return Is_A_Non_Empty_Comment(line))
    {
        return true;
    }

    return Has_A_Previous_Comment_Block(lines, index, Is_A_Non_Empty_Comment);
}
pub(crate) fn Has_Inline_Always_Attribute(code: &str) -> bool
{
    return code.contains("#[inline(always)]");
}

fn Has_Local_Inline_Always_Justification(lines: &[&str], index: usize) -> bool
{
    if lines.get(index).is_some_and(|line| return Is_A_Non_Empty_Comment(line))
    {
        return true;
    }

    return Has_A_Previous_Comment_Block(lines, index, Is_A_Non_Empty_Comment);
}
