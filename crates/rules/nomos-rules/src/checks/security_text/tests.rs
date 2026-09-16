use super::*;
use nomos_contracts::SubjectId;
use nomos_model::Content_Digest;

#[test]
fn Test_Check_A_Credential_Is_Not_Hardcoded_In_Source_Should_Report_An_Aws_Access_Key()
{
    let text = format!("const KEY: &str = \"{}\";\n", Aws_Access_Key_Fixture());
    let source = Source(SourceText { path: "src/config.rs", text: &text });
    let findings = Check_A_Credential_Is_Not_Hardcoded_In_Source(&[source]);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(A_CREDENTIAL_IS_NOT_HARDCODED_IN_SOURCE));
}

#[test]
fn Test_Check_A_Credential_Is_Not_Hardcoded_In_Source_Should_Ignore_The_Documented_Example_Key()
{
    let source = Source(SourceText { path: "src/config.rs", text: "const EXAMPLE: &str = \"AKIAIOSFODNN7EXAMPLE\";\n" });
    let findings = Check_A_Credential_Is_Not_Hardcoded_In_Source(&[source]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Credential_Is_Not_Hardcoded_In_Source_Should_Ignore_A_Stripe_Test_Key()
{
    let source = Source(SourceText { path: "src/config.rs", text: "const KEY: &str = \"sk_test_ABCDEFGHIJKLMNOPQRST\";\n" });
    let findings = Check_A_Credential_Is_Not_Hardcoded_In_Source(&[source]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Credential_Is_Not_Hardcoded_In_Source_Should_Report_A_Stripe_Live_Key()
{
    let text = format!("const KEY: &str = \"{}\";\n", Stripe_Live_Key_Fixture());
    let source = Source(SourceText { path: "src/config.rs", text: &text });
    let findings = Check_A_Credential_Is_Not_Hardcoded_In_Source(&[source]);
    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn Test_Check_A_Credential_Is_Not_Hardcoded_In_Source_Should_Report_A_Pem_Private_Key_Header()
{
    let text = Private_Key_Header_Fixture();
    let source = Source(SourceText { path: "src/config.rs", text: &text });
    let findings = Check_A_Credential_Is_Not_Hardcoded_In_Source(&[source]);
    assert_eq!(findings.len(), 1, "{findings:?}");
}

/// A PEM private-key header, assembled from its algorithm name and its body for the reason
/// stated beside the two shared fixtures at the end of this module: written whole, this
/// file would report itself. No key body follows it -- the header alone is what the rule
/// matches -- and the rule still receives the assembled header, whole.
fn Private_Key_Header_Fixture() -> String
{
    return format!("-----BEGIN {}PRIVATE KEY-----\n", "RSA ");
}

#[test]
fn Test_Check_A_Credential_Is_Not_Hardcoded_In_Source_Should_Ignore_A_Resource_Id()
{
    let source = Source(SourceText { path: "src/config.rs", text: "const RESOURCE_ID: &str = \"a1b2c3d4-e5f6-7890\";\n" });
    let findings = Check_A_Credential_Is_Not_Hardcoded_In_Source(&[source]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Credential_Is_Not_Hardcoded_In_Source_Should_Ignore_Test_Files()
{
    let text = format!("const KEY: &str = \"{}\";\n", Stripe_Live_Key_Fixture());
    let source = Source(SourceText { path: "tests/fixtures.rs", text: &text });
    let findings = Check_A_Credential_Is_Not_Hardcoded_In_Source(&[source]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Secret_Does_Not_Travel_In_A_Url_Should_Report_An_Api_Key_Query_Parameter()
{
    let text = Api_Key_Url_Fixture();
    let source = Source(SourceText { path: "src/client.rs", text: &text });
    let findings = Check_A_Secret_Does_Not_Travel_In_A_Url(&[source]);
    assert_eq!(findings.len(), 1, "{findings:?}");
}

/// A URL-assembling line whose query names `api_key`, assembled for the same reason: the
/// `?api_key=` shape in one literal is what this file would report about itself. The braces
/// the outer `format!` would read are doubled, so the assembled line reaches the rule
/// exactly as it did before.
fn Api_Key_Url_Fixture() -> String
{
    return format!("let url = format!(\"https://api.example.com/data?{}={{key}}\");\n", "api_key");
}

#[test]
fn Test_Check_A_Secret_Does_Not_Travel_In_A_Url_Should_Report_A_Second_Position_Token_Parameter()
{
    let text = Token_Url_Fixture();
    let source = Source(SourceText { path: "src/client.rs", text: &text });
    let findings = Check_A_Secret_Does_Not_Travel_In_A_Url(&[source]);
    assert_eq!(findings.len(), 1, "{findings:?}");
}

/// The same line with the credential in the *second* query position, which is a separate
/// case for the URL rule: the first pair must not hide the one behind it.
fn Token_Url_Fixture() -> String
{
    return format!("let url = format!(\"https://api.example.com/data?page=2&{}={{t}}\");\n", "token");
}

#[test]
fn Test_Check_A_Secret_Does_Not_Travel_In_A_Url_Should_Ignore_A_Non_Secret_Parameter()
{
    let source = Source(SourceText { path: "src/client.rs", text: "let url = format!(\"https://api.example.com/data?page={n}&limit=20\");\n" });
    let findings = Check_A_Secret_Does_Not_Travel_In_A_Url(&[source]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Certificate_Verification_Is_Not_Disabled_Should_Report_Insecure_Skip_Verify_True()
{
    let source = Source(SourceText { path: "src/client.go", text: "tls.Config{InsecureSkipVerify: true}\n" });
    let findings = Check_Certificate_Verification_Is_Not_Disabled(&[source]);
    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn Test_Check_Certificate_Verification_Is_Not_Disabled_Should_Report_Danger_Accept_Invalid_Certs()
{
    let source = Source(SourceText { path: "src/client.rs", text: "let client = Client::builder().danger_accept_invalid_certs(true).build()?;\n" });
    let findings = Check_Certificate_Verification_Is_Not_Disabled(&[source]);
    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn Test_Check_Certificate_Verification_Is_Not_Disabled_Should_Ignore_A_Variable_Value()
{
    let source = Source(SourceText { path: "src/client.go", text: "tls.Config{InsecureSkipVerify: cfg.SkipVerify}\n" });
    let findings = Check_Certificate_Verification_Is_Not_Disabled(&[source]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Certificate_Verification_Is_Not_Disabled_Should_Accept_An_Adjacent_Reason()
{
    let source = Source(
        SourceText { path: "src/client.rs", text: "// pinned by public-key hash below; certificate authentication is not needed\nlet client = Client::builder().danger_accept_invalid_certs(true).build()?;\n" },
    );
    let findings = Check_Certificate_Verification_Is_Not_Disabled(&[source]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Certificate_Verification_Is_Not_Disabled_Should_Ignore_Test_Files()
{
    let source = Source(SourceText { path: "tests/tls_helper.go", text: "tls.Config{InsecureSkipVerify: true}\n" });
    let findings = Check_Certificate_Verification_Is_Not_Disabled(&[source]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Credential_Is_Not_Hardcoded_In_Source_Should_Not_Judge_Its_Own_Implementation_File()
{
    let text = format!("let key = \"{}\";\n", Aws_Access_Key_Fixture());
    let source = Source(SourceText { path: "crates/rules/nomos-rules/src/checks/security_text.rs", text: &text });

    let findings = Check_A_Credential_Is_Not_Hardcoded_In_Source(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

// The two credential shapes more than one case needs, kept here rather than beside any one
// caller: a helper with a single caller belongs under it, a shared one belongs at the end.
//
// Each is assembled from two literals rather than written whole because this file is not a
// test file by path -- it IS the implementation of `Check_A_Credential_Is_Not_Hardcoded_In_
// Source` and its two neighbours -- so each rule below judges this file along with every
// other source it is handed, and a fixture written whole reports itself. The fixture cannot
// move: what is under test is exactly that these rules recognise these shapes in an
// ordinary source file. So the shape is split into parts, neither of which is
// credential-shaped on its own, and the rule still receives the assembled text byte for
// byte. That is a property of the fixture, not a waiver: no marker excuses any of these
// sites, and the assertions beside them are unchanged.

/// An AWS access key id: the prefix and a sixteen-character body, which in one literal is
/// the shape the credential rule reports.
fn Aws_Access_Key_Fixture() -> String
{
    return format!("AKIA{}", "ABCDEFGHIJKLMNOP");
}

/// A Stripe live secret key, assembled for the same reason. `sk_live_` alone is a prefix
/// the rule deliberately does not report, and the body alone carries no prefix at all.
fn Stripe_Live_Key_Fixture() -> String
{
    return format!("sk_live_{}", "ABCDEFGHIJKLMNOPQRST");
}

/// A fixture source's two halves, grouped so a call site names which string is the path
/// and which is the text, rather than counting two adjacent `&str` positions a caller
/// could transpose without the compiler objecting.
struct SourceText<'text>
{
    path: &'text str,
    text: &'text str,
}

fn Source(source: SourceText<'_>) -> SourceFile
{
    let SourceText { path, text } = source;
    return SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
}
