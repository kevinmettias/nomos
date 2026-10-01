//! The codec's own tests: a round trip, the stable bytes, and each refusal by its reason.

use super::*;

fn Sample() -> ConditionalPayload
{
    return ConditionalPayload {
        selection: BuildSelection { project: "src/App/App.csproj".to_owned(), configuration: "Debug".to_owned(), target_framework: "net8.0".to_owned() },
        symbols: vec!["DEBUG".to_owned(), "NET8_0".to_owned(), "TRACE".to_owned()],
        definitions: vec![FileDefinition { line: 1, symbol: "LOCAL".to_owned(), effect: DefinitionEffect::Define }],
        regions: vec![
            ConditionalRegion { branch: Branch::If, line: 3, end_line: 5, condition: "DEBUG && !LOCAL".to_owned(), state: BranchState::Skipped },
            ConditionalRegion { branch: Branch::Else, line: 5, end_line: 7, condition: String::new(), state: BranchState::Compiled },
        ],
    };
}

#[test]
fn Test_Parse_Payload_Should_Round_Trip_A_Payload_Through_Its_Own_Encoding()
{
    let payload = Sample();

    let decoded = Parse_Payload(&Encode_Payload(&payload)).expect("this crate's own encoding");

    assert_eq!(decoded, payload);
}

#[test]
fn Test_Encode_Payload_Should_Produce_Stable_Diffable_Bytes()
{
    let rendered = String::from_utf8(Encode_Payload(&Sample())).expect("UTF-8 by construction");

    assert_eq!(
        rendered,
        "selection\tsrc/App/App.csproj\tDebug\tnet8.0\n\
         symbol\tDEBUG\n\
         symbol\tNET8_0\n\
         symbol\tTRACE\n\
         define\t1\tLOCAL\n\
         region\tif\t3\t5\tskipped\tDEBUG && !LOCAL\n\
         region\telse\t5\t7\tcompiled\t\n"
    );
    assert!(!rendered.contains('\r'), "line endings must not be local");
}

/// A file with no conditional and a build defining nothing still names the build it is about.
#[test]
fn Test_A_Payload_With_Only_Its_Selection_Should_Round_Trip()
{
    let payload = ConditionalPayload { symbols: Vec::new(), definitions: Vec::new(), regions: Vec::new(), ..Sample() };

    let decoded = Parse_Payload(&Encode_Payload(&payload)).expect("a selection alone is a whole answer");

    assert_eq!(decoded, payload);
}

#[test]
fn Test_A_Payload_Naming_No_Build_Or_Two_Should_Be_Refused()
{
    let none = Parse_Payload(b"symbol\tDEBUG\n").expect_err("an answer about no build");
    let two = Parse_Payload(b"selection\ta\tDebug\tnet8.0\nselection\ta\tRelease\tnet8.0\n").expect_err("an answer about two builds");

    assert!(none.reason.contains("no selection line"), "{}", none.reason);
    assert!(two.reason.contains("a second selection line"), "{}", two.reason);
}

#[test]
fn Test_A_Malformed_Line_Should_Be_Refused_For_Its_Own_Reason()
{
    let cases: [(&str, &str); 7] = [
        ("selection\ta\tDebug\n", "exactly three fields"),
        ("selection\ta\tDebug\tnet8.0\nregion\tif\t3\t5\tskipped\n", "exactly five fields"),
        ("selection\ta\tDebug\tnet8.0\nregion\twhile\t3\t5\tskipped\tX\n", "unrecognized branch:"),
        ("selection\ta\tDebug\tnet8.0\nregion\tif\t3\t5\tmaybe\tX\n", "unrecognized branch state"),
        ("selection\ta\tDebug\tnet8.0\nregion\tif\tthree\t5\tskipped\tX\n", "not a count"),
        ("selection\ta\tDebug\tnet8.0\ndefine\tLOCAL\n", "a line and a symbol"),
        ("selection\ta\tDebug\tnet8.0\nsite\tX\n", "none of this schema's four kinds"),
    ];

    for (text, reason) in cases
    {
        let refusal = Parse_Payload(text.as_bytes()).expect_err("a malformed payload");
        assert!(refusal.reason.contains(reason), "{text:?}: {}", refusal.reason);
    }
}

#[test]
fn Test_Every_Label_Should_Round_Trip_Through_Its_Parser()
{
    for branch in [Branch::If, Branch::Elif, Branch::Else]
    {
        assert_eq!(Branch::From_Label(branch.Label()), Some(branch));
    }
    for state in [BranchState::Compiled, BranchState::Skipped, BranchState::Unevaluated]
    {
        assert_eq!(BranchState::From_Label(state.Label()), Some(state));
    }
    for effect in [DefinitionEffect::Define, DefinitionEffect::Undefine]
    {
        assert_eq!(DefinitionEffect::From_Label(effect.Label()), Some(effect));
    }
    assert_eq!(Branch::From_Label("endif"), None);
    assert_eq!(BranchState::From_Label("active"), None);
    assert_eq!(DefinitionEffect::From_Label("undefine"), None);
}
