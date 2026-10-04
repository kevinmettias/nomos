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
//!
//! # The ceiling is now a repository's own, resolved rather than compiled in
//!
//! `OD-RULES-011` named this threshold family as its own future instance of the naming
//! decision `checks::naming::Resolve_Read` already generalizes. The ceiling is the
//! `PARAMETER_COUNT_MAX` axis in `rule_descriptor::policy_axis`, read through
//! `checks::structure::Resolve_Limit`, the limits family's one resolver, which falls back to
//! the axis's declared default when a repository declares none. This file kept its own copy
//! of that resolver until `OD-RULES-035` decided a family has one; the two copies had already
//! diverged on the integer type they returned.

use super::structure::{Resolve_Limit, Undeclared_Limit};
use super::optional_reads;
use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_cap_syntax::{FUNCTION, Function_Arity, PayloadItem, SyntaxPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, RuleId, SubjectId};

mod function_arity_policy;
mod go_parameter_count;
mod receiver_allowance;

pub use function_arity_policy::FunctionArityPolicy;
pub use go_parameter_count::Check_Go_Parameter_Count;
pub use receiver_allowance::ReceiverAllowance;

/// The code-standards parameter-count rule id.
pub const PARAMETER_COUNT: &str = "parameter-count";
/// The Go-specific code-standards parameter-count rule id.
pub const GO_PARAMETER_COUNT: &str = "go-helpers-package-five-inputs";

/// Reports functions that definitely exceed the value-parameter cap — a repository's own
/// declared `nomos.cap.limits.policy` when it declares `parameter-count-max`, the axis's
/// declared default otherwise.
#[must_use]
pub fn Check_Parameter_Count(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let Some(max) = Resolve_Limit(facts, &optional_reads::PARAMETER_COUNT)
    else
    {
        return vec![Undeclared_Limit(PARAMETER_COUNT, optional_reads::PARAMETER_COUNT.axis)];
    };
    return Check_Function_Arity_Policy(
        sources,
        facts,
        FunctionArityPolicy::New(PARAMETER_COUNT, max).Allow_One_Receiver_For_Qualified_Functions(),
    );
}

/// Reports functions that violate a caller-supplied arity policy.
///
/// A test or example source is not judged. An arity cap is a claim about code somebody has
/// to call, and a test's parameters are its fixtures: the shapes that make a case say what
/// it varies. This is [`crate::checks::Is_Test_Or_Example_Source`], the same exemption
/// `concurrency_text` and `file_names` already read, applied here for the first time; it sits
/// beside the policy's population filter rather than inside it because where a file sits and
/// what it is written in are different questions, and `Is_Policy_Accepting_Source` answers only
/// the second.
#[must_use]
pub fn Check_Function_Arity_Policy(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
    policy: FunctionArityPolicy,
) -> Vec<Finding>
{
    let declared = crate::checks::Resolve_Declared_Fixture_Locations(facts);
    let mut findings = Vec::new();

    for source in sources
    {
        if !Is_Policy_Accepting_Source(policy, source) || crate::checks::Is_Test_Or_Example_Source(source, &declared)
        {
            continue;
        }

        let source_findings = Findings_For_Source(policy, facts, source);
        findings.extend(source_findings);
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Is_Policy_Accepting_Source(policy: FunctionArityPolicy, source: &SourceFile) -> bool
{
    return policy.source.Holds(source);
}

/// One source's findings under `policy`: its function-arity violations when its syntax
/// payload reads, or the payload's own unreadable-source finding relabeled under `policy`'s
/// rule id otherwise.
fn Findings_For_Source(policy: FunctionArityPolicy, facts: &mut dyn FactReader, source: &SourceFile) -> Vec<Finding>
{
    return match crate::checks::naming::reading::Payload_Of(source, facts)
    {
        Ok(payload) => Violations_In(policy, &payload, &source.path),
        Err(finding) =>
        {
            let unread = Unread_As_Rule(finding, policy.rule);
            vec![unread]
        }
    };
}

fn Violations_In(policy: FunctionArityPolicy, payload: &SyntaxPayload, path: &str) -> Vec<Finding>
{
    return payload
        .items
        .iter()
        .filter(|item| return item.kind == FUNCTION)
        .filter_map(|item| return Function_Arity(&item.shape).map(|arity| return (item, arity)))
        .filter(|(item, arity)| return Has_Definitely_Too_Many_Value_Parameters(policy, item, *arity))
        .map(|(item, arity)| return Violation_Finding(policy, path, item, arity))
        .collect();
}

fn Has_Definitely_Too_Many_Value_Parameters(policy: FunctionArityPolicy, item: &PayloadItem, arity: u32) -> bool
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
        address: Some(qualified.clone()),
        rule: RuleId::New(policy.rule),
        subject: SubjectId::From_Digest(Content_Digest(qualified.as_bytes())),
        subject_name: item.qualified_name.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: policy.gate,
        // The cap is named by its value alone: it was "the configured" cap whether or not anything
        // configured it, and `OD-RULES-011` version 3 decision 4 keeps a finding's text, and the
        // identities hashed from it, the same whichever source supplied the value.
        summary: format!(
            "`{}` has arity {arity}, which exceeds the value parameter cap of {}",
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
mod tests;
