//! Data-like names stay lower snake where the syntax fact exposes them.
//!
//! code-standards' `data-names-stay-lower-snake` also covers locals, parameters, enum
//! variant fields and package names. The current `nomos.cap.syntax.items` payload exposes
//! module declarations and named struct fields, so this rule judges that precise subset and
//! leaves the rest for a richer provider.

use crate::SourceFile;
use crate::checks::finding_shape::{Finding_Shape, Member_Finding};
use crate::checks::naming::Resolve_Case;
use nomos_analysis::FactReader;
use nomos_cap_naming_policy::Case;
use nomos_cap_syntax::{PayloadItem, Struct_Fields, SyntaxPayload};
use nomos_contracts::{Finding, RuleId};

/// This rule's own identifier, matching the code-standards rule id.
pub const DATA_NAMES_STAY_LOWER_SNAKE: &str = "data-names-stay-lower-snake";

const MODULE: &str = "Module";
const STRUCT: &str = "Struct";

/// Judges module declarations and named struct fields against lower snake case — a
/// repository's own `nomos.cap.naming.policy` when it declares `module`/`field`, this
/// rule's own prior default otherwise.
#[must_use]
pub fn Check_Data_Names_Stay_Lower_Snake(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let module_case = Resolve_Case(facts, None, "module", Case::LowerSnake);
    let field_case = Resolve_Case(facts, None, "field", Case::LowerSnake);
    let mut findings = Vec::new();

    for source in sources
    {
        match super::reading::Payload_Of(source, facts)
        {
            Ok(payload) =>
            {
                let violations = Violations_In(&payload, &source.path, module_case, field_case);
                findings.extend(violations);
            }
            Err(finding) => findings.push(Unread_As_This_Rule(finding)),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Violations_In(payload: &SyntaxPayload, path: &str, module_case: Case, field_case: Case) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for item in &payload.items
    {
        if item.kind == MODULE && !module_case.Is_The_Shape_Of(Unescaped_Name(item.Own_Name()))
        {
            let finding = Violation_Finding(path, item, item.Own_Name());
            findings.push(finding);
        }

        if item.kind == STRUCT
        {
            let field_violations = Field_Violations_In(path, item, field_case);
            findings.extend(field_violations);
        }
    }

    return findings;
}

fn Field_Violations_In(path: &str, item: &PayloadItem, field_case: Case) -> Vec<Finding>
{
    let Some(fields) = Struct_Fields(&item.shape)
    else
    {
        return Vec::new();
    };

    return fields
        .iter()
        .filter(|(name, _type_name)| return !field_case.Is_The_Shape_Of(Unescaped_Name(name)))
        .map(|(name, _type_name)| return Violation_Finding(path, item, name))
        .collect();
}

fn Unread_As_This_Rule(mut finding: Finding) -> Finding
{
    finding.rule = RuleId::New(DATA_NAMES_STAY_LOWER_SNAKE);
    finding.summary = finding
        .summary
        .replace("this file's naming could not be judged", "this file's data names could not be judged");
    return finding;
}

/// The name a raw identifier escapes.
///
/// `r#` is Rust's escape for spelling a keyword as an identifier, not part of the name it
/// escapes: `mod r#ref;` declares a module called `ref`, and `r#type: String` a field called
/// `type`. Both are already lower snake, and both were reported as not being so, because the
/// `#` in the escape is not a lower-snake character. The escape is stripped before the case
/// is judged and not before the finding is written, so a name that really does violate the
/// rule is still quoted back in the spelling its source carries.
///
/// `checks::facade` already strips the same prefix at two of its own call sites, inline. A
/// third site is the point at which this stops being a coincidence, so it is named here
/// rather than copied a third time unremarked -- but it is left in place, because folding
/// three inline `strip_prefix`es into shared plumbing is the deduplication `OD-RULES-014`
/// measured before it moved anything, and this item is not that measurement.
fn Unescaped_Name(name: &str) -> &str
{
    return name.strip_prefix("r#").unwrap_or(name);
}

/// One data name that is not lower snake case.
fn Violation_Finding(path: &str, item: &PayloadItem, name: &str) -> Finding
{
    return Member_Finding(
        Finding_Shape { rule: DATA_NAMES_STAY_LOWER_SNAKE, path, summary: format!("`{name}` is a data name that is not lower snake case") },
        item,
        name,
    );
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Violations_In_Should_Report_A_Module_Name_That_Is_Not_Lower_Snake()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tModule\tPrivate\tBadModule\t.\t.\n");

        let findings = Violations_In(&payload, "src/lib.rs", Case::LowerSnake, Case::LowerSnake);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "BadModule");
    }

    #[test]
    fn Test_Violations_In_Should_Report_A_Struct_Field_That_Is_Not_Lower_Snake()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tStruct\tPublic\tConfig\t.\t+fields\\nBadField\\tString\\nworker_count\\tusize\n",
        );

        let findings = Violations_In(&payload, "src/lib.rs", Case::LowerSnake, Case::LowerSnake);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "BadField");
    }

    #[test]
    fn Test_Violations_In_Should_Accept_Lower_Snake_Module_And_Field_Names()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tModule\tPrivate\tread_model\t.\t.\n\
             item\t1\tStruct\tPublic\tConfig\t.\t+fields\\nworker_count\\tusize\n",
        );

        let findings = Violations_In(&payload, "src/lib.rs", Case::LowerSnake, Case::LowerSnake);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// `mod r#ref;` declares a module called `ref`, which is lower snake. The `r#` is
    /// Rust escaping a keyword, and judging it as part of the name reported two module
    /// declarations in this workspace that nothing could be renamed to satisfy.
    #[test]
    fn Test_Violations_In_Should_Accept_A_Module_Named_By_A_Raw_Identifier()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tModule\tPrivate\tr#ref\t.\t.\n");

        let findings = Violations_In(&payload, "src/lib.rs", Case::LowerSnake, Case::LowerSnake);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// The escape is stripped to judge the case and not to write the finding, so a raw
    /// identifier that really is not lower snake is still reported, in its own spelling.
    #[test]
    fn Test_Violations_In_Should_Still_Report_A_Raw_Identifier_That_Is_Not_Lower_Snake()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tModule\tPrivate\tr#BadRef\t.\t.\n");

        let findings = Violations_In(&payload, "src/lib.rs", Case::LowerSnake, Case::LowerSnake);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "r#BadRef");
    }

    #[test]
    fn Test_Violations_In_Should_Ignore_Tuple_Structs()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPublic\tConfig\t.\t.\n");

        let findings = Violations_In(&payload, "src/lib.rs", Case::LowerSnake, Case::LowerSnake);

        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Payload_From_Text(text: &str) -> SyntaxPayload
    {
        return nomos_cap_syntax::Parse_Payload(text.as_bytes())
            .expect("this fixture payload is well formed");
    }
}
