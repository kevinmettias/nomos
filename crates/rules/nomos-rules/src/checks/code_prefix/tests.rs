use super::*;

/// The exact regression `P66-UNSAFE-JUSTIFICATION-STRING-BLINDNESS` measured:
/// `nomos-lang-rust-scan/src/item_kind.rs` declares
/// `("unsafe impl", Self::Implementation)`, a table entry whose own quoted text is not
/// a real `unsafe impl` — a bare substring search over `Code_Prefix`'s own output
/// (which preserves a string's real text) reads it as one anyway.
#[test]
fn Test_Code_With_String_Bodies_Masked_Should_Not_Read_A_Table_Entrys_Own_String_As_Real_Code()
{
    let line = "(\"unsafe impl\", Self::Implementation),";

    let masked = Code_With_String_Bodies_Masked(line);

    assert!(!masked.contains("unsafe impl"), "{masked:?}");
}

/// The masked scan still finds a real, unquoted `unsafe {` — it must not blind a real
/// caller to real code, only to a string's own quoted text.
#[test]
fn Test_Code_With_String_Bodies_Masked_Should_Still_Find_Real_Unquoted_Code()
{
    let line = "let s = \"unsafe impl\"; unsafe { core::ptr::read(p) }";

    let masked = Code_With_String_Bodies_Masked(line);

    assert!(masked.contains("unsafe { core::ptr::read(p) }"), "{masked:?}");
    assert!(!masked.contains("unsafe impl"), "the string's own text must be masked: {masked:?}");
}

/// A trailing `//` comment is still recognized once the preceding string is masked,
/// not read through by accident.
#[test]
fn Test_Code_With_String_Bodies_Masked_Should_Still_Strip_A_Trailing_Comment()
{
    let line = "let s = \"https://example.com\"; // a comment";

    let masked = Code_With_String_Bodies_Masked(line);

    assert!(!masked.contains("a comment"), "{masked:?}");
}

/// A raw string's own unsafe-shaped text is masked the same way a plain string's is.
#[test]
fn Test_Code_With_String_Bodies_Masked_Should_Mask_A_Raw_Strings_Own_Text()
{
    let line = "let s = r#\"unsafe impl Foo for Bar {}\"#;";

    let masked = Code_With_String_Bodies_Masked(line);

    assert!(!masked.contains("unsafe impl"), "{masked:?}");
}

#[test]
fn Test_Code_Prefix_Should_Strip_A_Trailing_Comment()
{
    assert_eq!(Code_Prefix("let x = 1; // a comment"), "let x = 1; ");
}

#[test]
fn Test_Code_Prefix_Should_Return_The_Whole_Line_With_No_Comment()
{
    assert_eq!(Code_Prefix("let x = 1;"), "let x = 1;");
}

/// The first real defect this rule exists to fix: a string literal holding a URL must
/// not truncate the real code that follows it on the same line.
///
/// Deliberately not spelled `unsafe { ... }` past the string, unlike an earlier version
/// of this test: `unsafe-justification`'s own `Has_Unsafe_Construct` is a bare substring
/// search over this same `Code_Prefix` output, and since this function preserves a
/// string's own body rather than blanking it, a fixture whose STRING happened to contain
/// that phrase already tripped it once — this file's own module doc names why blanking
/// is not the fix. `Real_Code(p)` proves the identical truncation property without
/// handing another rule's text scanner a phrase it does not know is quoted.
#[test]
fn Test_Code_Prefix_Should_Not_Truncate_At_A_Url_Inside_A_String()
{
    let line = "let doc = \"https://example.com\"; Real_Code(p)";

    let prefix = Code_Prefix(line);

    assert!(
        prefix.contains("Real_Code(p)"),
        "the string's own // must not hide the real code after it: {prefix:?}"
    );
}

/// A literal's own body is preserved, not blanked — this crate's own module doc names
/// why: `A_Rust_Path_Stays_Within_Its_Own_Subtree` and others need a specific string's
/// real value, not a placeholder. Says "block" rather than `unsafe`, for the reason the
/// test above now states explicitly: this string's own text must not spell a phrase
/// another rule's own text scanner reads as real code.
#[test]
fn Test_Code_Prefix_Should_Preserve_A_Strings_Own_Text()
{
    let line = "let example = \"call the block { ... } to do it\";";

    assert_eq!(Code_Prefix(line), line);
}

#[test]
fn Test_Code_Prefix_Should_Strip_A_Real_Comment_After_A_Url_Bearing_String()
{
    let line = "let doc = \"https://example.com\"; // real comment";

    let prefix = Code_Prefix(line);

    assert!(prefix.starts_with("let doc = "), "{prefix:?}");
    assert!(!prefix.contains("real comment"), "{prefix:?}");
}

#[test]
fn Test_Code_Prefix_Should_Honor_An_Escaped_Quote_Inside_A_String()
{
    let line = "let s = \"a \\\" // not a comment\"; Real_Code()";

    let prefix = Code_Prefix(line);

    assert!(prefix.contains("Real_Code()"), "the escaped quote must not end the string early: {prefix:?}");
}

#[test]
fn Test_Code_Prefix_Should_Not_Read_A_Quoted_Char_Literal_As_Opening_A_String()
{
    let line = "let c = '\"'; // real comment";

    let prefix = Code_Prefix(line);

    assert!(prefix.starts_with("let c = "), "{prefix:?}");
    assert!(!prefix.contains("real comment"), "{prefix:?}");
}

#[test]
fn Test_Code_Prefix_Should_Not_Read_An_Escaped_Quote_Char_Literal_As_Opening_A_String()
{
    let line = "let c = '\\''; // real comment";

    let prefix = Code_Prefix(line);

    assert!(prefix.starts_with("let c = "), "{prefix:?}");
    assert!(!prefix.contains("real comment"), "{prefix:?}");
}

/// A lifetime has no closing `'`; it must not be misread as an unterminated char
/// literal that swallows the rest of the line, including a real trailing comment.
#[test]
fn Test_Code_Prefix_Should_Not_Read_A_Lifetime_As_A_Char_Literal()
{
    let line = "fn F<'a>(x: &'a str) -> &'a str // real comment";

    let prefix = Code_Prefix(line);

    assert!(prefix.contains("fn F<'a>(x: &'a str) -> &'a str"), "{prefix:?}");
    assert!(!prefix.contains("real comment"), "{prefix:?}");
}

#[test]
fn Test_Code_Prefix_Should_Not_Truncate_At_A_Double_Slash_Inside_A_Raw_String()
{
    let line = "let s = r\"https://example.com\"; Real_Code(p)";

    let prefix = Code_Prefix(line);

    assert!(prefix.contains("Real_Code(p)"), "{prefix:?}");
}

#[test]
fn Test_Code_Prefix_Should_Not_Truncate_At_A_Double_Slash_Inside_A_Hashed_Raw_String()
{
    let line = "let s = r#\"a \" b // still a string\"#; Real_Code()";

    let prefix = Code_Prefix(line);

    assert!(prefix.contains("Real_Code()"), "{prefix:?}");
}

/// A raw string with an unbalanced quote inside it — one hash count, one real
/// delimiter — must still close only at its own real delimiter, not at the stray `"`.
/// Its own body reads `begin { let x = "`, not `unsafe { let x = "` as an earlier version
/// of this test spelled it: the shape under test is a brace and a stray quote inside a
/// raw string, and spelling it with the one word `unsafe-justification`'s own text
/// scanner reads as real code — a bare substring search over this same `Code_Prefix`
/// output — made this fixture's own quoted text trip a different rule's finding.
#[test]
fn Test_Code_Prefix_Should_Handle_A_Raw_String_With_An_Unbalanced_Quote()
{
    let line = "let s = r#\"begin { let x = \" } \"#; // real comment";

    let prefix = Code_Prefix(line);

    assert!(prefix.starts_with("let s = r#\"begin { let x = \" } \"#; "), "{prefix:?}");
    assert!(!prefix.contains("real comment"), "{prefix:?}");
}

#[test]
fn Test_Code_Prefix_Should_Strip_A_Comment_After_A_Byte_String()
{
    let line = "let b = b\"raw // bytes\"; // real comment";

    let prefix = Code_Prefix(line);

    assert!(prefix.starts_with("let b = "), "{prefix:?}");
    assert!(!prefix.contains("real comment"), "{prefix:?}");
}

#[test]
fn Test_Code_Prefix_Should_Strip_A_Comment_After_A_Raw_Byte_String()
{
    let line = "let b = br\"raw // bytes\"; // real comment";

    let prefix = Code_Prefix(line);

    assert!(prefix.starts_with("let b = "), "{prefix:?}");
    assert!(!prefix.contains("real comment"), "{prefix:?}");
}
