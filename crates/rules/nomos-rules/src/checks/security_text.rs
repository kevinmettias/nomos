//! Language-agnostic security text patterns from code-standards.
//!
//! Unlike `rust_text.rs`/`go_text.rs`, none of these three names a `language:` — each is
//! decidable from raw text against a fixed, narrow pattern set the standard's own doc
//! bounds explicitly, in any source file. None of the three *patterns* has a
//! repository-configurable dimension — a credential prefix, a sensitive URL parameter
//! name, or a disabled-verification literal is not a house-style choice. What each does
//! have, shared with the rest of this crate, is a repository-configurable *exemption*:
//! what counts as test material is read from `nomos.cap.test.material.policy`, composed
//! with this module's own toolchain-fixed clauses, rather than compiled in alone. So this
//! is a shared module that reads one capability, not a capability itself.
//!
//! Each rule states its own exact scope in its doc comment below, because each is
//! deliberately narrower than its title suggests: entropy-based secret scanning,
//! semantic URL analysis and runtime-value tracking are named out of scope by the
//! standards themselves and owned by a dedicated tool instead.

mod certificate_verification;

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

pub use certificate_verification::Check_Certificate_Verification_Is_Not_Disabled;

/// The code-standards hardcoded-credential rule id.
pub const A_CREDENTIAL_IS_NOT_HARDCODED_IN_SOURCE: &str = "a-credential-is-not-hardcoded-in-source";
/// The code-standards secret-in-url rule id.
pub const A_SECRET_DOES_NOT_TRAVEL_IN_A_URL: &str = "a-secret-does-not-travel-in-a-url";
/// The code-standards disabled-TLS-verification rule id.
pub const CERTIFICATE_VERIFICATION_IS_NOT_DISABLED: &str = "certificate-verification-is-not-disabled";

/// A provider-format credential prefix and the minimum run of credential-body characters
/// (alphanumeric) that must immediately follow it — long enough to separate a real key
/// from a bare mention of the prefix, short of the provider's own exact length so a
/// slightly different real key still matches.
const CREDENTIAL_PREFIXES: &[(&str, usize)] = &[
    ("AKIA", 16),   // AWS access key id
    ("ASIA", 16),   // AWS temporary access key id
    ("ghp_", 20),   // GitHub personal access token
    ("github_pat_", 20),
    ("xoxb-", 10), // Slack bot token
    ("xoxp-", 10),
    ("xoxa-", 10),
    ("xoxs-", 10),
    ("AIza", 20),   // Google API key
    ("ya29.", 20),  // Google OAuth access token
    ("sk_live_", 16), // Stripe live secret key -- sk_test_ is deliberately absent, per the standard
    ("rk_live_", 16), // Stripe live restricted key
    ("npm_", 20),   // npm token
];

/// Provider-documented example/placeholder credentials the standard names explicitly as
/// not real secrets.
const DOCUMENTED_EXAMPLE_CREDENTIALS: &[&str] = &["AKIAIOSFODNN7EXAMPLE"];

/// URL query-parameter names the standard names as carrying a secret when woven into a
/// path or query string.
const SENSITIVE_URL_PARAMETERS: &[&str] = &["api_key", "access_token", "token", "password", "sig", "signature"];

/// Reports a string literal whose text is a provider-minted credential: an AWS access key,
/// a GitHub/Slack/Google/Stripe/npm token, or a PEM private-key block. Does not attempt
/// the entropy-based scan of arbitrary high-entropy strings the standard names as a
/// dedicated secret scanner's own job, and skips test/fixture/example sources, where the
/// standard says example values and fixtures are expected to live.
#[must_use]
pub fn Check_A_Credential_Is_Not_Hardcoded_In_Source(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let declared = crate::checks::Resolve_Declared_Fixture_Locations(facts);
    let mut findings = Vec::new();
    for source in sources
    {
        if !Is_Test_Or_Fixture_Source(source, &declared) && !Is_Own_Implementation_File(source)
        {
            findings.extend(Credential_Findings_In(source));
        }
    }
    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Credential_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for (index, line) in source.text.lines().enumerate()
    {
        if let Some(matched) = Credential_Match_In(line)
        {
            let summary = format!("carries what looks like a live credential (`{matched}`) in source");
            let finding = Finding_For_Line(source, A_CREDENTIAL_IS_NOT_HARDCODED_IN_SOURCE, Line_Number(index), &summary);
            findings.push(finding);
        }
    }

    return findings;
}

fn Credential_Match_In(line: &str) -> Option<String>
{
    if line.contains("-----BEGIN") && line.contains("PRIVATE KEY-----")
    {
        return Some("PEM private-key block".to_owned());
    }

    for (prefix, minimum_trailing) in CREDENTIAL_PREFIXES
    {
        if let Some(matched) = Prefixed_Credential_In(line, Prefix(prefix), *minimum_trailing)
        {
            return Some(matched);
        }
    }

    return None;
}

/// The literal credential prefix [`Prefixed_Credential_In`] searches for, wrapped so its
/// parameter position cannot be transposed with `line` — the text being searched — with
/// nothing to catch it.
struct Prefix<'a>(&'a str);

/// Searches `line` for every occurrence of `prefix`, returning the first match whose
/// trailing alphanumeric run meets `minimum_trailing` and is not a documented example.
fn Prefixed_Credential_In(line: &str, prefix: Prefix<'_>, minimum_trailing: usize) -> Option<String>
{
    let mut search_from = 0usize;

    while let Some(offset) = line.get(search_from..).and_then(|rest| return rest.find(prefix.0))
    {
        let start = search_from.saturating_add(offset);

        if let Some(matched) = Credential_At(line, start, prefix.0, minimum_trailing)
        {
            return Some(matched);
        }

        search_from = start.saturating_add(prefix.0.len());
    }

    return None;
}

/// The credential text at `start` if `prefix`'s trailing alphanumeric run there meets
/// `minimum_trailing` and is not one of [`DOCUMENTED_EXAMPLE_CREDENTIALS`].
fn Credential_At(line: &str, start: usize, prefix: &str, minimum_trailing: usize) -> Option<String>
{
    let after_prefix = line.get(start.saturating_add(prefix.len())..).unwrap_or("");
    let trailing_run: usize = after_prefix.chars().take_while(|character| return character.is_ascii_alphanumeric()).count();
    if trailing_run < minimum_trailing
    {
        return None;
    }

    let matched_len = prefix.len().saturating_add(trailing_run);
    let matched = line.get(start..start.saturating_add(matched_len)).unwrap_or(prefix);
    if DOCUMENTED_EXAMPLE_CREDENTIALS.contains(&matched)
    {
        return None;
    }

    return Some(matched.to_owned());
}

/// Reports a named sensitive parameter (`api_key`, `token`, `access_token`, `password`,
/// `sig`, `signature`) woven into a URL's query string. Does not attempt to tell a
/// deliberate, short-lived pre-signed URL from a hardcoded one — the standard names that
/// distinction as the consuming system's call, not the parser's — and skips test/fixture
/// sources.
#[must_use]
pub fn Check_A_Secret_Does_Not_Travel_In_A_Url(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let declared = crate::checks::Resolve_Declared_Fixture_Locations(facts);
    let mut findings = Vec::new();
    for source in sources
    {
        if !Is_Test_Or_Fixture_Source(source, &declared) && !Is_Own_Implementation_File(source)
        {
            findings.extend(Url_Secret_Findings_In(source));
        }
    }
    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Url_Secret_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for (index, line) in source.text.lines().enumerate()
    {
        if Has_Secret_In_Url(line)
        {
            let finding = Finding_For_Line(
                source,
                A_SECRET_DOES_NOT_TRAVEL_IN_A_URL,
                Line_Number(index),
                "weaves a named secret parameter into a URL's query string",
            );
            findings.push(finding);
        }
    }

    return findings;
}

/// `?api_key=`/`&token=`-shaped: a query separator immediately followed by a named
/// sensitive parameter and `=`, the exact join a real query string produces.
fn Has_Secret_In_Url(line: &str) -> bool
{
    for parameter in SENSITIVE_URL_PARAMETERS
    {
        for separator in ['?', '&']
        {
            if line.contains(&format!("{separator}{parameter}=")) || line.contains(&format!("{separator}{parameter}%3D"))
            {
                return true;
            }
        }
    }

    return false;
}

/// A path segment any of `tests/`, `/test/`, `testdata/`, `fixtures/`, `examples/`
/// contains, or a `_test.`/`_tests.` file-name suffix — the standard's own "example
/// values, fixtures, and keys that live in test files" exemption, read broadly enough to
/// cover Go's `testdata/` and either language's fixture convention — or a repository's own
/// declared fixture location, the same additions [`super::Resolve_Declared_Fixture_Locations`]
/// hands every other test-or-example predicate in this crate.
fn Is_Test_Or_Fixture_Source(source: &SourceFile, declared: &[String]) -> bool
{
    let normalized = source.path.replace('\\', "/");
    return normalized.starts_with("tests/")
        || normalized.contains("/tests/")
        || normalized.contains("/test/")
        || normalized.contains("/testdata/")
        || normalized.contains("/fixtures/")
        || normalized.contains("/examples/")
        || normalized.ends_with("_test.rs")
        || normalized.ends_with("_tests.rs")
        || normalized.ends_with("_test.go")
        || declared.iter().any(|location| return crate::checks::Is_Under_Declared_Location(&normalized, location));
}

/// Every file this module is written in. Every rule here exempts its own implementing
/// files, the same self-exemption `rust_text.rs`'s syntax-shaped rules carry: each file
/// holds its own test fixtures and each rule's own detection-pattern constants
/// (`CREDENTIAL_PREFIXES`, the PEM-block markers, `SENSITIVE_URL_PARAMETERS`, the
/// disabled-verification literals), which necessarily spell out the exact values the rule
/// in that file looks for. Unlike `rust_text.rs`'s syntax-shaped rules, these three are
/// content-shaped -- the violation *is* a string's text, so a general
/// string-literal-stripping fix would defeat every one of these rules everywhere, not just
/// here; a self-file exemption is the only correct fix for this module. A LIST rather than
/// one path, because the split into `certificate_verification.rs` moved one rule's literals
/// into a file of its own, and a rule that only recognised the parent would flag its own
/// detector. Checked safe today: these files' own doc comments and detection code contain no
/// genuine live credential, sensitive-parameter URL, or disabled-verification literal outside
/// their own patterns and fixtures.
const OWN_IMPLEMENTATION_FILES: &[&str] = &[
    "crates/rules/nomos-rules/src/checks/security_text.rs",
    "crates/rules/nomos-rules/src/checks/security_text/certificate_verification.rs",
];

fn Is_Own_Implementation_File(source: &SourceFile) -> bool
{
    let normalized = source.path.replace('\\', "/");
    return OWN_IMPLEMENTATION_FILES.contains(&normalized.as_str());
}

fn Line_Number(index: usize) -> usize
{
    return index.saturating_add(1);
}

fn Finding_For_Line(source: &SourceFile, rule: &str, line_number: usize, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(rule),
        subject: source.subject,
        subject_name: format!("{}:{line_number}", source.path),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("{} line {line_number} {because}", source.path),
        locations: vec![format!("{}:{line_number}", source.path)],
    };
}

#[cfg(test)]
#[path = "security_text/tests.rs"]
mod tests;

/// Narrow, file-local proofs for this file's own public functions, addressed by name.
///
/// [`tests`] above is `security_text/tests.rs`, a separate physical file whose behavioural
/// suite this does not repeat or replace. `check-test-coverage`'s Rust front end keys a test's
/// companion unit off the literal file it is textually written in, so a test living in that
/// separate file can never address a function declared here, however it is named — this module
/// gives [`Check_A_Credential_Is_Not_Hardcoded_In_Source`] and
/// [`Check_A_Secret_Does_Not_Travel_In_A_Url`] the one-file address the check reads.
///
/// Each fixture is assembled rather than written out, for the same reason
/// `security_text/tests.rs` records in full: this file must not itself carry a secret-shaped
/// literal while the detector is still handed exactly the text it always was.
#[cfg(test)]
mod self_tests
{
    use super::*;
    use nomos_analysis::{MemoryFactStore, Reader};
    use nomos_capability::Registry;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_A_Credential_Is_Not_Hardcoded_In_Source_Should_Report_A_Prefixed_Credential()
    {
        let secret = format!("AKIA{}", "ABCDEFGHIJKLMNOP");
        let text = format!("const KEY: &str = \"{secret}\";\n");

        let findings = Check(Check_A_Credential_Is_Not_Hardcoded_In_Source, Source_For(Path("src/config.rs"), Text(&text)));

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(
            findings.first().expect("asserted len 1 above").rule,
            RuleId::New(A_CREDENTIAL_IS_NOT_HARDCODED_IN_SOURCE)
        );
    }

    #[test]
    fn Test_Check_A_Secret_Does_Not_Travel_In_A_Url_Should_Report_A_Named_Secret_Parameter()
    {
        let text = format!("let url = format!(\"https://api.example.com/data?{}={{key}}\");\n", "api_key");

        let findings = Check(Check_A_Secret_Does_Not_Travel_In_A_Url, Source_For(Path("src/client.rs"), Text(&text)));

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(
            findings.first().expect("asserted len 1 above").rule,
            RuleId::New(A_SECRET_DOES_NOT_TRAVEL_IN_A_URL)
        );
    }

    /// `path` and `text` are both `&str`; without a distinct type per position a call site like
    /// `Source_For("src/config.rs", "…")` reads as two interchangeable strings and a swap
    /// compiles silently. These wrappers give each position a type the other cannot satisfy.
    #[derive(Clone, Copy)]
    struct Path<'a>(&'a str);

    #[derive(Clone, Copy)]
    struct Text<'a>(&'a str);

    fn Source_For(path: Path<'_>, text: Text<'_>) -> SourceFile
    {
        let mut source = SourceFile::New(path.0, SubjectId::From_Digest(Content_Digest(path.0.as_bytes())), text.0);
        source.language = crate::Recognized_Language_In_Tests(path.0);
        return source;
    }

    /// Runs `check` over `source` through a real, empty reader — the three checks read only
    /// `nomos.cap.test.material.policy`, which no fixture here declares, so `Require` fails
    /// and each resolves to its own fixed clauses alone.
    fn Check(check: fn(&[SourceFile], &mut dyn FactReader) -> Vec<Finding>, source: SourceFile) -> Vec<Finding>
    {
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let mut facts = Reader::On(&store, &registry, crate::checks::test_support::Test_Context());
        return check(&[source], &mut facts);
    }
}
