use super::*;
use crate::checks::test_support::{self, FactToFile, OfferedProvider};
use nomos_analysis::{MemoryFactStore, Reader};
use nomos_capability::Registry;
use nomos_contracts::SubjectId;
use nomos_model::Content_Digest;

#[test]
fn Test_Check_Nesting_Depth_Should_Report_A_Function_Past_The_Limit()
{
    let findings = Judge(&[Source("demo/src/a.rs", Nested(MAX_NESTING_DEPTH + 1))]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.rule, RuleId::New(NESTING_DEPTH));
    assert_eq!(found.gate, GateCategory::Blocking);
}

/// A function *reaching* the limit is fine; only one past it is reported.
#[test]
fn Test_Check_Nesting_Depth_Should_Accept_A_Function_At_The_Limit()
{
    let findings = Judge(&[Source("demo/src/a.rs", Nested(MAX_NESTING_DEPTH))]);

    assert!(findings.is_empty(), "{findings:?}");
}

/// One finding per function, at the first construct to cross — not one per level.
#[test]
fn Test_Check_Nesting_Depth_Should_Report_A_Function_Once()
{
    let findings = Judge(&[Source("demo/src/a.rs", Nested(MAX_NESTING_DEPTH + PAST_THE_LIMIT))]);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

/// An else-if is the second question in one decision, not a decision inside one.
#[test]
fn Test_Check_Nesting_Depth_Should_Not_Deepen_For_An_Else_If()
{
    let text = Function(&[
        "    if a", "    {", "        if b", "        {", "            if c",
        "            {", "                return 1;", "            }",
        "            else if d", "            {", "                return 2;",
        "            }", "        }", "    }",
    ]);

    let findings = Judge(&[Source("demo/src/a.rs", text.to_owned())]);

    assert!(findings.is_empty(), "{findings:?}");
}

/// The other half of the same rule: an else-if does not deepen, but its body still does.
#[test]
fn Test_Check_Nesting_Depth_Should_Deepen_Inside_An_Else_If_Body()
{
    let text = Function(&[
        "    if a", "    {", "        if b", "        {", "            return 1;",
        "        }", "        else if c", "        {", "            if d",
        "            {", "                if e", "                {",
        "                    return 2;", "                }", "            }",
        "        }", "    }",
    ]);

    let findings = Judge(&[Source("demo/src/a.rs", text.to_owned())]);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

/// A brace that opens no control flow adds no level, which is the whole reason this
/// counts constructs rather than braces.
#[test]
fn Test_Check_Nesting_Depth_Should_Not_Count_A_Brace_That_Opens_No_Control_Flow()
{
    let text = Function(&[
        "    if a", "    {", "        let held = Point { x: 1, y: 2 };",
        "        let run = |value: u8| { return value; };", "        match held.x",
        "        {", "            0 => { return 1; },", "            _ => { return 2; },",
        "        }", "    }",
    ]);

    let findings = Judge(&[Source("demo/src/a.rs", text.to_owned())]);

    assert!(findings.is_empty(), "{findings:?}");
}

/// A labelled loop is a loop, and a scan that stopped at the quote would miss it.
#[test]
fn Test_Check_Nesting_Depth_Should_Count_A_Labelled_Loop()
{
    let text = Function(&[
        "    'outer: for row in rows", "    {", "        for column in columns",
        "        {", "            while ready", "            {", "                loop",
        "                {", "                    break 'outer;", "                }",
        "            }", "        }", "    }",
    ]);

    let findings = Judge(&[Source("demo/src/a.rs", text.to_owned())]);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

/// A higher-ranked binder in a where clause is not a loop, and reading it as one is a
/// mistake a sibling prototype actually made against this workspace.
#[test]
fn Test_Check_Nesting_Depth_Should_Not_Read_A_Higher_Ranked_Binder_As_A_Loop()
{
    let text = Function(&[
        "    if a", "    {", "        if b", "        {", "            if c",
        "            {", "                return 1;", "            }", "        }", "    }",
    ]);
    let bound = format!("where\n    for<'error> Error: From<&'error Cause>,\n{text}");

    let findings = Judge(&[Source("demo/src/a.rs", bound)]);

    assert!(findings.is_empty(), "{findings:?}");
}

/// A brace inside a string literal is not a block. This crate has paid twice for a
/// scanner that could not tell the difference.
#[test]
fn Test_Check_Nesting_Depth_Should_Not_Count_A_Brace_Inside_A_String()
{
    let text = Function(&[
        "    if a", "    {", "        let shape = \"if b { if c { if d { deep\";",
        "        return shape.len();", "    }",
    ]);

    let findings = Judge(&[Source("demo/src/a.rs", text.to_owned())]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Nesting_Depth_Should_Judge_Each_Function_Separately()
{
    let text = format!("{}\n{}", Nested(MAX_NESTING_DEPTH), Nested(MAX_NESTING_DEPTH + 1));

    let findings = Judge(&[Source("demo/src/a.rs", text.to_owned())]);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn Test_Check_Nesting_Depth_Should_Ignore_A_Language_It_Does_Not_Judge()
{
    let findings = Judge(&[Source("demo/src/a.go", Nested(MAX_NESTING_DEPTH + 1))]);

    assert!(findings.is_empty(), "{findings:?}");
}

/// The limit is a repository's to declare, the same `OD-RULES-011` mechanism the
/// file-size triggers already resolve through — a declared ceiling of one reports a
/// function the compiled default accepts.
#[test]
fn Test_Check_Nesting_Depth_Should_Resolve_A_Declared_Limit()
{
    let test_support::TestOffering { store, registry, .. } = Offering_With_Declared_Limit(1);
    let mut facts = Reader::On(&store, &registry, test_support::Test_Context());

    let findings = Check_Nesting_Depth(&[Source("demo/src/a.rs", Nested(NESTED_LEVELS))], &mut facts);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

/// A declared-and-offered limits capability whose one repository-wide row sets
/// [`NESTING_DEPTH_MAX_KEY`] to `value` — the policy source the declared-limit test reads.
fn Offering_With_Declared_Limit(value: u32) -> test_support::TestOffering
{
    let test_support::TestOffering { mut store, registry, offer } = test_support::Offered_Registry(
        OfferedProvider {
            contract: nomos_cap_limits_policy::Capability_Contract(),
            capability: nomos_cap_limits_policy::Capability(),
            version: nomos_cap_limits_policy::CONTRACT_VERSION,
            provider: "nomos.test.limits.provides",
            guarantee: nomos_cap_limits_policy::Ceiling(),
        },
    ).expect("a fresh Registry holds neither this contract nor this provider");
    let payload = nomos_cap_limits_policy::LimitsPolicyPayload {
        rows: vec![nomos_cap_limits_policy::PolicyRow {
            scope: nomos_cap_limits_policy::Scope::Repository,
            key: NESTING_DEPTH_MAX_KEY.to_owned(),
            value,
        }],
    };
    test_support::Materialize_Fact(
        &mut store,
        FactToFile {
            subject: nomos_model::Subject_Of_Path(""),
            offer: &offer,
            semantic_inputs: nomos_analysis::InputDigest::Of(&[]),
            schema: nomos_cap_limits_policy::Payload_Schema(),
            bytes: nomos_cap_limits_policy::Encode_Payload(&payload),
        },
    ).expect("the fixture's store holds no fact under this key at a newer generation");

    return test_support::TestOffering { store, registry, offer };
}

/// Every test above reads an empty store, so the compiled default is the limit.
fn Judge(sources: &[SourceFile]) -> Vec<Finding>
{
    let store = MemoryFactStore::New();
    let registry = Registry::New();
    let mut facts = Reader::On(&store, &registry, test_support::Test_Context());

    return Check_Nesting_Depth(sources, &mut facts);
}

/// A function whose control flow nests exactly `levels` deep, built rather than spelled
/// so the shape stays readable at any depth.
/// How far past the limit the fixture below nests -- one deeper than the level the rule
/// reports as the breach, so the fixture states the crossing without stating a second.
const PAST_THE_LIMIT: usize = 3;

/// The nesting depth the policy-limited fixture below builds: one level past the policy
/// row's own limit of one, making the breach unambiguous.
const NESTED_LEVELS: usize = 2;

fn Nested(levels: usize) -> String
{
    let mut body = Vec::new();

    for level in 0..levels
    {
        let indent = "    ".repeat(level.saturating_add(1));
        body.push(format!("{indent}if ready"));
        body.push(format!("{indent}{{"));
    }

    body.push("    return 1;".to_owned());

    for level in (0..levels).rev()
    {
        body.push(format!("{}}}", "    ".repeat(level.saturating_add(1))));
    }

    let borrowed: Vec<&str> = body.iter().map(|line| return line.as_str()).collect();
    return Function(&borrowed);
}

fn Function(body: &[&str]) -> String
{
    let mut lines = vec!["fn Judged(ready: bool) -> u8".to_owned(), "{".to_owned()];
    lines.extend(body.iter().map(|line| return (*line).to_owned()));
    lines.push("    return 0;".to_owned());
    lines.push("}".to_owned());
    return lines.join("\n");
}

fn Source(path: &str, text: String) -> SourceFile
{
    let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
    source.language = crate::Recognized_Language_In_Tests(path);
    return source;
}
