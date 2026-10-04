//! Go type names use the language's own exported/unexported camel-case split.
//!
//! code-standards' `types-use-upper-camel-case-lower-camel-case` is a Go naming rule. The
//! syntax payload already carries Go type declarations, their visibility, and their names,
//! so this check is exact for source files recognized as Go by path.

use crate::checks::finding_shape::{Finding_Shape, Own_Name_Finding};
use crate::checks::naming::{Case_Judged_Against, Resolve_Read};
use crate::checks::optional_reads::{GO_EXPORTED_TYPE, GO_UNEXPORTED_TYPE};
use crate::SourceFile;
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
    let (Some(exported_case), Some(unexported_case)) =
        (Resolve_Read(facts, &GO_EXPORTED_TYPE), Resolve_Read(facts, &GO_UNEXPORTED_TYPE))
    else
    {
        return Vec::new();
    };

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
        if !crate::checks::populations::GO_TYPE_NAMES_POPULATION.Holds(source)
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
        .map(|item| return (item, Go_Type_Case(item, exported_case, unexported_case)))
        .filter(|(item, case)| return !case.Is_The_Shape_Of(item.Own_Name()))
        .map(|(item, case)| return Violation_Finding(path, item, case))
        .collect();
}

fn Is_Go_Type_Like(item: &PayloadItem) -> bool
{
    return matches!(item.kind.as_str(), INTERFACE | STRUCT | TYPE_ALIAS | TYPE_DEFINITION);
}

/// The case `item`'s visibility selects: the one its name is judged against, and the one its
/// finding names.
fn Go_Type_Case(item: &PayloadItem, exported_case: Case, unexported_case: Case) -> Case
{
    if item.Is_Public()
    {
        return exported_case;
    }

    return unexported_case;
}

/// One Go type whose name is not `case`, the case its visibility selected.
///
/// The finding names the case the type was judged against. It once named `UpperCamelCase` or
/// `lowerCamelCase` by visibility alone, so a repository that declared another case for a side
/// was told its types missed a case nobody had held them to.
fn Violation_Finding(path: &str, item: &PayloadItem, case: Case) -> Finding
{
    let name = item.Own_Name();
    let case = Case_Judged_Against(case);

    return Own_Name_Finding(
        Finding_Shape { rule: TYPES_USE_UPPER_CAMEL_CASE_LOWER_CAMEL_CASE, path, summary: format!("Go type `{name}` is not {case}") },
        item,
    );
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

    /// `OrderBook` is upper camel case, and judged against lower snake case it is reported:
    /// the finding has to say lower snake case, because saying the type is not upper camel case
    /// would be false. Each side of visibility names its own case.
    #[test]
    fn Test_Violations_In_Should_Name_The_Case_Each_Side_Of_Visibility_Was_Judged_Against()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tStruct\tPublic\tOrderBook\t.\t.\n\
             item\t1\tTypeDefinition\tPrivate\torderState\t.\t.\n",
        );

        let findings = Violations_In(&payload, "orders.go", Case::LowerSnake, Case::ScreamingSnake);

        let summaries: Vec<&str> = findings.iter().map(|finding| return finding.summary.as_str()).collect();
        assert_eq!(summaries, vec!["Go type `OrderBook` is not lower-snake case", "Go type `orderState` is not screaming-snake case"]);
    }

    fn Payload_From_Text(text: &str) -> SyntaxPayload
    {
        return nomos_cap_syntax::Parse_Payload(text.as_bytes())
            .expect("this fixture payload is well formed");
    }
}
