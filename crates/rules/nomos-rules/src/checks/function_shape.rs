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
//! decision `checks::naming::Resolve_Case` already generalizes, and `checks::structure::
//! Resolve_Limit` already builds a second instance for the file-size triggers. This is a
//! third: [`Resolve_Limit`] asks `nomos.cap.limits.policy` for the value-parameter ceiling
//! a repository declares, falling back to [`MAX_VALUE_PARAMETERS`] when it declares none —
//! the identical `Require`-then-fall-back-on-any-`Err` shape, duplicated locally rather than
//! shared across `structure.rs`, matching this crate's own per-file convention (`Is_Go_File`
//! already has three independent copies) until a real need for one shared copy shows up.

use crate::{GO_LANGUAGE, SourceFile};
use nomos_analysis::{FactReader, InputDigest};
use nomos_cap_limits_policy::Scope;
use nomos_cap_syntax::{FUNCTION, Function_Arity, PayloadItem, SyntaxPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, RuleId, SubjectId};

mod function_arity_policy;
mod function_arity_source;
mod receiver_allowance;

pub use function_arity_policy::FunctionArityPolicy;
pub use function_arity_source::FunctionAritySource;
pub use receiver_allowance::ReceiverAllowance;

/// The code-standards parameter-count rule id.
pub const PARAMETER_COUNT: &str = "parameter-count";
/// The Go-specific code-standards parameter-count rule id.
pub const GO_HELPERS_PACKAGE_FIVE_INPUTS: &str = "go-helpers-package-five-inputs";

/// `standards.json`'s row key for the value-parameter ceiling, shared by the generic and
/// the Go rule since Go's own value equals the default and so needs no override row.
const PARAMETER_COUNT_MAX_KEY: &str = "parameter-count-max";

const GO: &str = "go";

const MAX_VALUE_PARAMETERS: u32 = 4;

/// Reports functions that definitely exceed the value-parameter cap — a repository's own
/// declared `nomos.cap.limits.policy` when it declares `parameter-count-max`, the prior
/// hardcoded default otherwise.
#[must_use]
pub fn Check_Parameter_Count(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let max = Resolve_Limit(facts, None, PARAMETER_COUNT_MAX_KEY, MAX_VALUE_PARAMETERS);
    return Check_Function_Arity_Policy(
        sources,
        facts,
        FunctionArityPolicy::New(PARAMETER_COUNT, max).Allow_One_Receiver_For_Qualified_Functions(),
    );
}

/// Reports Go functions and methods that definitely exceed the value-parameter cap — a
/// repository's own declared `nomos.cap.limits.policy` when it declares `go`'s own
/// `parameter-count-max`, the repository-wide value or the prior hardcoded default
/// otherwise.
#[must_use]
pub fn Check_Go_Helpers_Package_Five_Inputs(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let max = Resolve_Limit(facts, Some(GO), PARAMETER_COUNT_MAX_KEY, MAX_VALUE_PARAMETERS);
    return Check_Function_Arity_Policy(
        sources,
        facts,
        FunctionArityPolicy::New(GO_HELPERS_PACKAGE_FIVE_INPUTS, max)
            .For_Language(GO_LANGUAGE)
            .Allow_One_Receiver_For_Qualified_Functions(),
    );
}

/// Reports functions that violate a caller-supplied arity policy.
///
/// A test or example source is not judged. An arity cap is a claim about code somebody has
/// to call, and a test's parameters are its fixtures: the shapes that make a case say what
/// it varies. This is [`crate::checks::Is_Test_Or_Example_Source`], the same exemption
/// `concurrency_text` and `file_names` already read, applied here for the first time; it sits
/// beside the policy's language filter rather than inside it because where a file sits and
/// what it is written in are different questions, and `Policy_Accepts_Source` answers only
/// the second.
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
        if !Policy_Accepts_Source(policy, source) || crate::checks::Is_Test_Or_Example_Source(source)
        {
            continue;
        }

        let source_findings = Findings_For_Source(policy, facts, source);
        findings.extend(source_findings);
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Policy_Accepts_Source(policy: FunctionArityPolicy, source: &SourceFile) -> bool
{
    return match policy.source
    {
        FunctionAritySource::All => true,
        FunctionAritySource::Language(expected) => source.Is_Written_In(expected),
    };
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
        .filter(|(item, arity)| return Definitely_Too_Many_Value_Parameters(policy, item, *arity))
        .map(|(item, arity)| return Violation_Finding(policy, path, item, arity))
        .collect();
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

/// This crate's own floor for `nomos.cap.limits.policy` — stated at the capability's own
/// ceiling since there is only one real provider today and no weaker answer this crate
/// could honestly still act on. Mirrors `checks::naming::Naming_Policy_Requirement` and
/// `checks::structure::Limits_Policy_Requirement` exactly, for the identical capability.
fn Limits_Policy_Requirement() -> nomos_capability::Requirement
{
    return nomos_capability::Requirement::New(
        nomos_cap_limits_policy::Capability(),
        nomos_cap_limits_policy::CONTRACT_VERSION,
        nomos_cap_limits_policy::Ceiling(),
    );
}

/// Resolves the numeric ceiling `key` must take: a repository's own declared `nomos.cap.
/// limits.policy`, most-specific key first (`language`'s own override, then the
/// repository-wide default), falling back to `default` when neither is declared.
///
/// `OD-CAPABILITY-004` and `OD-RULES-011` settle how an absent read is treated here,
/// mirroring `checks::structure::Resolve_Limit` exactly: this capability is optional,
/// every caller already has a complete answer without it, so `facts.Require` failing for
/// any reason is exactly "no override" — never a `Finding`, never this capability's own
/// `Applicability` surfacing anywhere.
fn Resolve_Limit(facts: &mut dyn FactReader, language: Option<&str>, key: &str, default: u32) -> u32
{
    let Some(payload) = Limits_Policy_Payload(facts)
    else
    {
        return default;
    };

    if let Some(language) = language
    {
        let scope = Scope::Language(language.to_owned());
        if let Some(value) = Scoped_Row_Value(&payload, &scope, key)
        {
            return value;
        }
    }

    return Scoped_Row_Value(&payload, &Scope::Repository, key).unwrap_or(default);
}

/// Reads and parses this crate's own `nomos.cap.limits.policy` fact, collapsing every
/// failure reason (the capability is unread, or its payload does not parse) into `None` —
/// the caller's fallback-to-default is identical either way.
fn Limits_Policy_Payload(facts: &mut dyn FactReader) -> Option<nomos_cap_limits_policy::LimitsPolicyPayload>
{
    let subject = nomos_model::Subject_Of_Path("");
    let fact = facts
        .Require(&nomos_cap_limits_policy::Capability(), &subject, InputDigest::Of(&[]), &Limits_Policy_Requirement())
        .ok()?;
    return nomos_cap_limits_policy::Parse_Payload(&fact.payload.bytes).ok();
}

/// The value of the first row in `payload` matching both `scope` and `key`, if one exists.
fn Scoped_Row_Value(payload: &nomos_cap_limits_policy::LimitsPolicyPayload, scope: &Scope, key: &str) -> Option<u32>
{
    return payload.rows.iter().find(|row| return row.scope == *scope && row.key == key).map(|row| return row.value);
}

#[cfg(test)]
mod tests;
