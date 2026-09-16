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

mod abbreviations;
mod boolean_predicates;
mod clarity;
mod data_names;
mod file_names;
mod go;
pub(super) mod reading;
mod single_letter_names;
mod test_names;
mod violations;

use crate::SourceFile;
use nomos_analysis::{FactReader, InputDigest};
use nomos_cap_naming_policy::{Case, Scope};
use nomos_contracts::Finding;

pub use abbreviations::{Check_Abbreviations, ABBREVIATIONS};
pub use boolean_predicates::{Check_Boolean_Predicates, BOOLEAN_PREDICATES};
pub use clarity::{Check_Naming_Clarity, NAMING_CLARITY};
pub use data_names::{Check_Data_Names_Stay_Lower_Snake, DATA_NAMES_STAY_LOWER_SNAKE};
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

/// Resolves the case `symbol` must take: a repository's own declared `nomos.cap.naming.
/// policy`, most-specific key first (`language`'s own override, then the repository-wide
/// default), falling back to `default` when neither is declared.
///
/// `OD-CAPABILITY-004` and `OD-RULES-011` settle how an absent read is treated here: this
/// capability is optional, every caller already has a complete answer without it, so
/// `facts.Require` failing for any reason is exactly "no override" — never a `Finding`,
/// never this capability's own `Applicability` surfacing anywhere.
pub(super) fn Resolve_Case(facts: &mut dyn FactReader, language: Option<&str>, symbol: &str, default: Case) -> Case
{
    let Some(payload) = Naming_Policy_Payload(facts)
    else
    {
        return default;
    };

    return Case_For_Symbol(&payload, language, symbol, default);
}

/// Reads and decodes this repository's own `nomos.cap.naming.policy` fact, folding every
/// way the read can come back empty (absent, refused, malformed) into `None` — `Resolve_
/// Case`'s caller already has a complete default for that case, per `OD-CAPABILITY-004`.
fn Naming_Policy_Payload(facts: &mut dyn FactReader) -> Option<nomos_cap_naming_policy::NamingPolicyPayload>
{
    let subject = nomos_model::Subject_Of_Path("");
    let fact = facts
        .Require(&nomos_cap_naming_policy::Capability(), &subject, InputDigest::Of(&[]), &Naming_Policy_Requirement())
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
/// override, then the repository-wide default), or `default` when neither row exists.
fn Case_For_Symbol(payload: &nomos_cap_naming_policy::NamingPolicyPayload, language: Option<&str>, symbol: &str, default: Case) -> Case
{
    if let Some(language) = language
    {
        let scope = Scope::Language(language.to_owned());
        if let Some(row) = payload.rows.iter().find(|row| return row.scope == scope && row.symbol == symbol)
        {
            return row.case;
        }
    }

    if let Some(row) = payload.rows.iter().find(|row| return row.scope == Scope::Repository && row.symbol == symbol)
    {
        return row.case;
    }

    return default;
}

/// Judges every function `sources` declares against the workspace's naming convention.
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

    let case = Resolve_Case(facts, None, "function", Case::UpperSnake);
    let mut findings = Vec::new();

    for source in sources
    {
        match Payload_Of(source, facts)
        {
            Ok(payload) =>
            {
                let violations = Violations_In(&payload, &source.path, case);
                findings.extend(violations);
            }
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
        return test_support::Offering(
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
