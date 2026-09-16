//! `script_discipline.rs`'s policy-driven rule: a repository that has declared its tooling
//! language forbids the extensions it named.

use super::{Because, DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS, Finding_For_Source, Rule};
use crate::SourceFile;
use nomos_analysis::{FactReader, InputDigest};
use nomos_cap_scripting_policy::ScriptingPolicyPayload;
use nomos_contracts::Finding;

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

    let mut findings = Forbidden_Extension_Findings(sources, &policy, language);

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
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

fn Forbidden_Extension_Findings(sources: &[SourceFile], policy: &ScriptingPolicyPayload, language: &str) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if Is_Forbidden_Extension(&source.path, &policy.forbidden_extensions)
        {
            let because = format!("is written in a language this repository's declared tooling language ({language}) does not use");
            let finding = Finding_For_Source(source, Rule(DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS), Because(&because));
            findings.push(finding);
        }
    }

    return findings;
}

fn Is_Forbidden_Extension(path: &str, forbidden_extensions: &[String]) -> bool
{
    let lowered = path.to_lowercase();
    return forbidden_extensions
        .iter()
        .any(|extension| return lowered.ends_with(&extension.to_lowercase()));
}

#[cfg(test)]
mod tests
{
    use super::super::tests::{Source, SourceText};
    use super::*;
    use crate::checks::test_support::{self, FactToFile, OfferedProvider, TestOffering};
    use nomos_analysis::{MemoryFactStore, Reader};
    use nomos_cap_scripting_policy::Encode_Payload;
    use nomos_capability::Registry;
    use nomos_contracts::RuleId;

    #[test]
    fn Test_Check_Declared_Tooling_Language_For_Scripts_Should_Report_No_Findings_When_The_Capability_Is_Unmaterialized()
    {
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());
        let source = Source(SourceText { path: "tooling/deploy.ps1", text: "" });

        let findings = Check_Declared_Tooling_Language_For_Scripts(&[source], &mut facts);

        assert!(findings.is_empty(), "an unconfigured repository must not be judged: {findings:?}");
    }

    #[test]
    fn Test_Check_Declared_Tooling_Language_For_Scripts_Should_Report_No_Findings_When_No_Language_Is_Declared()
    {
        let TestOffering { mut store, registry, offer } = Scripting_Offering();
        Materialize_Scripting_Fact(&mut store, &offer, None, vec![".ps1".to_owned()]);
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());
        let source = Source(SourceText { path: "tooling/deploy.ps1", text: "" });

        let findings = Check_Declared_Tooling_Language_For_Scripts(&[source], &mut facts);

        assert!(findings.is_empty(), "an opted-out repository must not be judged: {findings:?}");
    }

    #[test]
    fn Test_Check_Declared_Tooling_Language_For_Scripts_Should_Report_A_Forbidden_Extension()
    {
        let TestOffering { mut store, registry, offer } = Scripting_Offering();
        Materialize_Scripting_Fact(&mut store, &offer, Some("rust".to_owned()), vec![".ps1".to_owned(), ".sh".to_owned()]);
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());
        let source = Source(SourceText { path: "tooling/deploy.ps1", text: "" });

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
        let source = Source(SourceText { path: "scripts/setup.sh", text: "" });

        let findings = Check_Declared_Tooling_Language_For_Scripts(&[source], &mut facts);

        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Scripting_Offering() -> TestOffering
    {
        return test_support::Offered_Registry(
            OfferedProvider {
                contract: nomos_cap_scripting_policy::Capability_Contract(),
                capability: nomos_cap_scripting_policy::Capability(),
                version: nomos_cap_scripting_policy::CONTRACT_VERSION,
                provider: "nomos.test.scripting.provides",
                guarantee: nomos_cap_scripting_policy::Ceiling(),
            },
        ).expect("a fresh Registry holds neither this contract nor this provider");
    }

    fn Materialize_Scripting_Fact(
        store: &mut MemoryFactStore,
        offer: &nomos_capability::ProviderOffer,
        tooling_language: Option<String>,
        forbidden_extensions: Vec<String>,
    )
    {
        let payload = ScriptingPolicyPayload { tooling_language, forbidden_extensions };
        test_support::Materialize_Fact(
            store,
            FactToFile {
                subject: nomos_model::Subject_Of_Path(""),
                offer,
                semantic_inputs: InputDigest::Of(&[]),
                schema: nomos_cap_scripting_policy::Payload_Schema(),
                bytes: Encode_Payload(&payload),
            },
        ).expect("the fixture's store holds no fact under this key at a newer generation");
    }
}
