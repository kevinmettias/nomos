//! Function-shape rules that are visible in `nomos.cap.syntax.items`.
//!
//! The current syntax payload carries function arity, including a receiver when one is
//! declared. It does not mark whether a qualified function's first input is a receiver, so
//! this rule reports only cases that are definitely over code-standards' four value
//! parameter cap: top-level/module functions with arity greater than four, and qualified
//! functions/methods with arity greater than five. The latter threshold allows one possible
//! receiver without producing a false positive.
//!
//! The public rule functions below are presets, not the rule engine itself. The engine is
//! [`Check_Function_Arity_Policy`], whose dimensions are deliberately data: rule id, source
//! selection, value-parameter ceiling, receiver allowance and gate category. That is the
//! shape code-standards already has in practice across `parameter-count` and
//! `go-helpers-package-five-inputs`, and it leaves a repository room to adopt the same
//! judgment with a different threshold or language surface.

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_cap_syntax::{FUNCTION, Function_Arity, PayloadItem, SyntaxPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

/// The code-standards parameter-count rule id.
pub const PARAMETER_COUNT: &str = "parameter-count";
/// The Go-specific code-standards parameter-count rule id.
pub const GO_HELPERS_PACKAGE_FIVE_INPUTS: &str = "go-helpers-package-five-inputs";

const MAX_VALUE_PARAMETERS: u32 = 4;

/// Which source files a function-arity policy applies to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FunctionAritySource
{
    /// Every source handed to the rule.
    All,
    /// Only files with this extension, case-insensitively and without the leading dot.
    Extension(&'static str),
}

/// Whether the policy may treat one input on a qualified function as a receiver.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReceiverAllowance
{
    /// Qualified and unqualified functions are judged against the same ceiling.
    None,
    /// A qualified function may have one extra input, because it may be a receiver.
    OneForQualifiedFunctions,
}

/// A configurable function-arity rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FunctionArityPolicy
{
    /// The rule id reported on findings.
    pub rule: &'static str,
    /// Which files this policy judges.
    pub source: FunctionAritySource,
    /// Maximum value parameters allowed before a finding is reported.
    pub max_value_parameters: u32,
    /// Whether qualified functions get one possible receiver input.
    pub receiver_allowance: ReceiverAllowance,
    /// The gate category reported for supported findings.
    pub gate: GateCategory,
}

impl FunctionArityPolicy
{
    /// Builds a policy for every source with no receiver allowance.
    #[must_use]
    pub const fn New(rule: &'static str, max_value_parameters: u32) -> Self
    {
        return Self {
            rule,
            source: FunctionAritySource::All,
            max_value_parameters,
            receiver_allowance: ReceiverAllowance::None,
            gate: GateCategory::Blocking,
        };
    }

    /// Narrows this policy to a file extension.
    #[must_use]
    pub const fn For_Extension(mut self, extension: &'static str) -> Self
    {
        self.source = FunctionAritySource::Extension(extension);
        return self;
    }

    /// Allows one possible receiver for qualified functions.
    #[must_use]
    pub const fn Allow_One_Receiver_For_Qualified_Functions(mut self) -> Self
    {
        self.receiver_allowance = ReceiverAllowance::OneForQualifiedFunctions;
        return self;
    }

    /// Changes the gate category reported by supported findings.
    #[must_use]
    pub const fn With_Gate(mut self, gate: GateCategory) -> Self
    {
        self.gate = gate;
        return self;
    }
}

/// Reports functions that definitely exceed the four-value-parameter cap.
#[must_use]
pub fn Check_Parameter_Count(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    return Check_Function_Arity_Policy(
        sources,
        facts,
        FunctionArityPolicy::New(PARAMETER_COUNT, MAX_VALUE_PARAMETERS).Allow_One_Receiver_For_Qualified_Functions(),
    );
}

/// Reports Go functions and methods that definitely exceed the four-value-parameter cap.
#[must_use]
pub fn Check_Go_Helpers_Package_Five_Inputs(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    return Check_Function_Arity_Policy(
        sources,
        facts,
        FunctionArityPolicy::New(GO_HELPERS_PACKAGE_FIVE_INPUTS, MAX_VALUE_PARAMETERS)
            .For_Extension("go")
            .Allow_One_Receiver_For_Qualified_Functions(),
    );
}

/// Reports functions that violate a caller-supplied arity policy.
#[must_use]
pub fn Check_Function_Arity_Policy(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
    policy: FunctionArityPolicy,
) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if !Policy_Accepts_Source(policy, source)
        {
            continue;
        }

        match crate::checks::naming::reading::Payload_Of(source, facts)
        {
            Ok(payload) => findings.extend(Violations_In(policy, &payload, &source.path)),
            Err(finding) => findings.push(Unread_As_Rule(finding, policy.rule)),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Violations_In(policy: FunctionArityPolicy, payload: &SyntaxPayload, path: &str) -> Vec<Finding>
{
    return payload
        .items
        .iter()
        .filter(|item| return item.kind == FUNCTION)
        .filter_map(|item| return Function_Arity(&item.shape).map(|arity| return (item, arity)))
        .filter(|(item, arity)| return Definitely_Too_Many_Value_Parameters(policy, item, *arity))
        .map(|(item, arity)| return Violation_Finding(policy, path, item, arity))
        .collect();
}

fn Policy_Accepts_Source(policy: FunctionArityPolicy, source: &SourceFile) -> bool
{
    return match policy.source
    {
        FunctionAritySource::All => true,
        FunctionAritySource::Extension(expected) => Source_Has_Extension(source, expected),
    };
}

fn Source_Has_Extension(source: &SourceFile, expected: &str) -> bool
{
    return std::path::Path::new(&source.path)
        .extension()
        .is_some_and(|extension| return extension.eq_ignore_ascii_case(expected));
}

fn Definitely_Too_Many_Value_Parameters(policy: FunctionArityPolicy, item: &PayloadItem, arity: u32) -> bool
{
    let allowed = match policy.receiver_allowance
    {
        ReceiverAllowance::OneForQualifiedFunctions if item.qualified_name.contains("::") =>
        {
            policy.max_value_parameters.saturating_add(1)
        }
        ReceiverAllowance::None | ReceiverAllowance::OneForQualifiedFunctions => policy.max_value_parameters,
    };

    return arity > allowed;
}

fn Violation_Finding(policy: FunctionArityPolicy, path: &str, item: &PayloadItem, arity: u32) -> Finding
{
    use nomos_model::Content_Digest;

    let qualified = format!("{path}::{}", item.qualified_name);

    return Finding {
        rule: RuleId::New(policy.rule),
        subject: SubjectId::From_Digest(Content_Digest(qualified.as_bytes())),
        subject_name: item.qualified_name.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: policy.gate,
        summary: format!(
            "`{}` has arity {arity}, which exceeds the configured value parameter cap of {}",
            item.qualified_name, policy.max_value_parameters
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

        let findings = Violations_In(Parameter_Count_Policy(), &payload, "src/lib.rs");

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(PARAMETER_COUNT));
    }

    #[test]
    fn Test_Violations_In_Should_Accept_A_Free_Function_With_Four_Parameters()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tBuild\t.\t+fn/4\n");

        let findings = Violations_In(Parameter_Count_Policy(), &payload, "src/lib.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Violations_In_Should_Not_Report_A_Qualified_Function_At_Receiver_Plus_Four()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tBuilder::Build\t.\t+fn/5\n");

        let findings = Violations_In(Parameter_Count_Policy(), &payload, "src/lib.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Violations_In_Should_Report_A_Qualified_Function_With_Six_Parameters()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tBuilder::Build\t.\t+fn/6\n");

        let findings = Violations_In(Parameter_Count_Policy(), &payload, "src/lib.rs");

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Violations_In_Should_Use_The_Configured_Rule_Id_Threshold_And_Gate()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tBuild\t.\t+fn/3\n");
        let policy = FunctionArityPolicy::New("custom-three-parameter-cap", 2)
            .With_Gate(GateCategory::Advisory);

        let findings = Violations_In(policy, &payload, "src/lib.rs");

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New("custom-three-parameter-cap"));
        assert_eq!(found.gate, GateCategory::Advisory);
        assert!(found.summary.contains("cap of 2"), "{}", found.summary);
    }

    #[test]
    fn Test_Check_Function_Arity_Policy_Should_Filter_By_Configured_Source_Extension()
    {
        let source = Source("src/lib.rs", "pub fn Build(a: A, b: B, c: C) {}");
        let TestOffering { store, registry, .. } = Offering();
        let policy = FunctionArityPolicy::New("custom-go-only-cap", 2).For_Extension("go");

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Function_Arity_Policy(&[source], &mut reader, policy);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Violations_In_Should_Apply_Receiver_Allowance_Only_When_Configured()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tBuilder::Build\t.\t+fn/5\n");

        let strict = Violations_In(FunctionArityPolicy::New("strict-cap", 4), &payload, "src/lib.rs");
        let receiver_aware = Violations_In(
            FunctionArityPolicy::New("receiver-aware-cap", 4).Allow_One_Receiver_For_Qualified_Functions(),
            &payload,
            "src/lib.rs",
        );

        assert_eq!(strict.len(), 1, "{strict:?}");
        assert!(receiver_aware.is_empty(), "{receiver_aware:?}");
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

    fn Parameter_Count_Policy() -> FunctionArityPolicy
    {
        return FunctionArityPolicy::New(PARAMETER_COUNT, MAX_VALUE_PARAMETERS).Allow_One_Receiver_For_Qualified_Functions();
    }
}
