//! Script discipline rules from code-standards.
//!
//! The repository-wide standard has more to say about scripts, including executable bits
//! and sourced shell libraries. This crate is handed source text, not file modes, so
//! [`Check_Scripts_Use_A_Portable_Shebang`] and [`Check_A_Script_Declares_Its_Purpose`]
//! judge the exact subset visible from a file's first lines: shebang portability and the
//! purpose comment immediately after the shebang/blank-line prefix.
//!
//! [`Check_Declared_Tooling_Language_For_Scripts`] is different in shape: it needs a
//! repository's own declaration, so it reads `nomos.cap.scripting.policy` — a third
//! `OD-RULES-011` instance, mirroring `checks::structure::Resolve_Limit` and `checks::
//! naming::Resolve_Case`, except its own capability's absence (or an undeclared tooling
//! language) resolves to *no findings at all* rather than a hardcoded prior value: this
//! rule never existed before this capability did, so there is no earlier default to fall
//! back to, and `check-script-discipline`'s own `spec.go` states the reason directly — a
//! repository that has not declared a tooling language has opted out, not defaulted in.

use crate::SourceFile;
use nomos_analysis::{FactReader, InputDigest};
use nomos_cap_scripting_policy::ScriptingPolicyPayload;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// The code-standards portable-shebang rule id.
pub const SCRIPTS_USE_A_PORTABLE_SHEBANG: &str = "scripts-use-a-portable-shebang";
/// The code-standards script-purpose rule id.
pub const A_SCRIPT_DECLARES_ITS_PURPOSE: &str = "a-script-declares-its-purpose";
/// The code-standards declared-tooling-language rule id.
pub const DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS: &str = "declared-tooling-language-for-scripts";

/// Reports shebang scripts whose first line hardcodes an interpreter path.
#[must_use]
pub fn Check_Scripts_Use_A_Portable_Shebang(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        let Some(first_line) = First_Line(source)
        else
        {
            continue;
        };

        if first_line.starts_with("#!") && !first_line.starts_with("#!/usr/bin/env ")
        {
            findings.push(Finding_For_Source(
                source,
                SCRIPTS_USE_A_PORTABLE_SHEBANG,
                "has a hardcoded shebang; use `#!/usr/bin/env <interpreter>`",
            ));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Reports shebang scripts whose first nonblank line after the shebang is not a comment.
#[must_use]
pub fn Check_A_Script_Declares_Its_Purpose(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if !Is_Shebang_Script(source)
        {
            continue;
        }

        if !Has_Purpose_Comment(source)
        {
            findings.push(Finding_For_Source(
                source,
                A_SCRIPT_DECLARES_ITS_PURPOSE,
                "does not declare its purpose in the first nonblank line after the shebang",
            ));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Reports files whose path ends in an extension the repository has declared forbidden,
/// once the repository has opted in by declaring its tooling language. Ported from
/// `check-script-discipline`'s own `Is_Forbidden`: the path alone convicts a file, nothing
/// is read.
#[must_use]
pub fn Check_Declared_Tooling_Language_For_Scripts(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let Some(policy) = Resolve_Scripting_Policy(facts)
    else
    {
        return Vec::new();
    };
    let Some(language) = &policy.tooling_language
    else
    {
        return Vec::new();
    };

    let mut findings = Vec::new();
    for source in sources
    {
        if Is_Forbidden_Extension(&source.path, &policy.forbidden_extensions)
        {
            findings.push(Finding_For_Source(
                source,
                DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS,
                &format!("is written in a language this repository's declared tooling language ({language}) does not use"),
            ));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// This crate's own floor for `nomos.cap.scripting.policy` — stated at the capability's
/// own ceiling since there is only one real provider today and no weaker answer this crate
/// could honestly still act on. Mirrors `checks::naming::Naming_Policy_Requirement` and
/// `checks::structure::Limits_Policy_Requirement` exactly, for the third sibling
/// capability.
fn Scripting_Policy_Requirement() -> nomos_capability::Requirement
{
    return nomos_capability::Requirement::New(
        nomos_cap_scripting_policy::Capability(),
        nomos_cap_scripting_policy::CONTRACT_VERSION,
        nomos_cap_scripting_policy::Ceiling(),
    );
}

/// Resolves a repository's own declared scripting policy — `None` on any `Require`
/// failure, per `OD-CAPABILITY-004`/`OD-RULES-011`'s settled optional-read pattern: this
/// capability is optional, and its absence must never surface as a `Finding` or this
/// capability's own `Applicability`.
fn Resolve_Scripting_Policy(facts: &mut dyn FactReader) -> Option<ScriptingPolicyPayload>
{
    let subject = nomos_model::Subject_Of_Path("");
    let fact = facts
        .Require(&nomos_cap_scripting_policy::Capability(), &subject, InputDigest::Of(&[]), &Scripting_Policy_Requirement())
        .ok()?;

    return nomos_cap_scripting_policy::Parse_Payload(&fact.payload.bytes).ok();
}

fn Is_Forbidden_Extension(path: &str, forbidden_extensions: &[String]) -> bool
{
    let lowered = path.to_lowercase();
    return forbidden_extensions
        .iter()
        .any(|extension| return lowered.ends_with(&extension.to_lowercase()));
}

fn First_Line(source: &SourceFile) -> Option<&str>
{
    return source.text.lines().next();
}

fn Is_Shebang_Script(source: &SourceFile) -> bool
{
    return First_Line(source).is_some_and(|line| return line.starts_with("#!"));
}

fn Has_Purpose_Comment(source: &SourceFile) -> bool
{
    return source
        .text
        .lines()
        .skip(1)
        .find(|line| return !line.trim().is_empty())
        .is_some_and(|line| return line.trim_start().starts_with('#'));
}

fn Finding_For_Source(source: &SourceFile, rule: &str, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(rule),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("{} {because}", source.path),
        locations: vec![source.path.clone()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::checks::test_support::{self, TestOffering};
    use nomos_analysis::{MemoryFactStore, Reader};
    use nomos_cap_scripting_policy::Encode_Payload;
    use nomos_capability::Registry;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_Scripts_Use_A_Portable_Shebang_Should_Report_A_Hardcoded_Shebang()
    {
        let source = Source("scripts/check.sh", "#!/bin/bash\n# check -- run checks\n");

        let findings = Check_Scripts_Use_A_Portable_Shebang(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(SCRIPTS_USE_A_PORTABLE_SHEBANG));
    }

    #[test]
    fn Test_Check_Scripts_Use_A_Portable_Shebang_Should_Accept_Env_Shebang()
    {
        let source = Source("scripts/check.sh", "#!/usr/bin/env bash\n# check -- run checks\n");

        let findings = Check_Scripts_Use_A_Portable_Shebang(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Script_Declares_Its_Purpose_Should_Report_Code_As_The_First_Content()
    {
        let source = Source("scripts/check.sh", "#!/usr/bin/env bash\n\nset -u\n");

        let findings = Check_A_Script_Declares_Its_Purpose(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(A_SCRIPT_DECLARES_ITS_PURPOSE));
    }

    #[test]
    fn Test_Check_A_Script_Declares_Its_Purpose_Should_Accept_A_Comment_After_Blanks()
    {
        let source = Source("scripts/check.sh", "#!/usr/bin/env bash\n\n# check -- run checks\nset -u\n");

        let findings = Check_A_Script_Declares_Its_Purpose(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Script_Declares_Its_Purpose_Should_Ignore_Non_Shebang_Files()
    {
        let source = Source("src/lib.rs", "fn Check() {}\n");

        let findings = Check_A_Script_Declares_Its_Purpose(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Declared_Tooling_Language_For_Scripts_Should_Report_No_Findings_When_The_Capability_Is_Unmaterialized()
    {
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());
        let source = Source("tooling/deploy.ps1", "");

        let findings = Check_Declared_Tooling_Language_For_Scripts(&[source], &mut facts);

        assert!(findings.is_empty(), "an unconfigured repository must not be judged: {findings:?}");
    }

    #[test]
    fn Test_Check_Declared_Tooling_Language_For_Scripts_Should_Report_No_Findings_When_No_Language_Is_Declared()
    {
        let TestOffering { mut store, registry, offer } = Scripting_Offering();
        Materialize_Scripting_Fact(&mut store, &offer, None, vec![".ps1".to_owned()]);
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());
        let source = Source("tooling/deploy.ps1", "");

        let findings = Check_Declared_Tooling_Language_For_Scripts(&[source], &mut facts);

        assert!(findings.is_empty(), "an opted-out repository must not be judged: {findings:?}");
    }

    #[test]
    fn Test_Check_Declared_Tooling_Language_For_Scripts_Should_Report_A_Forbidden_Extension()
    {
        let TestOffering { mut store, registry, offer } = Scripting_Offering();
        Materialize_Scripting_Fact(&mut store, &offer, Some("rust".to_owned()), vec![".ps1".to_owned(), ".sh".to_owned()]);
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());
        let source = Source("tooling/deploy.ps1", "");

        let findings = Check_Declared_Tooling_Language_For_Scripts(&[source], &mut facts);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New(DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS));
        assert!(found.summary.contains("rust"), "{}", found.summary);
    }

    #[test]
    fn Test_Check_Declared_Tooling_Language_For_Scripts_Should_Accept_A_File_Outside_The_Forbidden_List()
    {
        let TestOffering { mut store, registry, offer } = Scripting_Offering();
        Materialize_Scripting_Fact(&mut store, &offer, Some("rust".to_owned()), vec![".ps1".to_owned()]);
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());
        let source = Source("scripts/setup.sh", "");

        let findings = Check_Declared_Tooling_Language_For_Scripts(&[source], &mut facts);

        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Scripting_Offering() -> TestOffering
    {
        return test_support::Offering(
            nomos_cap_scripting_policy::Capability_Contract(),
            nomos_cap_scripting_policy::Capability(),
            nomos_cap_scripting_policy::CONTRACT_VERSION,
            "nomos.test.scripting.provides",
            nomos_cap_scripting_policy::Ceiling(),
        );
    }

    fn Materialize_Scripting_Fact(
        store: &mut MemoryFactStore,
        offer: &nomos_capability::ProviderOffer,
        tooling_language: Option<String>,
        forbidden_extensions: Vec<String>,
    )
    {
        let payload = ScriptingPolicyPayload { tooling_language, forbidden_extensions };
        test_support::Materialize(
            store,
            nomos_model::Subject_Of_Path(""),
            offer,
            InputDigest::Of(&[]),
            nomos_cap_scripting_policy::Payload_Schema(),
            Encode_Payload(&payload),
        );
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        return SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
    }
}
