//! Public type declarations should live in the file named for that type.
//!
//! code-standards' `file-name-matches-declared-type` rule is cross-language and includes
//! generated/partial companion-file behavior. The current syntax payload exposes declared
//! items and visibility, so this rule judges public type-like declarations whose file stem
//! is meaningful to compare, while skipping structural Rust entrypoints such as `lib.rs`
//! and `main.rs`.
//!
//! `one-public-type-per-file` is the sibling rule for the same home: the public-surface
//! form counts top-level public type-like declarations. The C# internal/file-scoped split
//! is outside this crate's current syntax inputs, but the Rust/Go/public-surface rule is
//! exact over the payload.

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_cap_syntax::{PayloadItem, SyntaxPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

/// This rule's own identifier, matching the code-standards rule id.
pub const FILE_NAME_MATCHES_DECLARED_TYPE: &str = "file-name-matches-declared-type";
/// This rule's own identifier, matching the code-standards rule id.
pub const ONE_PUBLIC_TYPE_PER_FILE: &str = "one-public-type-per-file";

const ENUM: &str = "Enum";
const INTERFACE: &str = "Interface";
const STRUCT: &str = "Struct";
const TRAIT: &str = "Trait";
const TRAIT_ALIAS: &str = "TraitAlias";
const TYPE_ALIAS: &str = "TypeAlias";
const TYPE_DEFINITION: &str = "TypeDefinition";
const UNION: &str = "Union";

/// Reports public type-like declarations whose file stem does not match the declared type.
#[must_use]
pub fn Check_File_Name_Matches_Declared_Type(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        match super::reading::Payload_Of(source, facts)
        {
            Ok(payload) => findings.extend(Violations_In(&payload, &source.path)),
            Err(finding) => findings.push(Unread_As_This_Rule(finding)),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Reports files with more than one top-level public type-like declaration.
#[must_use]
pub fn Check_One_Public_Type_Per_File(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        match super::reading::Payload_Of(source, facts)
        {
            Ok(payload) => findings.extend(One_Public_Type_Violations_In(&payload, &source.path)),
            Err(finding) => findings.push(Unread_As_One_Public_Type_Rule(finding)),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Violations_In(payload: &SyntaxPayload, path: &str) -> Vec<Finding>
{
    let Some(stem) = Comparable_Stem(path)
    else
    {
        return Vec::new();
    };

    return payload
        .items
        .iter()
        .filter(|item| return item.Is_Public() && Is_Type_Like(item))
        .filter(|item| return To_Snake_Case(item.Own_Name()) != stem)
        .map(|item| return Violation_Finding(path, &stem, item))
        .collect();
}

fn Is_Type_Like(item: &PayloadItem) -> bool
{
    return matches!(
        item.kind.as_str(),
        ENUM | INTERFACE | STRUCT | TRAIT | TRAIT_ALIAS | TYPE_ALIAS | TYPE_DEFINITION | UNION
    );
}

fn One_Public_Type_Violations_In(payload: &SyntaxPayload, path: &str) -> Vec<Finding>
{
    let public_file_home_types: Vec<&PayloadItem> = payload
        .items
        .iter()
        .filter(|item| return item.Is_Public() && Is_Type_Like(item) && Is_Top_Level(item))
        .collect();

    if public_file_home_types.len() <= 1
    {
        return Vec::new();
    }

    return public_file_home_types
        .into_iter()
        .map(|item| return One_Public_Type_Finding(path, item))
        .collect();
}

fn Is_Top_Level(item: &PayloadItem) -> bool
{
    return !item.qualified_name.contains("::");
}

fn Comparable_Stem(path: &str) -> Option<String>
{
    let normalized = path.replace('\\', "/");
    let file_name = normalized.rsplit('/').next().unwrap_or(&normalized);
    let stem = file_name.split('.').next().unwrap_or(file_name);

    if matches!(stem, "lib" | "main" | "mod")
    {
        return None;
    }

    return Some(stem.to_owned());
}

fn To_Snake_Case(name: &str) -> String
{
    let mut snake = String::new();
    let mut previous_was_lower_or_digit = false;

    for character in name.chars()
    {
        if character.is_ascii_uppercase()
        {
            if previous_was_lower_or_digit
            {
                snake.push('_');
            }
            snake.push(character.to_ascii_lowercase());
            previous_was_lower_or_digit = false;
            continue;
        }

        previous_was_lower_or_digit = character.is_ascii_lowercase() || character.is_ascii_digit();
        snake.push(character);
    }

    return snake;
}

fn Violation_Finding(path: &str, stem: &str, item: &PayloadItem) -> Finding
{
    use nomos_model::Content_Digest;

    let type_name = item.Own_Name();
    let expected = To_Snake_Case(type_name);
    let qualified = format!("{path}::{}", item.qualified_name);

    return Finding {
        rule: RuleId::New(FILE_NAME_MATCHES_DECLARED_TYPE),
        subject: SubjectId::From_Digest(Content_Digest(qualified.as_bytes())),
        subject_name: type_name.to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("`{type_name}` is public but {path} has stem `{stem}` instead of `{expected}`"),
        locations: vec![path.to_owned()],
    };
}

fn Unread_As_This_Rule(mut finding: Finding) -> Finding
{
    finding.rule = RuleId::New(FILE_NAME_MATCHES_DECLARED_TYPE);
    finding.summary = finding
        .summary
        .replace("this file's naming could not be judged", "this file's declared public types could not be judged");
    return finding;
}

fn One_Public_Type_Finding(path: &str, item: &PayloadItem) -> Finding
{
    use nomos_model::Content_Digest;

    let type_name = item.Own_Name();
    let qualified = format!("{path}::{}", item.qualified_name);

    return Finding {
        rule: RuleId::New(ONE_PUBLIC_TYPE_PER_FILE),
        subject: SubjectId::From_Digest(Content_Digest(qualified.as_bytes())),
        subject_name: type_name.to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("{path} declares more than one top-level public type; `{type_name}` needs its own file"),
        locations: vec![path.to_owned()],
    };
}

fn Unread_As_One_Public_Type_Rule(mut finding: Finding) -> Finding
{
    finding.rule = RuleId::New(ONE_PUBLIC_TYPE_PER_FILE);
    finding.summary = finding.summary.replace(
        "this file's naming could not be judged",
        "this file's public type count could not be judged",
    );
    return finding;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Violations_In_Should_Report_A_Public_Type_Whose_File_Stem_Does_Not_Match()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPublic\tOrderBook\t.\t.\n");

        let findings = Violations_In(&payload, "src/orders.rs");

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New(FILE_NAME_MATCHES_DECLARED_TYPE));
        assert_eq!(found.subject_name, "OrderBook");
    }

    #[test]
    fn Test_Violations_In_Should_Accept_A_Public_Type_Whose_File_Stem_Matches()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPublic\tOrderBook\t.\t.\n");

        let findings = Violations_In(&payload, "src/order_book.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Violations_In_Should_Ignore_Private_Types()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPrivate\tOrderBook\t.\t.\n");

        let findings = Violations_In(&payload, "src/orders.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Violations_In_Should_Compare_Companion_Files_By_The_First_Stem()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPublic\tOrderBook\t.\t.\n");

        let findings = Violations_In(&payload, "src/order_book.generated.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Violations_In_Should_Ignore_Structural_Entrypoints()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPublic\tOrderBook\t.\t.\n");

        let findings = Violations_In(&payload, "src/lib.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_One_Public_Type_Violations_In_Should_Report_Each_Public_Type_When_Two_Are_Top_Level()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tStruct\tPublic\tOrder\t.\t.\n\
             item\t1\tEnum\tPublic\tOrderKind\t.\t.\n",
        );

        let findings = One_Public_Type_Violations_In(&payload, "src/order.rs");

        assert_eq!(findings.len(), 2, "{findings:?}");
        assert!(findings.iter().all(|finding| return finding.rule == RuleId::New(ONE_PUBLIC_TYPE_PER_FILE)));
    }

    #[test]
    fn Test_One_Public_Type_Violations_In_Should_Accept_One_Public_Type_With_Private_Helpers()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tStruct\tPublic\tOrder\t.\t.\n\
             item\t1\tEnum\tPrivate\tOrderKind\t.\t.\n",
        );

        let findings = One_Public_Type_Violations_In(&payload, "src/order.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_One_Public_Type_Violations_In_Should_Ignore_Nested_Public_Types()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tStruct\tPublic\tOrder\t.\t.\n\
             item\t1\tStruct\tPublic\tOrder::Builder\t.\t.\n",
        );

        let findings = One_Public_Type_Violations_In(&payload, "src/order.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Payload_From_Text(text: &str) -> SyntaxPayload
    {
        return nomos_cap_syntax::Parse_Payload(text.as_bytes())
            .expect("this fixture payload is well formed");
    }
}
