//! Go type names use the language's own exported/unexported camel-case split.
//!
//! code-standards' `types-use-upper-camel-case-lower-camel-case` is a Go naming rule. The
//! syntax payload already carries Go type declarations, their visibility, and their names,
//! so this check is exact for source files recognized as Go by path.

use crate::checks::finding_shape::Own_Name_Finding;
use crate::checks::naming::Resolve_Case;
use crate::{GO_LANGUAGE, SourceFile};
use nomos_analysis::FactReader;
use nomos_cap_naming_policy::Case;
use nomos_cap_syntax::{PayloadItem, SyntaxPayload};
use nomos_contracts::{Finding, RuleId};

/// This rule's own identifier, matching the code-standards rule id.
pub const TYPES_USE_UPPER_CAMEL_CASE_LOWER_CAMEL_CASE: &str = "types-use-upper-camel-case-lower-camel-case";

const INTERFACE: &str = "Interface";
const STRUCT: &str = "Struct";
const TYPE_ALIAS: &str = "TypeAlias";
const TYPE_DEFINITION: &str = "TypeDefinition";

/// Judges Go type declarations and aliases against exported/unexported camel case — a
/// repository's own `nomos.cap.naming.policy` when it declares `type.exported`/`type.
/// unexported` for `go`, this rule's own prior default otherwise.
#[must_use]
pub fn Check_Go_Type_Names_Use_Camel_Case(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let exported_case = Resolve_Case(facts, Some("go"), "type.exported", Case::UpperCamel);
    let unexported_case = Resolve_Case(facts, Some("go"), "type.unexported", Case::LowerCamel);

    return Judged_Go_Type_Sources(sources, facts, exported_case, unexported_case);
}

fn Judged_Go_Type_Sources(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
    exported_case: Case,
    unexported_case: Case,
) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if !source.Is_Written_In(GO_LANGUAGE)
        {
            continue;
        }

        match super::super::reading::Payload_Of(source, facts)
        {
            Ok(payload) =>
            {
                let violations = Violations_In(&payload, &source.path, exported_case, unexported_case);
                findings.extend(violations);
            }
            Err(finding) => findings.push(Unread_As_This_Rule(finding)),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Violations_In(payload: &SyntaxPayload, path: &str, exported_case: Case, unexported_case: Case) -> Vec<Finding>
{
    return payload
        .items
        .iter()
        .filter(|item| return Is_Go_Type_Like(item))
        .filter(|item| return !Has_Go_Type_Case(item, exported_case, unexported_case))
        .map(|item| return Violation_Finding(path, item))
        .collect();
}

fn Is_Go_Type_Like(item: &PayloadItem) -> bool
{
    return matches!(item.kind.as_str(), INTERFACE | STRUCT | TYPE_ALIAS | TYPE_DEFINITION);
}

fn Has_Go_Type_Case(item: &PayloadItem, exported_case: Case, unexported_case: Case) -> bool
{
    let name = item.Own_Name();

    if item.Is_Public()
    {
        return exported_case.Is_The_Shape_Of(name);
    }

    return unexported_case.Is_The_Shape_Of(name);
}

/// One Go type whose name does not carry the case its visibility requires.
fn Violation_Finding(path: &str, item: &PayloadItem) -> Finding
{
    let name = item.Own_Name();
    let expected = Expected_Case_Label(item);

    return Own_Name_Finding(
        TYPES_USE_UPPER_CAMEL_CASE_LOWER_CAMEL_CASE,
        path,
        item,
        format!("Go type `{name}` is not {expected}"),
    );
}

fn Expected_Case_Label(item: &PayloadItem) -> &'static str
{
    if item.Is_Public()
    {
        return "UpperCamelCase";
    }

    return "lowerCamelCase";
}

fn Unread_As_This_Rule(mut finding: Finding) -> Finding
{
    finding.rule = RuleId::New(TYPES_USE_UPPER_CAMEL_CASE_LOWER_CAMEL_CASE);
    finding.summary = finding
        .summary
        .replace("this file's naming could not be judged", "this Go file's type names could not be judged");
    return finding;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Violations_In_Should_Report_Exported_Go_Types_That_Are_Not_Upper_Camel()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPublic\torder_book\t.\t.\n");

        let findings = Violations_In(&payload, "orders.go", Case::UpperCamel, Case::LowerCamel);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "order_book");
    }

    #[test]
    fn Test_Violations_In_Should_Report_Unexported_Go_Types_That_Are_Not_Lower_Camel()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tInterface\tPrivate\treader_handle\t.\t.\n");

        let findings = Violations_In(&payload, "reader.go", Case::UpperCamel, Case::LowerCamel);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "reader_handle");
    }

    #[test]
    fn Test_Violations_In_Should_Accept_Exported_And_Unexported_Go_Type_Names()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tStruct\tPublic\tOrderBook\t.\t.\n\
             item\t1\tTypeDefinition\tPrivate\torderState\t.\t.\n",
        );

        let findings = Violations_In(&payload, "orders.go", Case::UpperCamel, Case::LowerCamel);

        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Payload_From_Text(text: &str) -> SyntaxPayload
    {
        return nomos_cap_syntax::Parse_Payload(text.as_bytes())
            .expect("this fixture payload is well formed");
    }
}
