//! A function's own name must match this workspace's `Pascal_Snake_Case` convention.
//!
//! `README.md`'s Conventions section states the rule directly: "Function names are
//! `Pascal_Snake_Case`... Types stay `UpperCamelCase`." `Cargo.toml`'s
//! `[workspace.lints.rust]` names the resulting gap in its own comment — the one rustc
//! lint that would have caught a deviation from Rust's *own* convention, `non_snake_case`,
//! is turned off "because... this is the workspace convention, not an oversight" — and
//! nothing was turned on to check the convention that replaced it. This rule is that check.
//!
//! # A second rule, deliberately different in shape from the first
//!
//! [`crate::Check_Completeness_Mirrors`] reconciles a claim embedded in one place (a doc
//! comment) against a fact derived from another (whether a name it claims exists). This
//! rule is a one-sided lexical predicate over each function's own name, stated once in
//! prose rather than declared per-subject — no claimed mirror, no cross-file resolution,
//! no [`crate::Reading::Unobserved`] handling, because nothing here depends on
//! documentation being observed at all. `OD-PACKAGE-008` asked whether a `RulePackage`
//! manifest's field boundaries generalize past a population of one rule; this is the
//! second data point that question needs.
//!
//! # Why this has no `CONTRACT_RECORD`
//!
//! Unlike [`crate::Check_Completeness_Mirrors`], whose contract is `D-134` — a versioned
//! governing record `tests/contract/tests/rule_contract_citation.rs` checks against — this
//! rule's contract is `README.md`'s Conventions section: prose, not a record with a
//! `version:` front-matter field that same mechanism could read. `PKG-014`'s traceability
//! requirement is accordingly not mechanically checked for this rule, the same way it was
//! not checked for the first one before that citation existed. Left named rather than
//! manufactured — a record authored only to give this rule something to cite would be the
//! record standing in for the check, not the other way around.
//!
//! # Split by responsibility
//!
//! [`reading`] requires and decodes one file's own syntax fact. [`violations`] judges an
//! already-decoded payload against the naming convention. This file keeps only what
//! composes the two: the rule's own identifier and [`Check_Naming_Convention`] itself, plus
//! the end-to-end tests that exercise both together through a real reader.
//!
//! # The convention is now a repository's own, resolved rather than compiled in
//!
//! `OD-RULES-011` generalizes this rule's `Pascal_Snake_Case` default, and every other
//! casing rule this crate ships, onto one shared read: [`Resolve_Case`] asks `nomos.cap.
//! naming.policy` for the case a symbol key must take, falling back to each rule's own
//! prior hardcoded default when a repository declares none. The convention above is that
//! default, still real and still what an unconfigured repository gets, not what every
//! repository is now fixed to.
//!
//! # A function reads its case through the refinement its visibility selects
//!
//! `OD-RULES-035` decision 7 settled which keys this rule reads for Rust, by counting the keys
//! the repositories that declare a Rust function case actually write: three write the plain
//! `function`, and xvpe writes only `function.exported` and `function.unexported`. So a Rust
//! function declared exactly `pub` is judged against the first of `function.exported` and
//! `function` the repository declares, and every other Rust function against the first of
//! `function.unexported` and `function` -- each looked up for `rust` before repository-wide,
//! code-standards' own order -- and against upper-snake when neither is declared.
//! [`Resolve_Read`] is that read, and [`function_cases`] is the split.
//!
//! Decision 8 reads every other language the same way, each key looked up for the language the
//! source is written in: xvpe and code-standards declare their Go function case only under
//! `languages.go.naming`, which this rule never read. It also has a Go method read
//! `method.exported` or `method.unexported`, then `method`, ahead of the function keys, because
//! Go's syntax fact says which functions are methods. Rust's does not -- it does not say which
//! parameter is a receiver, which is what code-standards means by a Rust method -- so a Rust
//! method's case is still read under the function keys alone.
//!
//! # A finding names the case it judged against, and not where that case came from
//!
//! `OD-RULES-011` version 3 decision 4: a finding states the value it was judged against, and
//! nothing about that value that only one of its sources makes true. This rule's finding once
//! cited the `README.md` and `Cargo.toml` above whatever case it had resolved, which was false in
//! every repository but this one, and false here too for any case but the default. Every casing
//! rule in this family now writes the case through [`Case_Judged_Against`], so a repository that
//! declares the default reads the same text, and the same two identities hashed from it, as one
//! that declares nothing.

mod abbreviations;
mod boolean_predicates;
mod clarity;
mod data_names;
mod file_names;
mod function_cases;
mod go;
pub(super) mod reading;
mod single_letter_names;
mod test_names;
mod violations;

use crate::rule_descriptor::policy_axis::{
    CaseRead, PolicyAxis, EXPORTED_FUNCTION_READ, EXPORTED_METHOD_READ, FUNCTION_CASE, UNEXPORTED_FUNCTION_READ, UNEXPORTED_METHOD_READ,
};
use crate::rule_descriptor::RequiredFact;
use crate::{SourceFile, GO_LANGUAGE};
use function_cases::{FunctionCases, SourceCases};
use nomos_analysis::{FactReader, InputDigest};
use nomos_cap_naming_policy::{Case, NamingPolicyPayload, Scope};
use nomos_contracts::Finding;

pub use abbreviations::{Check_Abbreviations, ABBREVIATIONS};
pub use boolean_predicates::{Check_Boolean_Predicates, BOOLEAN_PREDICATES};
pub use clarity::{Check_Naming_Clarity, NAMING_CLARITY};
pub use data_names::{Check_Module_And_Field_Names_Stay_Lower_Snake, MODULE_AND_FIELD_NAMES_STAY_LOWER_SNAKE};
pub use file_names::{
    Check_File_Name_Matches_Declared_Type, Check_One_Public_Type_Per_File, FILE_NAME_MATCHES_DECLARED_TYPE,
    ONE_PUBLIC_TYPE_PER_FILE,
};
pub use go::data_names::{
    Check_Go_Constants_Split_By_Export, Check_Go_Variables_Use_Lower_Snake_Case, CONSTANTS_SPLIT_BY_EXPORT,
    GO_VARIABLES_USE_LOWER_SNAKE_CASE,
};
pub use go::function_names::{
    Check_Exported_Go_Functions_Use_Upper_Snake_Case, Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter,
    EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE, UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER,
};
pub use go::type_names::{Check_Go_Type_Names_Use_Camel_Case, TYPES_USE_UPPER_CAMEL_CASE_LOWER_CAMEL_CASE};
pub use single_letter_names::{Check_Single_Letter_Names, SINGLE_LETTER_NAMES};
pub use test_names::{Check_Test_Names_Describe_Behavior, TEST_NAME_DESCRIBES_BEHAVIOR};

/// This rule's own identifier.
pub const NAMING_CONVENTION: &str = "function-naming-convention";
/// The code-standards identifier for this workspace's function naming convention.
pub const PROJECT_OWNED_FUNCTION_NAMES_USE_UPPER_SNAKE_CASE: &str = "project-owned-function-names-use-upper-snake-case";

/// Resolves the case `axis` takes: a repository's own declared `nomos.cap.naming.policy`,
/// most-specific row first (`language`'s own override, then the repository-wide row), falling
/// back to the axis's declared default for `language` when neither is declared. The naming
/// family's one resolver; every naming axis is declared in `rule_descriptor::policy_axis`.
///
/// `None` only for an axis with no value to judge by, which is an axis reported as undeclared,
/// and `policy_axis`'s tests hold every naming axis to a default because this family has no
/// path that reports one. A caller given `None` judges nothing: with no case there is nothing
/// to judge a name against.
///
/// `OD-CAPABILITY-004` and `OD-RULES-011` settle how an absent read is treated here: this
/// capability is optional, every caller already has a complete answer without it, so
/// `facts.Require` failing for any reason is exactly "no override" — never a `Finding`,
/// never this capability's own `Applicability` surfacing anywhere.
pub(super) fn Resolve_Case(facts: &mut dyn FactReader, language: Option<&str>, axis: &PolicyAxis<Case>) -> Option<Case>
{
    return Resolve_Read(facts, language, &CaseRead { keys: &[axis], undeclared: axis });
}

/// Resolves the case `read` takes: the first of its keys the repository declares, each looked up
/// for `language` before repository-wide, and its undeclared axis's default for `language` when it
/// declares none of them. [`Resolve_Case`] is this over one axis.
///
/// `OD-RULES-035` decisions 7 and 8 are this order. It is code-standards' own for the same block
/// of the same file -- its `Override_Case` takes the refined key before the plain one, over a block
/// in which a language's key has already replaced the repository's -- so one declaration in
/// `standards.json` resolves to one case in both tools.
pub(super) fn Resolve_Read(facts: &mut dyn FactReader, language: Option<&str>, read: &CaseRead<'_>) -> Option<Case>
{
    let payload = Naming_Policy_Payload(facts, read.undeclared.family);

    return Read_From(payload.as_ref(), language, read);
}

/// [`Resolve_Read`] over a payload already read, or over none when the repository's naming
/// policy could not be read.
fn Read_From(payload: Option<&NamingPolicyPayload>, language: Option<&str>, read: &CaseRead<'_>) -> Option<Case>
{
    let declared = payload.and_then(|payload| return read.keys.iter().find_map(|axis| return Case_For_Symbol(payload, language, axis.key)));

    return declared.or_else(|| return read.undeclared.Undeclared_Value(language));
}

/// How a finding names the case a name was judged against: the spelling a repository writes for
/// it in `standards.json`, as `upper-snake case`, and nothing about whether a repository wrote it.
///
/// Every casing rule in this family writes its case through this one function, so the text a
/// finding carries, and both identities `nomos-model` hashes from it, are the same whether the
/// case was declared or is the axis's undeclared value -- `OD-RULES-011` version 3 decision 4.
pub(super) fn Case_Judged_Against(case: Case) -> String
{
    return format!("{} case", case.Label());
}

/// The cases `source`'s functions are judged against, read for the language it is written in: one
/// per side of visibility, each through the refinement of `function` that side selects ahead of
/// `function` itself, and for a Go source a method's as well, through the method keys first.
fn Source_Cases(payload: Option<&NamingPolicyPayload>, source: &SourceFile) -> Option<SourceCases>
{
    let language = source.language.as_ref().map(nomos_cap_syntax::Language::As_Str);
    let sides = |exported: &CaseRead<'_>, unexported: &CaseRead<'_>| {
        return Some(FunctionCases { exported: Read_From(payload, language, exported)?, unexported: Read_From(payload, language, unexported)? });
    };
    let methods = if source.Is_Written_In(GO_LANGUAGE) { Some(sides(&EXPORTED_METHOD_READ, &UNEXPORTED_METHOD_READ)?) } else { None };

    return Some(SourceCases { functions: sides(&EXPORTED_FUNCTION_READ, &UNEXPORTED_FUNCTION_READ)?, methods });
}

/// Reads and decodes this repository's own `nomos.cap.naming.policy` fact, folding every
/// way the read can come back empty (absent, refused, malformed) into `None` — `Resolve_
/// Case`'s caller already has a complete default for that case, per `OD-CAPABILITY-004`.
///
/// The capability required is the one `family` names, which for every naming axis is
/// `nomos.cap.naming.policy`: `rule_descriptor::policy_axis`'s own tests pin that.
fn Naming_Policy_Payload(facts: &mut dyn FactReader, family: RequiredFact) -> Option<NamingPolicyPayload>
{
    let subject = nomos_model::Subject_Of_Path("");
    let fact = facts
        .Require(&family.Capability(), &subject, InputDigest::Of(&[]), &Naming_Policy_Requirement())
        .ok()?;

    return nomos_cap_naming_policy::Parse_Payload(&fact.payload.bytes).ok();
}

/// This crate's own floor for `nomos.cap.naming.policy` — stated at the capability's own
/// ceiling since there is only one real provider today and no weaker answer this crate
/// could honestly still act on.
fn Naming_Policy_Requirement() -> nomos_capability::Requirement
{
    return nomos_capability::Requirement::New(
        nomos_cap_naming_policy::Capability(),
        nomos_cap_naming_policy::CONTRACT_VERSION,
        nomos_cap_naming_policy::Ceiling(),
    );
}

/// The case `payload` declares for `symbol`, most-specific key first (`language`'s own
/// override, then the repository-wide row), or `None` when neither row exists.
fn Case_For_Symbol(payload: &NamingPolicyPayload, language: Option<&str>, symbol: &str) -> Option<Case>
{
    if let Some(language) = language
    {
        let scope = Scope::Language(language.to_owned());
        if let Some(row) = payload.rows.iter().find(|row| return row.scope == scope && row.symbol == symbol)
        {
            return Some(row.case);
        }
    }

    return payload.rows.iter().find(|row| return row.scope == Scope::Repository && row.symbol == symbol).map(|row| return row.case);
}

/// Judges every function `sources` declares against the case the repository declares for it, in
/// the language it is written in: on its side of visibility, and for a Go method as a method.
///
/// One fact per file, the same shape [`crate::Check_Completeness_Mirrors`] reads — this
/// rule states the same [`crate::Syntax_Requirement`] floor for the same reason: a name
/// spelled inside a comment or a string literal must not resolve as a real declaration,
/// and nothing weaker than a sound parse can promise that.
#[must_use]
pub fn Check_Naming_Convention(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    use reading::Payload_Of;
    use violations::Violations_In;

    let policy = Naming_Policy_Payload(facts, FUNCTION_CASE.family);
    let mut findings = Vec::new();

    for source in sources
    {
        let Some(cases) = Source_Cases(policy.as_ref(), source)
        else
        {
            continue;
        };
        match Payload_Of(source, facts)
        {
            Ok(payload) => findings.extend(Violations_In(&payload, &source.path, cases)),
            Err(finding) => findings.push(finding),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Judges the same function-name convention under the code-standards rule id.
#[must_use]
pub fn Check_Project_Owned_Function_Names_Use_Upper_Snake_Case(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let mut findings = Check_Naming_Convention(sources, facts);

    for finding in &mut findings
    {
        finding.rule = nomos_contracts::RuleId::New(PROJECT_OWNED_FUNCTION_NAMES_USE_UPPER_SNAKE_CASE);
        if finding.applicability == nomos_contracts::Applicability::Supported
        {
            finding.gate = nomos_contracts::GateCategory::Blocking;
        }
    }

    return findings;
}

/// [`Check_Naming_Convention`] itself, through a real registry, store and reader — the
/// half [`violations::tests`] does not reach, because a fact has to be required and
/// decoded before there is a payload to judge at all.
#[cfg(test)]
#[path = "naming/tests.rs"]
mod tests;

/// Narrow, file-local proofs for this file's own public functions, addressed by name.
///
/// [`tests`] above is `naming/tests.rs`, a separate physical file whose behavioural suite this
/// does not repeat or replace. `check-test-coverage`'s Rust front end keys a test's companion
/// unit off the literal file it is textually written in, so a test living in that separate file
/// can never address a function declared here, however it is named — this module gives
/// [`Check_Naming_Convention`] and [`Check_Project_Owned_Function_Names_Use_Upper_Snake_Case`]
/// the one-file address the check reads.
#[cfg(test)]
mod self_tests
{
    use super::*;
    use crate::checks::test_support::{self, OfferedProvider, Test_Context, TestOffering};
    use nomos_analysis::Reader;
    use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, SubjectId};
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_Naming_Convention_Should_Report_Under_Its_Own_Rule()
    {
        let findings = Findings_For(Check_Naming_Convention);

        assert_eq!(findings.len(), 1, "an unread subject must not render as a clean one: {findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, nomos_contracts::RuleId::New(NAMING_CONVENTION));
    }

    #[test]
    fn Test_Check_Project_Owned_Function_Names_Use_Upper_Snake_Case_Should_Report_Under_Its_Own_Rule()
    {
        let findings = Findings_For(Check_Project_Owned_Function_Names_Use_Upper_Snake_Case);

        assert_eq!(findings.len(), 1, "the same judgment, under the code-standards id: {findings:?}");
        assert_eq!(
            findings.first().expect("asserted len 1 above").rule,
            nomos_contracts::RuleId::New(PROJECT_OWNED_FUNCTION_NAMES_USE_UPPER_SNAKE_CASE)
        );
    }

    /// The spelling a repository writes in `standards.json`, and not a name this crate made up
    /// for it: the vocabulary a reader would declare a different case in.
    #[test]
    fn Test_Case_Judged_Against_Should_Name_A_Case_By_The_Spelling_A_Repository_Declares_It_In()
    {
        assert_eq!(Case_Judged_Against(Case::UpperSnake), "upper-snake case");
        assert_eq!(Case_Judged_Against(Case::MixedSnake), "mixed-snake case");
        assert_eq!(Case_Judged_Against(Case::LowerCamel), "lower-camel case");
    }

    /// What this file's own checks do with a subject no provider answered for.
    fn Findings_For(check: fn(&[SourceFile], &mut dyn FactReader) -> Vec<Finding>) -> Vec<Finding>
    {
        let path = "src/lib.rs";
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), "fn bad_name() {}");
        source.language = crate::Recognized_Language_In_Tests(path);

        let TestOffering { store, registry, .. } = Offering();
        let mut reader = Reader::On(&store, &registry, Test_Context());
        return check(&[source], &mut reader);
    }

    fn Offering() -> TestOffering
    {
        return test_support::Offered_Registry(
            OfferedProvider {
                contract: nomos_cap_syntax::Capability_Contract(),
                capability: nomos_cap_syntax::Capability(),
                version: nomos_cap_syntax::CONTRACT_VERSION,
                provider: "nomos.test.naming.self",
                guarantee: Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File),
            },
        ).expect("a fresh Registry holds neither this contract nor this provider");
    }
}
