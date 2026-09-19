use super::*;
use nomos_analysis::{FactReader, MemoryFactStore, Reader};
use nomos_capability::Registry;
use nomos_contracts::SubjectId;
use nomos_model::Content_Digest;

#[test]
fn Test_Check_Unwrap_Expect_Discipline_Should_Report_Unwrap_In_Production_Rust()
{
    let source = Source(Path("src/lib.rs"), Text("let value = option.unwrap();\n"));

    let findings = Check(Check_Unwrap_Expect_Discipline, source);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(UNWRAP_EXPECT_DISCIPLINE));
}

#[test]
fn Test_Check_Unwrap_Expect_Discipline_Should_Ignore_Test_Rust()
{
    let source = Source(Path("tests/parser.rs"), Text("let value = option.unwrap();\n"));

    let findings = Check(Check_Unwrap_Expect_Discipline, source);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Unwrap_Expect_Discipline_Should_Report_Placeholder_Expect()
{
    let source = Source(Path("src/lib.rs"), Text("let value = result.expect(\"should not happen\");\n"));

    let findings = Check(Check_Unwrap_Expect_Discipline, source);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn Test_Check_Panics_Are_Justified_Documented_And_Validated_Should_Report_Unexplained_Panic()
{
    let source = Source(Path("src/lib.rs"), Text("fn Crash()\n{\n    panic!(\"bad\");\n}\n"));

    let findings = Check_Panics_Are_Justified_Documented_And_Validated(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(
        findings.first().expect("asserted len 1 above").rule,
        RuleId::New(PANICS_ARE_JUSTIFIED_DOCUMENTED_AND_VALIDATED)
    );
}

#[test]
fn Test_Check_Panics_Are_Justified_Documented_And_Validated_Should_Accept_A_Local_Invariant_Note()
{
    let source = Source(Path("src/lib.rs"), Text("fn Crash()\n{\n    // invariant: the parser produced a non-empty token stack\n    unreachable!();\n}\n"));

    let findings = Check_Panics_Are_Justified_Documented_And_Validated(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Rust_Path_Stays_Within_Its_Own_Subtree_Should_Report_Parent_Escape()
{
    let source = Source(Path("src/lib.rs"), Text("#[path = \"../outside.rs\"]\nmod outside;\n"));

    let findings = Check_A_Rust_Path_Stays_Within_Its_Own_Subtree(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(
        findings.first().expect("asserted len 1 above").rule,
        RuleId::New(A_RUST_PATH_STAYS_WITHIN_ITS_OWN_SUBTREE)
    );
}

#[test]
fn Test_Check_A_Rust_Path_Stays_Within_Its_Own_Subtree_Should_Accept_Normalized_Internal_Parent()
{
    let source = Source(Path("src/lib.rs"), Text("#[path = \"pool/../pool/tests.rs\"]\nmod tests;\n"));

    let findings = Check_A_Rust_Path_Stays_Within_Its_Own_Subtree(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Shared_Interior_Mutability_Says_Why_Should_Report_Unexplained_Rc_RefCell()
{
    let source = Source(Path("src/lib.rs"), Text("nodes: Vec<Rc<RefCell<Node>>>,\n"));

    let findings = Check_Shared_Interior_Mutability_Says_Why(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(
        findings.first().expect("asserted len 1 above").rule,
        RuleId::New(SHARED_INTERIOR_MUTABILITY_SAYS_WHY)
    );
}

#[test]
fn Test_Check_Shared_Interior_Mutability_Says_Why_Should_Accept_A_Reason()
{
    let source = Source(Path("src/lib.rs"), Text("// smart-pointer: allow: the graph is cyclic, so no single owner exists\nnodes: Rc<RefCell<Node>>,\n"));

    let findings = Check_Shared_Interior_Mutability_Says_Why(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Shared_Interior_Mutability_Says_Why_Should_Ignore_Arc_Mutex()
{
    let source = Source(Path("src/lib.rs"), Text("state: Arc<Mutex<State>>,\n"));

    let findings = Check_Shared_Interior_Mutability_Says_Why(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

/// `P69-SELF-MATCH-VIA-STRING-LITERALS-FIVE-MORE-RULES`: a string literal's own prose
/// naming the construct is not the construct.
#[test]
fn Test_Check_Shared_Interior_Mutability_Says_Why_Should_Not_Judge_A_String_Literals_Own_Text()
{
    let source = Source(Path("src/lib.rs"), Text("let reason = \"no Rc<RefCell<...>>-shaped construct anywhere\";\n"));

    let findings = Check_Shared_Interior_Mutability_Says_Why(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Every_Allow_Carries_A_Justification_Should_Report_An_Unexplained_Allow()
{
    let source = Source(Path("src/lib.rs"), Text("#[allow(clippy::redundant_clone)]\nlet processed = input.clone();\n"));

    let findings = Check(Check_Every_Allow_Carries_A_Justification, source);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(
        findings.first().expect("asserted len 1 above").rule,
        RuleId::New(EVERY_ALLOW_CARRIES_A_JUSTIFICATION)
    );
}

#[test]
fn Test_Check_Every_Allow_Carries_A_Justification_Should_Accept_An_Explained_Allow()
{
    let source = Source(
        Path("src/lib.rs",),
        Text("// the clone is required because the caller retains the original elsewhere\n\
         #[allow(clippy::redundant_clone)]\n\
         let processed = input.clone();\n"),
    );

    let findings = Check(Check_Every_Allow_Carries_A_Justification, source);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Every_Allow_Carries_A_Justification_Should_Accept_A_Crate_Level_Allow_With_A_Reason()
{
    let source = Source(Path("src/lib.rs"), Text("// this crate is a thin FFI shim and every public item is consumed externally\n#![allow(dead_code)]\n"));

    let findings = Check(Check_Every_Allow_Carries_A_Justification, source);

    assert!(findings.is_empty(), "{findings:?}");
}

/// The same unexplained allow the case above reports, moved into a test source. It is a
/// path exemption, so the content is deliberately identical: only where the file sits
/// differs.
#[test]
fn Test_Check_Every_Allow_Carries_A_Justification_Should_Not_Judge_A_Test_Source()
{
    let source = Source(Path("crates/languages/nomos-lang-rust/tests/guarantee.rs"), Text("#[allow(clippy::redundant_clone)]\nlet processed = input.clone();\n"));

    let findings = Check(Check_Every_Allow_Carries_A_Justification, source);

    assert!(findings.is_empty(), "{findings:?}");
}

/// The sibling rules in this file deliberately do not carry the exemption, and this is
/// the case that keeps that deliberate. A disabled test lives in a test source by
/// construction, so exempting one here would delete the rule rather than narrow it.
#[test]
fn Test_Check_A_Disabled_Test_States_Why_Should_Still_Judge_A_Test_Source()
{
    let source = Source(Path("crates/languages/nomos-lang-rust/tests/guarantee.rs"), Text("#[ignore]\nfn Test_Something() {}\n"));

    let findings = Check_A_Disabled_Test_States_Why(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn Test_Check_Unsafe_Justification_Should_Report_An_Unexplained_Unsafe_Block()
{
    let source = Source(Path("src/lib.rs"), Text("let slice = unsafe { core::slice::from_raw_parts(ptr, len) };\n"));

    let findings = Check_Unsafe_Justification(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(UNSAFE_JUSTIFICATION));
}

#[test]
fn Test_Check_Unsafe_Justification_Should_Accept_A_Safety_Comment()
{
    let source = Source(
        Path("src/lib.rs",),
        Text("// SAFETY:\n\
         // - ptr is valid for len elements, checked by the caller above\n\
         let slice = unsafe { core::slice::from_raw_parts(ptr, len) };\n"),
    );

    let findings = Check_Unsafe_Justification(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Unsafe_Justification_Should_Ignore_The_Forbid_Unsafe_Code_Attribute()
{
    let source = Source(Path("src/lib.rs"), Text("#![forbid(unsafe_code)]\n"));

    let findings = Check_Unsafe_Justification(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

/// `P66-UNSAFE-JUSTIFICATION-STRING-BLINDNESS`: a real, committed table entry in
/// `nomos-lang-rust-scan/src/item_kind.rs`, `("unsafe impl", Self::Implementation),`,
/// was reported as an unjustified `unsafe impl` — the phrase is this line's own quoted
/// data, not a declaration, and the file this line comes from has no real `unsafe` at
/// all (`#![forbid(unsafe_code)]`, matching the test above).
#[test]
fn Test_Check_Unsafe_Justification_Should_Not_Read_A_String_Literals_Own_Text_As_A_Declaration()
{
    let source = Source(Path("src/item_kind.rs"), Text("        (\"unsafe impl\", Self::Implementation),\n"));

    let findings = Check_Unsafe_Justification(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

/// The same string-literal blindness, for the bare-block half of the dispatch: a
/// string spelling `unsafe { ... }` as data must not be read as a real block either.
#[test]
fn Test_Check_Unsafe_Justification_Should_Not_Read_A_String_Literals_Own_Block_Text_As_A_Declaration()
{
    let source = Source(Path("src/fixture.rs"), Text("let line = \"unsafe { core::ptr::read(p) }\";\n"));

    let findings = Check_Unsafe_Justification(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Unsafe_Justification_Should_Report_An_Unsafe_Fn_With_No_Safety_Comment()
{
    let source = Source(Path("src/lib.rs"), Text("pub unsafe fn Read_Raw(ptr: *const u8) -> u8\n{\n    return *ptr;\n}\n"));

    let findings = Check_Unsafe_Justification(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

/// `OD-RULES-021`'s real regression: the shape `aho-corasick`'s own `packed/ext.rs` and
/// `automaton.rs` both carry on a real `unsafe fn`/`unsafe trait`, measured directly and
/// reported unjustified before this fix.
#[test]
fn Test_Check_Unsafe_Justification_Should_Accept_A_Rustdoc_Safety_Section_On_An_Unsafe_Fn()
{
    let source = Source(
        Path("src/lib.rs",),
        Text("/// Reads one byte from `ptr`.\n\
         ///\n\
         /// # Safety\n\
         ///\n\
         /// `ptr` must be valid for reads of one byte.\n\
         pub unsafe fn Read_Raw(ptr: *const u8) -> u8\n{\n    return *ptr;\n}\n"),
    );

    let findings = Check_Unsafe_Justification(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Unsafe_Justification_Should_Accept_A_Rustdoc_Safety_Section_On_An_Unsafe_Trait()
{
    let source = Source(
        Path("src/lib.rs",),
        Text("/// # Safety\n\
         ///\n\
         /// Implementors must uphold the layout invariant.\n\
         pub unsafe trait Packed\n{\n}\n"),
    );

    let findings = Check_Unsafe_Justification(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Unsafe_Justification_Should_Report_An_Unsafe_Fn_Whose_Safety_Section_Has_No_Text()
{
    let source = Source(
        Path("src/lib.rs",),
        Text("/// # Safety\n\
         pub unsafe fn Read_Raw(ptr: *const u8) -> u8\n{\n    return *ptr;\n}\n"),
    );

    let findings = Check_Unsafe_Justification(&[source]);

    assert_eq!(findings.len(), 1, "a bare heading with nothing after it must not satisfy the rule: {findings:?}");
}

#[test]
fn Test_Check_Unsafe_Justification_Should_Not_Accept_An_Unversioned_Comment_On_An_Unsafe_Fn()
{
    let source = Source(
        Path("src/lib.rs",),
        Text("// SAFETY: ptr is valid\n\
         pub unsafe fn Read_Raw(ptr: *const u8) -> u8\n{\n    return *ptr;\n}\n"),
    );

    let findings = Check_Unsafe_Justification(&[source]);

    assert_eq!(
        findings.len(),
        1,
        "a declaration's own doc-comment site is the artifact this rule asks for, not an adjacent // comment: {findings:?}"
    );
}

#[test]
fn Test_Check_Unsafe_Justification_Should_Report_A_Bare_Safety_Marker_With_No_Reason()
{
    let source = Source(
        Path("src/lib.rs",),
        Text("// SAFETY:\n\
         let slice = unsafe { core::slice::from_raw_parts(ptr, len) };\n"),
    );

    let findings = Check_Unsafe_Justification(&[source]);

    assert_eq!(findings.len(), 1, "a marker with nothing after it must not satisfy the rule: {findings:?}");
}

#[test]
fn Test_Check_Inline_Always_Justification_Should_Report_An_Unexplained_Inline_Always()
{
    let source = Source(Path("src/lib.rs"), Text("#[inline(always)]\npub fn Sample_Texel() -> Color { todo!() }\n"));

    let findings = Check_Inline_Always_Justification(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(INLINE_ALWAYS_JUSTIFICATION));
}

#[test]
fn Test_Check_Inline_Always_Justification_Should_Accept_An_Explained_Inline_Always()
{
    let source = Source(Path("src/lib.rs"), Text("// hot path, measured 8% improvement in benches/hot_path.rs\n#[inline(always)]\npub fn Sample_Texel() -> Color { todo!() }\n"));

    let findings = Check_Inline_Always_Justification(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

/// `P69-SELF-MATCH-VIA-STRING-LITERALS-FIVE-MORE-RULES`: a string literal's own prose
/// naming the attribute is not the attribute.
#[test]
fn Test_Check_Inline_Always_Justification_Should_Not_Judge_A_String_Literals_Own_Text()
{
    let source = Source(Path("src/lib.rs"), Text("let reason = \"no #[inline(always)] attribute in the fixture\";\n"));

    let findings = Check_Inline_Always_Justification(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Disabled_Test_States_Why_Should_Report_A_Bare_Ignore()
{
    let source = Source(Path("tests/lib.rs"), Text("#[test]\n#[ignore]\nfn Test_Something() {}\n"));

    let findings = Check_A_Disabled_Test_States_Why(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(A_DISABLED_TEST_STATES_WHY));
}

#[test]
fn Test_Check_A_Disabled_Test_States_Why_Should_Accept_An_Ignore_With_A_Reason_Value()
{
    let source = Source(Path("tests/lib.rs"), Text("#[test]\n#[ignore = \"needs a GPU adapter; no headless runner has one\"]\nfn Test_Something() {}\n"));

    let findings = Check_A_Disabled_Test_States_Why(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Disabled_Test_States_Why_Should_Accept_A_Preceding_Comment()
{
    let source = Source(Path("tests/lib.rs"), Text("#[test]\n// flaky under -race, see #88\n#[ignore]\nfn Test_Something() {}\n"));

    let findings = Check_A_Disabled_Test_States_Why(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

/// `P69-SELF-MATCH-VIA-STRING-LITERALS-FIVE-MORE-RULES`: a string literal's own prose
/// naming the attribute is not the attribute.
#[test]
fn Test_Check_A_Disabled_Test_States_Why_Should_Not_Judge_A_String_Literals_Own_Text()
{
    let source = Source(Path("src/lib.rs"), Text("let reason = \"there is no #[ignore] attribute for this rule to examine\";\n"));

    let findings = Check_A_Disabled_Test_States_Why(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Unsafe_Justification_Should_Not_Judge_Its_Own_Implementation_File()
{
    let source = Source(Path("crates/rules/nomos-rules/src/checks/rust_text.rs"), Text("let slice = unsafe { core::slice::from_raw_parts(ptr, len) };\n"));

    let findings = Check_Unsafe_Justification(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

/// The exemption covers the whole module this rule was split into, not only the file it
/// originally named: the child a rule's own detection patterns and fixtures moved into is
/// exactly as self-matching as the file they were written in.
#[test]
fn Test_Check_Unsafe_Justification_Should_Not_Judge_A_Child_Of_Its_Own_Implementation_Module()
{
    let path = "crates/rules/nomos-rules/src/checks/rust_text/comment_justified.rs";
    let source = Source(Path(path), Text("let slice = unsafe { core::slice::from_raw_parts(ptr, len) };\n"));

    let findings = Check_Unsafe_Justification(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

/// `P45-CODE-PREFIX-KNOWS-STRINGS-2`'s real third defect: two files carrying the
/// identical content, one at this crate's own real path and one at a path that merely
/// ends the same way, must get identical verdicts — a path-suffix self-exemption a
/// stranger's repository could reproduce is not a real self-exemption.
#[test]
fn Test_Check_Unsafe_Justification_Should_Judge_A_Path_That_Only_Ends_Like_The_Own_Implementation_File()
{
    let text = "let slice = unsafe { core::slice::from_raw_parts(ptr, len) };\n";
    let real = Source(Path("crates/rules/nomos-rules/src/checks/rust_text.rs"), Text(text));
    let spoofed = Source(Path("vendored/crates/rules/nomos-rules/src/checks/rust_text.rs"), Text(text));

    let real_findings = Check_Unsafe_Justification(&[real]);
    let spoofed_findings = Check_Unsafe_Justification(&[spoofed]);

    assert!(real_findings.is_empty(), "{real_findings:?}");
    assert_eq!(spoofed_findings.len(), 1, "a suffix match is not this crate's own implementation file: {spoofed_findings:?}");
}

/// The fixture's path position, named so a call site cannot transpose it with the text.
struct Path<'a>(&'a str);

/// The fixture's text position, named so a call site cannot transpose it with the path.
struct Text<'a>(&'a str);

fn Source(path: Path<'_>, text: Text<'_>) -> SourceFile
{
    let mut source = SourceFile::New(path.0, SubjectId::From_Digest(Content_Digest(path.0.as_bytes())), text.0);
    source.language = crate::Recognized_Language_In_Tests(path.0);
    return source;
}

/// Runs `check` over `source` through a real, empty reader — these checks read only
/// `nomos.cap.test.material.policy`, which no fixture here declares, so `Require` fails and
/// each resolves to its own fixed clauses alone.
fn Check(check: fn(&[SourceFile], &mut dyn FactReader) -> Vec<Finding>, source: SourceFile) -> Vec<Finding>
{
    let store = MemoryFactStore::New();
    let registry = Registry::New();
    let mut facts = Reader::On(&store, &registry, crate::checks::test_support::Test_Context());
    return check(&[source], &mut facts);
}
