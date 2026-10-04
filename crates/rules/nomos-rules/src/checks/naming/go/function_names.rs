//! Go functions keep the workspace's operation-name convention on both sides of
//! visibility.
//!
//! code-standards' `exported-functions-use-upper-snake-case` and
//! `unexported-functions-lowercase-only-the-first-letter` are Go-specific and split the
//! same convention by visibility, because Go decides visibility by a name's first letter
//! rather than a keyword: an unexported name lowercases only that first letter and keeps
//! the rest of the shape intact (`Run_With_Backend` becomes `run_With_Backend`).
//!
//! # `OD-RULES-011` fixed a real drift here
//!
//! This crate's own prior reading of "`Upper_Snake_Case`" for the exported half was every
//! character uppercase, digit, or `_` — `screaming-snake` in code-standards' own closed
//! vocabulary, not `upper-snake` (`Compute_Total`, `^[A-Z][A-Za-z0-9]*(_[A-Z0-9]
//! [A-Za-z0-9]*)*$`), which is what the rule id actually names and what `Check_Naming_
//! Convention`'s own `Pascal_Snake_Case` already implements correctly. The unexported half
//! mirrored that same wrong shape rather than [`nomos_cap_naming_policy::Case::MixedSnake`]
//! (`compute_Total`), the style code-standards' own vocabulary names for exactly "the
//! first word lower, the rest cased normally." Both are corrected to their real defaults
//! here, the same way every casing rule in this crate now resolves its case from a
//! repository's own `nomos.cap.naming.policy` rather than a hand-rolled predicate.
//!
//! # Which keys a Go function's case is read under
//!
//! `OD-RULES-035` decision 8. A Go function reads the refinement its side of visibility selects,
//! then `function`, each looked up for `go` before repository-wide: code-standards' own order over
//! the same block, which falls back to `function` where these rules once stopped at the
//! refinement. A Go method -- a function declared on a receiver -- reads `method.exported` or
//! `method.unexported`, then `method`, ahead of those. With none of them declared each rule keeps
//! its refinement's own Go default, which is what it judged against before it read the others.
//!
//! Reading `function` holds an unexported Go function to a case that may be one no unexported Go
//! name can take: this repository declares `function` upper-snake, and Go exports a name by its
//! first letter. That is the repository's declaration, and code-standards reads it the same way.
//! What it must not reach is `main` and `init` declared without a receiver, which the Go
//! specification names and no repository can rename, so the unexported rule judges neither.

use crate::checks::naming::function_cases::Is_Go_Method;
use crate::checks::naming::{Case_Judged_Against, Resolve_Read};
use crate::checks::optional_reads::{GO_EXPORTED_FUNCTION, GO_EXPORTED_METHOD, GO_UNEXPORTED_FUNCTION, GO_UNEXPORTED_METHOD};
use crate::rule_descriptor::NamingRead;
use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_cap_naming_policy::Case;
use nomos_cap_syntax::{FUNCTION, PayloadItem, SyntaxPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

/// This rule's own identifier, matching the code-standards rule id.
pub const EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE: &str = "exported-functions-use-upper-snake-case";
/// The code-standards identifier for the unexported half of the same convention.
pub const UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER: &str = "unexported-functions-lowercase-only-the-first-letter";

/// The names the Go specification gives a function declared without a receiver: a program's entry
/// point and a package's initializer. Neither is the author's to choose.
const NAMED_BY_GO: [&str; 2] = ["main", "init"];

/// Judges exported Go function and method names against the case a repository's own
/// `nomos.cap.naming.policy` declares for them, `Upper_Snake_Case` when it declares none.
#[must_use]
pub fn Check_Exported_Go_Functions_Use_Upper_Snake_Case(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let Some(cases) = Go_Cases(facts, &GO_EXPORTED_FUNCTION, &GO_EXPORTED_METHOD)
    else
    {
        return Vec::new();
    };
    return Judged_Go_Function_Sources(
        sources,
        facts,
        Judgment {
            cases,
            judge: ViolationConstructor(Violations_In),
            unread: UnreadFindingConstructor(Unread_As_This_Rule),
        },
    );
}

/// Judges unexported Go function and method names against the case a repository's own
/// `nomos.cap.naming.policy` declares for them, and against the same convention with only its
/// first word lowercased when it declares none.
#[must_use]
pub fn Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let Some(cases) = Go_Cases(facts, &GO_UNEXPORTED_FUNCTION, &GO_UNEXPORTED_METHOD)
    else
    {
        return Vec::new();
    };
    return Judged_Go_Function_Sources(
        sources,
        facts,
        Judgment {
            cases,
            judge: ViolationConstructor(Unexported_Violations_In),
            unread: UnreadFindingConstructor(Unread_As_Unexported_Rule),
        },
    );
}

/// The case a Go name on one rule's side of visibility is judged against: a function's, or a
/// method's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GoCases
{
    function: Case,
    method: Case,
}

impl GoCases
{
    /// The case `item` is judged against: a method's when it is declared on a receiver.
    fn For(self, item: &PayloadItem) -> Case
    {
        if Is_Go_Method(item)
        {
            return self.method;
        }

        return self.function;
    }
}

/// The cases one side of visibility reads for Go, `function` for a function and `method` for a
/// method.
fn Go_Cases(facts: &mut dyn FactReader, function: &NamingRead, method: &NamingRead) -> Option<GoCases>
{
    return Some(GoCases { function: Resolve_Read(facts, function)?, method: Resolve_Read(facts, method)? });
}

fn Violations_In(payload: &SyntaxPayload, path: &str, cases: GoCases) -> Vec<Finding>
{
    return payload
        .items
        .iter()
        .filter(|item| return Is_Exported_Go_Function(item))
        .filter(|item| return !cases.For(item).Is_The_Shape_Of(item.Own_Name()))
        .map(|item| return Violation_Finding(path, item, cases.For(item)))
        .collect();
}

fn Is_Exported_Go_Function(item: &PayloadItem) -> bool
{
    return item.kind == FUNCTION && item.Is_Public();
}

fn Violation_Finding(path: &str, item: &PayloadItem, case: Case) -> Finding
{
    let name = item.Own_Name();
    let summary = format!("exported Go function `{name}` is not {}", Case_Judged_Against(case));
    return Function_Naming_Finding(EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE, path, item, summary);
}

fn Unexported_Violations_In(payload: &SyntaxPayload, path: &str, cases: GoCases) -> Vec<Finding>
{
    return payload
        .items
        .iter()
        .filter(|item| return Is_Unexported_Go_Function(item) && !Is_Named_By_Go(item))
        .filter(|item| return !cases.For(item).Is_The_Shape_Of(item.Own_Name()))
        .map(|item| return Unexported_Violation_Finding(path, item, cases.For(item)))
        .collect();
}

fn Is_Unexported_Go_Function(item: &PayloadItem) -> bool
{
    return item.kind == FUNCTION && !item.Is_Public();
}

/// Whether `item` is a function declared without a receiver under a name [`NAMED_BY_GO`] holds.
fn Is_Named_By_Go(item: &PayloadItem) -> bool
{
    return !Is_Go_Method(item) && NAMED_BY_GO.contains(&item.qualified_name.as_str());
}

fn Unexported_Violation_Finding(path: &str, item: &PayloadItem, case: Case) -> Finding
{
    let name = item.Own_Name();
    let summary = format!("unexported Go function `{name}` is not {}", Case_Judged_Against(case));
    return Function_Naming_Finding(UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER, path, item, summary);
}

fn Unread_As_This_Rule(mut finding: Finding) -> Finding
{
    finding.rule = RuleId::New(EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE);
    finding.summary = finding
        .summary
        .replace("this file's naming could not be judged", "this Go file's exported function names could not be judged");
    return finding;
}

fn Unread_As_Unexported_Rule(mut finding: Finding) -> Finding
{
    finding.rule = RuleId::New(UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER);
    finding.summary = finding
        .summary
        .replace("this file's naming could not be judged", "this Go file's unexported function names could not be judged");
    return finding;
}

/// The shape both checks above share: skip a non-Go source, read each remaining source's
/// own syntax fact, and fold either a real reading failure or `judge`'s own findings
/// (against the already-resolved `case`) into one sorted list.
/// Constructs the violations one payload's function names hold against an already-resolved
/// case, named so it reads as a collaborator with one documented operation rather than a
/// bare stored callable.
struct ViolationConstructor(fn(&SyntaxPayload, &str, GoCases) -> Vec<Finding>);

impl ViolationConstructor
{
    fn Violations(&self, payload: &SyntaxPayload, path: &str, cases: GoCases) -> Vec<Finding>
    {
        return (self.0)(payload, path, cases);
    }
}

/// Wraps a reading failure into this rule's own finding, named for the same reason as
/// [`ViolationConstructor`].
struct UnreadFindingConstructor(fn(Finding) -> Finding);

impl UnreadFindingConstructor
{
    fn Wrap(&self, finding: Finding) -> Finding
    {
        return (self.0)(finding);
    }
}

/// How to judge one payload's function names: the already-resolved cases a name must match,
/// the violation constructor, and how to wrap a reading failure into its own finding.
/// Grouped so the two callers above and this function stay under the parameter-count
/// ceiling.
struct Judgment
{
    cases: GoCases,
    judge: ViolationConstructor,
    unread: UnreadFindingConstructor,
}

fn Judged_Go_Function_Sources(sources: &[SourceFile], facts: &mut dyn FactReader, judgment: Judgment) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if !crate::checks::populations::GO_FUNCTION_NAMES_POPULATION.Holds(source)
        {
            continue;
        }

        match super::super::reading::Payload_Of(source, facts)
        {
            Ok(payload) =>
            {
                let violations = judgment.judge.Violations(&payload, &source.path, judgment.cases);
                findings.extend(violations);
            }
            Err(finding) => findings.push(judgment.unread.Wrap(finding)),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// The `Finding` shape both violation constructors above share: only the rule id and the
/// already-composed summary differ between an exported and an unexported Go function name.
fn Function_Naming_Finding(rule: &'static str, path: &str, item: &PayloadItem, summary: String) -> Finding
{
    use nomos_model::Content_Digest;

    let qualified = format!("{path}::{}", item.qualified_name);

    return Finding {
        address: Some(qualified.clone()),
        rule: RuleId::New(rule),
        subject: SubjectId::From_Digest(Content_Digest(qualified.as_bytes())),
        subject_name: item.qualified_name.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary,
        locations: vec![path.to_owned()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Violations_In_Should_Report_Exported_Go_Functions_That_Are_Not_Upper_Snake()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\trun_With_Backend\t.\t+fn/0\n");

        let findings = Violations_In(&payload, "main.go", Alike(Case::UpperSnake));

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "run_With_Backend");
    }

    #[test]
    fn Test_Violations_In_Should_Accept_Exported_Go_Functions_In_Upper_Snake()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tRun_With_Backend\t.\t+fn/0\n");

        let findings = Violations_In(&payload, "main.go", Alike(Case::UpperSnake));

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Violations_In_Should_Ignore_Unexported_Go_Functions()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPrivate\trunWithBackend\t.\t+fn/0\n");

        let findings = Violations_In(&payload, "main.go", Alike(Case::UpperSnake));

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// The doc's own worked example: only the first word lowercases.
    #[test]
    fn Test_Unexported_Violations_In_Should_Accept_Only_The_First_Word_Lowercased()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPrivate\trow_Breaches\t.\t+fn/0\n");

        let findings = Unexported_Violations_In(&payload, "index.go", Alike(Case::MixedSnake));

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Unexported_Violations_In_Should_Report_A_Recased_Name_With_No_Word_Boundary()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPrivate\trowBreaches\t.\t+fn/0\n");

        let findings = Unexported_Violations_In(&payload, "index.go", Alike(Case::MixedSnake));

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(
            findings.first().expect("asserted len 1 above").rule,
            RuleId::New(UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER)
        );
    }

    #[test]
    fn Test_Unexported_Violations_In_Should_Report_A_First_Word_That_Is_Not_Fully_Lowercased()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPrivate\tRow_Breaches\t.\t+fn/0\n");

        let findings = Unexported_Violations_In(&payload, "index.go", Alike(Case::MixedSnake));

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Unexported_Violations_In_Should_Ignore_Exported_Go_Functions()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tRow_Breaches\t.\t+fn/0\n");

        let findings = Unexported_Violations_In(&payload, "index.go", Alike(Case::MixedSnake));

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// A method is judged against the method's case, a free function against the function's, and
    /// each finding names the case its own subject was judged against.
    #[test]
    fn Test_Violations_In_Should_Judge_A_Method_Against_The_Methods_Case()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tFunction\tPublic\tRowCount\t.\t+fn/0\n\
             item\t1\tFunction\tPublic\tTable::Row_Count\t.\t+fn/1\n",
        );
        let cases = GoCases { function: Case::UpperSnake, method: Case::UpperCamel };

        let findings = Violations_In(&payload, "table.go", cases);

        assert_eq!(Summaries(&findings), vec!["exported Go function `Row_Count` is not upper-camel case"], "{findings:?}");
    }

    /// `main` and `init` declared without a receiver are the Go specification's, and no declared
    /// case reaches them; a method that happens to share the name is the author's, and is judged.
    #[test]
    fn Test_Unexported_Violations_In_Should_Not_Judge_A_Name_Go_Gives_A_Function()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tFunction\tPrivate\tmain\t.\t+fn/0\n\
             item\t1\tFunction\tPrivate\tinit\t.\t+fn/0\n\
             item\t2\tFunction\tPrivate\tTable::init\t.\t+fn/1\n",
        );

        let findings = Unexported_Violations_In(&payload, "main.go", Alike(Case::UpperSnake));

        assert_eq!(Summaries(&findings), vec!["unexported Go function `init` is not upper-snake case"], "{findings:?}");
        assert_eq!(findings.first().expect("asserted one finding above").subject_name, "Table::init");
    }

    /// One case for a function and a method alike: what every test above judges against.
    fn Alike(case: Case) -> GoCases
    {
        return GoCases { function: case, method: case };
    }

    fn Summaries(findings: &[Finding]) -> Vec<&str>
    {
        return findings.iter().map(|finding| return finding.summary.as_str()).collect();
    }

    fn Payload_From_Text(text: &str) -> SyntaxPayload
    {
        return nomos_cap_syntax::Parse_Payload(text.as_bytes())
            .expect("this fixture payload is well formed");
    }
}
