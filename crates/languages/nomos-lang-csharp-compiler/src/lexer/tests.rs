//! Each construct that can span a line, opened and closed; each opener that a string or comment
//! hides; and the one flag the walk reads besides where a line starts.

use super::LineLexer;

/// Feeds `lines` to a fresh lexer and reports, after each, whether the next line starts in code.
fn Code_After_Each(lines: &[&str]) -> Vec<bool>
{
    let mut lexer = LineLexer::default();
    return lines
        .iter()
        .map(|line| {
            lexer.Read_Line(line);
            return lexer.Is_At_Code();
        })
        .collect();
}

#[test]
fn Test_A_Delimited_Comment_Should_Span_Lines_Until_It_Closes()
{
    assert_eq!(Code_After_Each(&["int a; /* opens", "#if NOT_A_DIRECTIVE", "closes */ int b;"]), [false, false, true]);
}

#[test]
fn Test_A_Verbatim_String_Should_Span_Lines_And_Keep_A_Doubled_Quote_Inside()
{
    assert_eq!(Code_After_Each(&["var s = @\"opens", "a \"\"quoted\"\" word", "closes\";"]), [false, false, true]);
}

#[test]
fn Test_A_Raw_String_Should_Close_Only_On_As_Many_Quotes_As_Opened_It()
{
    assert_eq!(Code_After_Each(&["var s = \"\"\"\"", "a \"\"\" is text here", "\"\"\"\";"]), [false, false, true]);
}

#[test]
fn Test_A_Single_Line_Raw_String_Should_Open_And_Close_On_Its_Line()
{
    assert_eq!(Code_After_Each(&["var s = \"\"\"text with \"quotes\" inside\"\"\";"]), [true]);
}

/// The openers that must not open anything: each sits inside a construct that is already open.
#[test]
fn Test_An_Opener_Inside_A_String_Or_Comment_Should_Open_Nothing()
{
    let hidden = [
        "var s = \"/* not a comment\";",
        "var s = \"@\\\"not verbatim\";",
        "// \"not a string, and /* not a comment",
        "var c = '\"';",
        "var s = @\"a /* inside \"\" verbatim\";",
        "var s = \"\"; var t = \"\";",
    ];

    for line in hidden
    {
        assert_eq!(Code_After_Each(&[line]), [true], "{line}");
    }
}

#[test]
fn Test_An_Interpolation_Hole_Should_Hold_Code_That_Opens_Its_Own_Strings()
{
    assert_eq!(Code_After_Each(&["var s = $\"a {F(\"}\")} b\";"]), [true]);
    assert_eq!(Code_After_Each(&["var s = $@\"opens {x}", "{{literal}} and { F(\"x\") }", "closes\";"]), [false, false, true]);
}

/// A hole in an interpolated string can span lines, and code inside it is code: a line starting
/// there starts in code even though a string is open beneath it.
#[test]
fn Test_A_Hole_Spanning_Lines_Should_Leave_The_Line_Start_In_Code()
{
    assert_eq!(Code_After_Each(&["var s = $\"a {F(", "x)} b\";"]), [true, true]);
}

#[test]
fn Test_An_Interpolated_Raw_String_Should_Open_A_Hole_Only_On_Its_Own_Brace_Count()
{
    assert_eq!(Code_After_Each(&["var s = $$\"\"\"", "{ literal } {{hole}}", "\"\"\";"]), [false, false, true]);
}

/// A regular string cannot span a line, so one left open ends with its line.
#[test]
fn Test_An_Unterminated_Regular_String_Should_End_With_Its_Line()
{
    assert_eq!(Code_After_Each(&["var s = \"never closed", "int b;"]), [true, true]);
}

#[test]
fn Test_Seen_Token_Should_Stay_False_Across_Blank_And_Comment_Lines_Only()
{
    let mut lexer = LineLexer::default();
    for line in ["", "   ", "// a comment", "/* a block */"]
    {
        lexer.Read_Line(line);
    }
    assert!(!lexer.seen_token);

    lexer.Read_Line("using System;");
    assert!(lexer.seen_token);
}
