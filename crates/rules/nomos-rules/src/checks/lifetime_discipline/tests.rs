use super::*;
use nomos_contracts::SubjectId;
use nomos_model::Content_Digest;

#[test]
fn Test_Check_Lifetimes_Follow_The_Descriptive_Naming_Rule_Should_Report_Two_Single_Letter_Lifetimes()
{
    let sources = vec![Source("demo/src/a.rs", Declaration("struct", &[&Lifetime("a"), &Lifetime("b")]))];

    let findings = Check_Lifetimes_Follow_The_Descriptive_Naming_Rule(&sources);

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.rule, RuleId::New(LIFETIMES_FOLLOW_THE_DESCRIPTIVE_NAMING_RULE));
    assert_eq!(found.gate, GateCategory::Blocking);
}

/// One lifetime has nothing to be told apart from, which is the whole reason the count
/// gate exists.
#[test]
fn Test_Check_Lifetimes_Follow_The_Descriptive_Naming_Rule_Should_Accept_A_Single_Lifetime()
{
    let only = Lifetime("a");
    let text = format!("fn Split<{only}>(rest: &{only} str) -> (&{only} str, &{only} str)");
    let sources = vec![Source("demo/src/a.rs", text.to_owned())];

    let findings = Check_Lifetimes_Follow_The_Descriptive_Naming_Rule(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Lifetimes_Follow_The_Descriptive_Naming_Rule_Should_Accept_Descriptive_Names()
{
    let sources = vec![Source("demo/src/a.rs", Declaration("struct", &[&Lifetime("arena"), &Lifetime("frame")]))];

    let findings = Check_Lifetimes_Follow_The_Descriptive_Naming_Rule(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

/// Neither of these is a name an author chose, so a pair of them is not a pair to tell
/// apart. This is the shape most of this workspace's own signatures already carry.
#[test]
fn Test_Check_Lifetimes_Follow_The_Descriptive_Naming_Rule_Should_Not_Count_The_Unchosen_Lifetimes()
{
    let anonymous = Lifetime("_");
    let text = format!("fn Read(reader: &mut Reader<{anonymous}, {anonymous}>) -> &{} str", Lifetime("static"));
    let sources = vec![Source("demo/src/a.rs", text.to_owned())];

    let findings = Check_Lifetimes_Follow_The_Descriptive_Naming_Rule(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

/// A char literal opens with the same character a lifetime does, and this crate has
/// already paid twice for a scanner that could not tell them apart.
#[test]
fn Test_Check_Lifetimes_Follow_The_Descriptive_Naming_Rule_Should_Not_Read_A_Char_Literal_As_A_Lifetime()
{
    let text = format!(
        "fn Split(line: &str) -> bool {{ return line.contains({}) && line.contains({}); }}",
        Char_Literal('a'),
        Char_Literal('b')
    );
    let sources = vec![Source("demo/src/a.rs", text.to_owned())];

    let findings = Check_Lifetimes_Follow_The_Descriptive_Naming_Rule(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

/// A lifetime used on a line that opens no declaration was declared somewhere else.
#[test]
fn Test_Check_Lifetimes_Follow_The_Descriptive_Naming_Rule_Should_Only_Judge_A_Declaration()
{
    let text = format!("    left: &{} str,\n    right: &{} str,", Lifetime("a"), Lifetime("b"));
    let sources = vec![Source("demo/src/a.rs", text.to_owned())];

    let findings = Check_Lifetimes_Follow_The_Descriptive_Naming_Rule(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

/// `P69-SELF-MATCH-VIA-STRING-LITERALS-FIVE-MORE-RULES`: a string literal's own prose can
/// spell "struct"/"impl" and quote a lifetime by name several times without a single one
/// of those being real, chosen syntax.
#[test]
fn Test_Check_Lifetimes_Follow_The_Descriptive_Naming_Rule_Should_Not_Judge_A_String_Literals_Own_Text()
{
    let text = "let reason = \"struct BytesToHexChars<'a>, impl<'a> ... for BytesToHexChars<'a>\";";
    let sources = vec![Source("demo/src/a.rs", text.to_owned())];

    let findings = Check_Lifetimes_Follow_The_Descriptive_Naming_Rule(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Static_Bounds_Are_Justified_Should_Report_An_Unexplained_Bound()
{
    let sources = vec![Source("demo/src/a.rs", Bounded_Function(""))];

    let findings = Check_Static_Bounds_Are_Justified(&sources);

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.rule, RuleId::New(STATIC_BOUNDS_ARE_JUSTIFIED));
    assert_eq!(found.gate, GateCategory::Blocking);
}

#[test]
fn Test_Check_Static_Bounds_Are_Justified_Should_Report_A_Bound_Added_With_A_Plus()
{
    let sources = vec![Source("demo/src/a.rs", Bounded_Function("Read + Send + "))];

    let findings = Check_Static_Bounds_Are_Justified(&sources);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn Test_Check_Static_Bounds_Are_Justified_Should_Report_A_Where_Clause_Bound()
{
    let text = format!("    where Source: Read + {}", Lifetime("static"));
    let sources = vec![Source("demo/src/a.rs", text.to_owned())];

    let findings = Check_Static_Bounds_Are_Justified(&sources);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

/// The divergence this port is built on. The one real instance in this workspace carries
/// exactly this shape: prose above the bound, in no particular spelling.
#[test]
fn Test_Check_Static_Bounds_Are_Justified_Should_Accept_An_Adjacent_Explanation()
{
    let text = format!(
        "    // Spawning moves the reader onto a detached thread that outlives this call, so\n    // the standard library demands the bound rather than this signature choosing it.\n{}",
        Bounded_Function("Read + Send + ")
    );
    let sources = vec![Source("demo/src/a.rs", text.to_owned())];

    let findings = Check_Static_Bounds_Are_Justified(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

/// A divider is not an explanation.
#[test]
fn Test_Check_Static_Bounds_Are_Justified_Should_Not_Accept_A_Wordless_Comment()
{
    let text = format!("    // ----\n{}", Bounded_Function(""));
    let sources = vec![Source("demo/src/a.rs", text.to_owned())];

    let findings = Check_Static_Bounds_Are_Justified(&sources);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

/// A reference *to* something living that long is not a bound placed on a caller's type,
/// and this workspace is full of the former.
#[test]
fn Test_Check_Static_Bounds_Are_Justified_Should_Ignore_A_Static_Reference()
{
    let text = format!("fn Name() -> &{} str", Lifetime("static"));
    let sources = vec![Source("demo/src/a.rs", text.to_owned())];

    let findings = Check_Static_Bounds_Are_Justified(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

/// `P69-SELF-MATCH-VIA-STRING-LITERALS-FIVE-MORE-RULES`: a string literal's own prose can
/// quote `'static` without that being a real trait bound.
#[test]
fn Test_Check_Static_Bounds_Are_Justified_Should_Not_Judge_A_String_Literals_Own_Text()
{
    let text = "let reason = \"the fixture's only 'static usage is a reference's own lifetime, not T: 'static\";";
    let sources = vec![Source("demo/src/a.rs", text.to_owned())];

    let findings = Check_Static_Bounds_Are_Justified(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Static_Bounds_Are_Justified_Should_Ignore_A_Language_It_Does_Not_Judge()
{
    let sources = vec![Source("demo/src/a.go", Bounded_Function(""))];

    let findings = Check_Static_Bounds_Are_Justified(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

/// The character a lifetime opens with, held once so no fixture below has to spell one.
///
/// Every fixture in this module is built rather than written out, because a rule about
/// lifetimes whose own tests spell lifetimes is a rule that reports its own test module.
/// Measured, not assumed: writing them literally produced five findings against this
/// file on a real run over this workspace. Nine modules in this crate answer the same
/// hazard with a path-suffix exemption that would exempt any file in any repository
/// ending the same way, and `P45-CODE-PREFIX-KNOWS-STRINGS` is retiring it, so this
/// file does not add a tenth.
const LIFETIME_MARK: char = '\'';

fn Lifetime(name: &str) -> String
{
    return format!("{LIFETIME_MARK}{name}");
}

fn Char_Literal(held: char) -> String
{
    return format!("{LIFETIME_MARK}{held}{LIFETIME_MARK}");
}

fn Declaration(keyword: &str, lifetimes: &[&String]) -> String
{
    let named: Vec<&str> = lifetimes.iter().map(|lifetime| return lifetime.as_str()).collect();
    return format!("{keyword} Judged<{}>", named.join(", "));
}

fn Bounded_Function(leading_bounds: &str) -> String
{
    return format!("fn Judged<T: {leading_bounds}{}>(value: T) -> T", Lifetime("static"));
}

fn Source(path: &str, text: String) -> SourceFile
{
    let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
    source.language = crate::Recognized_Language_In_Tests(path);
    return source;
}
