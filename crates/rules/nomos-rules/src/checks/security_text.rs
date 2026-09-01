//! Language-agnostic security text patterns from code-standards.
//!
//! Unlike `rust_text.rs`/`go_text.rs`, none of these three names a `language:` — each is
//! decidable from raw text against a fixed, narrow pattern set the standard's own doc
//! bounds explicitly, in any source file. None has a repository-configurable dimension
//! (a credential prefix, a sensitive URL parameter name, or a disabled-verification
//! literal is not a house-style choice), so this is a shared module, not a capability.
//!
//! Each rule states its own exact scope in its doc comment below, because each is
//! deliberately narrower than its title suggests: entropy-based secret scanning,
//! semantic URL analysis and runtime-value tracking are named out of scope by the
//! standards themselves and owned by a dedicated tool instead.

use crate::SourceFile;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

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
const SENSITIVE_URL_PARAMS: &[&str] = &["api_key", "access_token", "token", "password", "sig", "signature"];

/// Reports a string literal whose text is a provider-minted credential: an AWS access key,
/// a GitHub/Slack/Google/Stripe/npm token, or a PEM private-key block. Does not attempt
/// the entropy-based scan of arbitrary high-entropy strings the standard names as a
/// dedicated secret scanner's own job, and skips test/fixture/example sources, where the
/// standard says example values and fixtures are expected to live.
#[must_use]
pub fn Check_A_Credential_Is_Not_Hardcoded_In_Source(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();
    for source in sources
    {
        if !Is_Test_Or_Fixture_Source(source)
        {
            findings.extend(Credential_Findings_In(source));
        }
    }
    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Reports a named sensitive parameter (`api_key`, `token`, `access_token`, `password`,
/// `sig`, `signature`) woven into a URL's query string. Does not attempt to tell a
/// deliberate, short-lived pre-signed URL from a hardcoded one — the standard names that
/// distinction as the consuming system's call, not the parser's — and skips test/fixture
/// sources.
#[must_use]
pub fn Check_A_Secret_Does_Not_Travel_In_A_Url(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();
    for source in sources
    {
        if !Is_Test_Or_Fixture_Source(source)
        {
            findings.extend(Url_Secret_Findings_In(source));
        }
    }
    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Reports a TLS/SSL client or server told, by a literal value, to skip verifying the
/// peer's certificate: `InsecureSkipVerify` set to the literal `true`, `.danger_accept_
/// invalid_certs(true)`, `SslVerifyMode::NONE`, or `verify` set to the literal `False`.
/// Does not flag a value set from a variable or config flag — the standard is explicit
/// that whether that path disables verification is not a fact the source states — and
/// accepts either an adjacent explanatory comment (a reasoned, recorded exception) or a
/// test/fixture source, both of which the standard names as exempt.
#[must_use]
pub fn Check_Certificate_Verification_Is_Not_Disabled(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();
    for source in sources
    {
        if !Is_Test_Or_Fixture_Source(source)
        {
            findings.extend(Tls_Verification_Findings_In(source));
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
            findings.push(Finding_For_Line(
                source,
                A_CREDENTIAL_IS_NOT_HARDCODED_IN_SOURCE,
                Line_Number(index),
                &format!("carries what looks like a live credential (`{matched}`) in source"),
            ));
        }
    }

    return findings;
}

fn Url_Secret_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for (index, line) in source.text.lines().enumerate()
    {
        if Has_Secret_In_Url(line)
        {
            findings.push(Finding_For_Line(
                source,
                A_SECRET_DOES_NOT_TRAVEL_IN_A_URL,
                Line_Number(index),
                "weaves a named secret parameter into a URL's query string",
            ));
        }
    }

    return findings;
}

fn Tls_Verification_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let lines: Vec<&str> = source.text.lines().collect();
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        if Has_Disabled_Certificate_Verification(line) && !Has_Adjacent_Explanation(&lines, index)
        {
            findings.push(Finding_For_Line(
                source,
                CERTIFICATE_VERIFICATION_IS_NOT_DISABLED,
                Line_Number(index),
                "disables TLS peer-certificate verification with no adjacent comment recording why",
            ));
        }
    }

    return findings;
}

/// A path segment any of `tests/`, `/test/`, `testdata/`, `fixtures/`, `examples/`
/// contains, or a `_test.`/`_tests.` file-name suffix — the standard's own "example
/// values, fixtures, and keys that live in test files" exemption, read broadly enough to
/// cover Go's `testdata/` and either language's fixture convention.
fn Is_Test_Or_Fixture_Source(source: &SourceFile) -> bool
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
        || normalized.ends_with("_test.go");
}

fn Line_Number(index: usize) -> usize
{
    return index.saturating_add(1);
}

fn Credential_Match_In(line: &str) -> Option<String>
{
    if line.contains("-----BEGIN") && line.contains("PRIVATE KEY-----")
    {
        return Some("PEM private-key block".to_owned());
    }

    for (prefix, minimum_trailing) in CREDENTIAL_PREFIXES
    {
        let mut search_from = 0usize;
        while let Some(offset) = line.get(search_from..).and_then(|rest| return rest.find(prefix))
        {
            let start = search_from.saturating_add(offset);
            let after_prefix = line.get(start.saturating_add(prefix.len())..).unwrap_or("");
            let trailing_run: usize = after_prefix.chars().take_while(|character| return character.is_ascii_alphanumeric()).count();

            if trailing_run >= *minimum_trailing
            {
                let matched_len = prefix.len().saturating_add(trailing_run);
                let matched = line.get(start..start.saturating_add(matched_len)).unwrap_or(prefix);
                if !DOCUMENTED_EXAMPLE_CREDENTIALS.contains(&matched)
                {
                    return Some(matched.to_owned());
                }
            }

            search_from = start.saturating_add(prefix.len());
        }
    }

    return None;
}

/// `?api_key=`/`&token=`-shaped: a query separator immediately followed by a named
/// sensitive parameter and `=`, the exact join a real query string produces.
fn Has_Secret_In_Url(line: &str) -> bool
{
    for parameter in SENSITIVE_URL_PARAMS
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

fn Has_Disabled_Certificate_Verification(line: &str) -> bool
{
    let compact: String = line.chars().filter(|character| return !character.is_whitespace()).collect();

    return compact.contains("InsecureSkipVerify:true")
        || compact.contains("InsecureSkipVerify=true")
        || line.contains(".danger_accept_invalid_certs(true)")
        || line.contains("SslVerifyMode::NONE")
        || compact.contains("verify=False")
        || compact.contains("verify:False");
}

fn Comment_Text_Of(line: &str) -> Option<&str>
{
    let trimmed = line.trim_start();
    for marker in ["//", "#"]
    {
        if let Some(comment) = trimmed.strip_prefix(marker)
        {
            return Some(comment.trim_start());
        }
    }
    return None;
}

/// A trailing same-line comment with non-empty text, or a non-empty comment on the line
/// immediately above — "the reasoned exception is recorded rather than hidden," the same
/// adjacent-explanation shape `rust_text.rs`/`go_text.rs` already use, generalized to a
/// third consumer.
fn Has_Adjacent_Explanation(lines: &[&str], index: usize) -> bool
{
    if let Some(reason) = lines
        .get(index)
        .and_then(|line| return line.split_once("//").or_else(|| return line.split_once('#')).map(|(_, rest)| return rest))
    {
        if !reason.trim().is_empty()
        {
            return true;
        }
    }

    if let Some(previous) = index.checked_sub(1)
    {
        if lines.get(previous).is_some_and(|line| return Comment_Text_Of(line).is_some_and(|c| return !c.trim().is_empty()))
        {
            return true;
        }
    }

    return false;
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
mod tests
{
    use super::*;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_A_Credential_Is_Not_Hardcoded_In_Source_Should_Report_An_Aws_Access_Key()
    {
        let source = Source("src/config.rs", "const KEY: &str = \"AKIAABCDEFGHIJKLMNOP\";\n");
        let findings = Check_A_Credential_Is_Not_Hardcoded_In_Source(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(A_CREDENTIAL_IS_NOT_HARDCODED_IN_SOURCE));
    }

    #[test]
    fn Test_Check_A_Credential_Is_Not_Hardcoded_In_Source_Should_Ignore_The_Documented_Example_Key()
    {
        let source = Source("src/config.rs", "const EXAMPLE: &str = \"AKIAIOSFODNN7EXAMPLE\";\n");
        let findings = Check_A_Credential_Is_Not_Hardcoded_In_Source(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Credential_Is_Not_Hardcoded_In_Source_Should_Ignore_A_Stripe_Test_Key()
    {
        let source = Source("src/config.rs", "const KEY: &str = \"sk_test_ABCDEFGHIJKLMNOPQRST\";\n");
        let findings = Check_A_Credential_Is_Not_Hardcoded_In_Source(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Credential_Is_Not_Hardcoded_In_Source_Should_Report_A_Stripe_Live_Key()
    {
        let source = Source("src/config.rs", "const KEY: &str = \"sk_live_ABCDEFGHIJKLMNOPQRST\";\n");
        let findings = Check_A_Credential_Is_Not_Hardcoded_In_Source(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Credential_Is_Not_Hardcoded_In_Source_Should_Report_A_Pem_Private_Key_Header()
    {
        let source = Source("src/config.rs", "-----BEGIN RSA PRIVATE KEY-----\n");
        let findings = Check_A_Credential_Is_Not_Hardcoded_In_Source(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Credential_Is_Not_Hardcoded_In_Source_Should_Ignore_A_Resource_Id()
    {
        let source = Source("src/config.rs", "const RESOURCE_ID: &str = \"a1b2c3d4-e5f6-7890\";\n");
        let findings = Check_A_Credential_Is_Not_Hardcoded_In_Source(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Credential_Is_Not_Hardcoded_In_Source_Should_Ignore_Test_Files()
    {
        let source = Source("tests/fixtures.rs", "const KEY: &str = \"sk_live_ABCDEFGHIJKLMNOPQRST\";\n");
        let findings = Check_A_Credential_Is_Not_Hardcoded_In_Source(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Secret_Does_Not_Travel_In_A_Url_Should_Report_An_Api_Key_Query_Param()
    {
        let source = Source("src/client.rs", "let url = format!(\"https://api.example.com/data?api_key={key}\");\n");
        let findings = Check_A_Secret_Does_Not_Travel_In_A_Url(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Secret_Does_Not_Travel_In_A_Url_Should_Report_A_Second_Position_Token_Param()
    {
        let source = Source("src/client.rs", "let url = format!(\"https://api.example.com/data?page=2&token={t}\");\n");
        let findings = Check_A_Secret_Does_Not_Travel_In_A_Url(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Secret_Does_Not_Travel_In_A_Url_Should_Ignore_A_Non_Secret_Param()
    {
        let source = Source("src/client.rs", "let url = format!(\"https://api.example.com/data?page={n}&limit=20\");\n");
        let findings = Check_A_Secret_Does_Not_Travel_In_A_Url(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Certificate_Verification_Is_Not_Disabled_Should_Report_Insecure_Skip_Verify_True()
    {
        let source = Source("src/client.go", "tls.Config{InsecureSkipVerify: true}\n");
        let findings = Check_Certificate_Verification_Is_Not_Disabled(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Certificate_Verification_Is_Not_Disabled_Should_Report_Danger_Accept_Invalid_Certs()
    {
        let source = Source("src/client.rs", "let client = Client::builder().danger_accept_invalid_certs(true).build()?;\n");
        let findings = Check_Certificate_Verification_Is_Not_Disabled(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Certificate_Verification_Is_Not_Disabled_Should_Ignore_A_Variable_Value()
    {
        let source = Source("src/client.go", "tls.Config{InsecureSkipVerify: cfg.SkipVerify}\n");
        let findings = Check_Certificate_Verification_Is_Not_Disabled(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Certificate_Verification_Is_Not_Disabled_Should_Accept_An_Adjacent_Reason()
    {
        let source = Source(
            "src/client.rs",
            "// pinned by public-key hash below; certificate authentication is not needed\nlet client = Client::builder().danger_accept_invalid_certs(true).build()?;\n",
        );
        let findings = Check_Certificate_Verification_Is_Not_Disabled(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Certificate_Verification_Is_Not_Disabled_Should_Ignore_Test_Files()
    {
        let source = Source("tests/tls_helper.go", "tls.Config{InsecureSkipVerify: true}\n");
        let findings = Check_Certificate_Verification_Is_Not_Disabled(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        return SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
    }
}
