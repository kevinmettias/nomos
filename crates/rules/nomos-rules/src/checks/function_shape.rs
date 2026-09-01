//! Function-shape rules that are visible in `nomos.cap.syntax.items`.
//!
//! The current syntax payload carries function arity, including a receiver when one is
//! declared. It does not mark whether a qualified function's first input is a receiver, so
//! this rule reports only cases that are definitely over code-standards' four value
//! parameter cap: top-level/module functions with arity greater than four, and qualified
//! functions/methods with arity greater than five. The latter threshold allows one possible
//! receiver without producing a false positive.

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_cap_syntax::{FUNCTION, Function_Arity, PayloadItem, SyntaxPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

/// The code-standards parameter-count rule id.
pub const PARAMETER_COUNT: &str = "parameter-count";
/// The Go-specific code-standards parameter-count rule id.
pub const GO_HELPERS_PACKAGE_FIVE_INPUTS: &str = "go-helpers-package-five-inputs";

const MAX_VALUE_PARAMETERS: u32 = 4;

/// Reports functions that definitely exceed the four-value-parameter cap.
#[must_use]
pub fn Check_Parameter_Count(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    return Check_Parameter_Count_With(sources, facts, PARAMETER_COUNT, |_| return true);
}

/// Reports Go functions and methods that definitely exceed the four-value-parameter cap.
#[must_use]
pub fn Check_Go_Helpers_Package_Five_Inputs(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    return Check_Parameter_Count_With(
        sources,
        facts,
        GO_HELPERS_PACKAGE_FIVE_INPUTS,
        |source| return Is_Go_File(&source.path),
    );
}

fn Check_Parameter_Count_With(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
    rule: &'static str,
    accepts_source: impl Fn(&SourceFile) -> bool,
) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if !accepts_source(source)
        {
            continue;
        }

        match crate::checks::naming::reading::Payload_Of(source, facts)
        {
            Ok(payload) => findings.extend(Violations_In(rule, &payload, &source.path)),
            Err(finding) => findings.push(Unread_As_Rule(finding, rule)),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Violations_In(rule: &'static str, payload: &SyntaxPayload, path: &str) -> Vec<Finding>
{
    return payload
        .items
        .iter()
        .filter(|item| return item.kind == FUNCTION)
        .filter_map(|item| return Function_Arity(&item.shape).map(|arity| return (item, arity)))
        .filter(|(item, arity)| return Definitely_Too_Many_Value_Parameters(item, *arity))
        .map(|(item, arity)| return Violation_Finding(rule, path, item, arity))
        .collect();
}

fn Is_Go_File(path: &str) -> bool
{
    return std::path::Path::new(path)
        .extension()
        .is_some_and(|extension| return extension.eq_ignore_ascii_case("go"));
}

fn Definitely_Too_Many_Value_Parameters(item: &PayloadItem, arity: u32) -> bool
{
    if item.qualified_name.contains("::")
    {
        return arity > MAX_VALUE_PARAMETERS.saturating_add(1);
    }

    return arity > MAX_VALUE_PARAMETERS;
}

fn Violation_Finding(rule: &'static str, path: &str, item: &PayloadItem, arity: u32) -> Finding
{
    use nomos_model::Content_Digest;

    let qualified = format!("{path}::{}", item.qualified_name);

    return Finding {
        rule: RuleId::New(rule),
        subject: SubjectId::From_Digest(Content_Digest(qualified.as_bytes())),
        subject_name: item.qualified_name.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!(
            "`{}` has arity {arity}, which definitely exceeds the four value parameter cap",
            item.qualified_name
        ),
        locations: vec![path.to_owned()],
    };
}

fn Unread_As_Rule(mut finding: Finding, rule: &'static str) -> Finding
{
    finding.rule = RuleId::New(rule);
    finding.summary = finding
        .summary
        .replace("this file's naming could not be judged", "this file's parameter counts could not be judged");
    return finding;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::checks::test_support::{self, Test_Context, TestOffering};
    use nomos_analysis::{InputDigest, MemoryFactStore, Reader};
    use nomos_capability::ProviderOffer;
    use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity};

    #[test]
    fn Test_Violations_In_Should_Report_A_Free_Function_With_Five_Parameters()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tBuild\t.\t+fn/5\n");

        let findings = Violations_In(PARAMETER_COUNT, &payload, "src/lib.rs");

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(PARAMETER_COUNT));
    }

    #[test]
    fn Test_Violations_In_Should_Accept_A_Free_Function_With_Four_Parameters()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tBuild\t.\t+fn/4\n");

        let findings = Violations_In(PARAMETER_COUNT, &payload, "src/lib.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Violations_In_Should_Not_Report_A_Qualified_Function_At_Receiver_Plus_Four()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tBuilder::Build\t.\t+fn/5\n");

        let findings = Violations_In(PARAMETER_COUNT, &payload, "src/lib.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Violations_In_Should_Report_A_Qualified_Function_With_Six_Parameters()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tBuilder::Build\t.\t+fn/6\n");

        let findings = Violations_In(PARAMETER_COUNT, &payload, "src/lib.rs");

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Go_Helpers_Package_Five_Inputs_Should_Report_Under_The_Go_Rule_Id()
    {
        let source = Source("builder.go", "func Build(a A, b B, c C, d D, e E) {}");
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Syntax_Fact(
            &mut store,
            &source,
            &offer,
            "unexpanded\t0\nitem\t0\tFunction\tPublic\tBuild\t.\t+fn/5\n",
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Go_Helpers_Package_Five_Inputs(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(
            findings.first().expect("asserted len 1 above").rule,
            RuleId::New(GO_HELPERS_PACKAGE_FIVE_INPUTS)
        );
    }

    #[test]
    fn Test_Check_Go_Helpers_Package_Five_Inputs_Should_Ignore_Non_Go_Sources()
    {
        let source = Source("src/lib.rs", "pub fn Build(a: A, b: B, c: C, d: D, e: E) {}");
        let TestOffering { store, registry, .. } = Offering();

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Go_Helpers_Package_Five_Inputs(&[source], &mut reader);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Parameter_Count_Should_Read_And_Judge_A_Real_Fact()
    {
        let source = Source("src/build.rs", "pub fn Build(a: A, b: B, c: C, d: D, e: E) {}");
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Syntax_Fact(
            &mut store,
            &source,
            &offer,
            "unexpanded\t0\nitem\t0\tFunction\tPublic\tBuild\t.\t+fn/5\n",
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Parameter_Count(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "Build");
    }

    fn Payload_From_Text(text: &str) -> SyntaxPayload
    {
        return nomos_cap_syntax::Parse_Payload(text.as_bytes())
            .expect("this fixture payload is well formed");
    }

    fn Materialize_Syntax_Fact(store: &mut MemoryFactStore, source: &SourceFile, offer: &ProviderOffer, payload: &str)
    {
        let inputs = InputDigest::Of(&[source.text.as_bytes()]);
        test_support::Materialize(store, source.subject, offer, inputs, nomos_cap_syntax::Payload_Schema(), payload.as_bytes().to_vec());
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        return SourceFile::New(
            path,
            nomos_contracts::SubjectId::From_Digest(nomos_model::Content_Digest(path.as_bytes())),
            text,
        );
    }

    fn Offering() -> TestOffering
    {
        return test_support::Offering(
            nomos_cap_syntax::Capability_Contract(),
            nomos_cap_syntax::Capability(),
            nomos_cap_syntax::CONTRACT_VERSION,
            "nomos.test.parameter-count.parses",
            Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File),
        );
    }
}
