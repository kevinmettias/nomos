//! `shared-interior-mutability-says-why`: `Rc`/`Arc` plus `RefCell` with no stated reason.
//!
//! Split out of [`super`], which states the family's shared reasoning.

use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::Finding;

use super::comment_block::{Comment_Text_Of, Has_A_Previous_Comment_Block};
use super::{
    ConstructDetector, Detector, Is_Own_Implementation_File, JustificationDetector, Message, Rule,
    Unjustified_Construct_Findings_In, SHARED_INTERIOR_MUTABILITY_SAYS_WHY,
};

/// Reports shared `Rc`/`Arc` plus `RefCell` constructs that do not say why.
#[must_use]
pub fn Check_Shared_Interior_Mutability_Says_Why(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE) && !Is_Own_Implementation_File(source)
        {
            findings.extend(Shared_Interior_Mutability_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Shared_Interior_Mutability_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    return Unjustified_Construct_Findings_In(
        source,
        Rule(SHARED_INTERIOR_MUTABILITY_SAYS_WHY),
        Message("uses shared interior mutability without `smart-pointer: allow: <reason>`"),
        Detector { has_construct: ConstructDetector(Has_Shared_RefCell_Construct), has_local_justification: JustificationDetector(Has_Local_Smart_Pointer_Reason) },
    );
}

fn Has_Shared_RefCell_Construct(code: &str) -> bool
{
    let compact = code
        .chars()
        .filter(|character| return !character.is_whitespace())
        .collect::<String>();

    return Is_Shared_Type_Containing_RefCell(CompactTypeText(&compact), SmartPointerName("Rc"))
        || Is_Shared_Type_Containing_RefCell(CompactTypeText(&compact), SmartPointerName("Arc"))
        || compact.contains("Rc::new(RefCell::new(")
        || compact.contains("Arc::new(RefCell::new(")
        || compact.contains("Rc::<RefCell<")
        || compact.contains("Arc::<RefCell<");
}

/// `compact` and `smart_pointer` are both `&str`; without a distinct type per position, a
/// call site like `Is_Shared_Type_Containing_RefCell(compact, smart_pointer)` reads as two
/// interchangeable strings and a swap compiles silently.
struct CompactTypeText<'a>(&'a str);

struct SmartPointerName<'a>(&'a str);

fn Is_Shared_Type_Containing_RefCell(
    compact: CompactTypeText<'_>,
    smart_pointer: SmartPointerName<'_>,
) -> bool
{
    let compact = compact.0;
    let pattern = format!("{}<", smart_pointer.0);
    let Some(start) = compact.find(&pattern)
    else
    {
        return false;
    };

    let Some(rest) = compact.get(start.saturating_add(pattern.len())..)
    else
    {
        return false;
    };

    let Some(end) = rest.find('>')
    else
    {
        return false;
    };

    return rest.get(..end).is_some_and(|inner| return inner.contains("RefCell<"));
}

fn Has_Local_Smart_Pointer_Reason(lines: &[&str], index: usize) -> bool
{
    if lines
        .get(index)
        .is_some_and(|line| return Has_A_Smart_Pointer_Reason(line))
    {
        return true;
    }

    return Has_A_Previous_Comment_Block(lines, index, Has_A_Smart_Pointer_Reason);
}

fn Has_A_Smart_Pointer_Reason(line: &str) -> bool
{
    let Some(comment) = Comment_Text_Of(line)
    else
    {
        return false;
    };

    let Some(reason) = comment.split("smart-pointer: allow:").nth(1)
    else
    {
        return false;
    };

    return !reason.trim().is_empty();
}
