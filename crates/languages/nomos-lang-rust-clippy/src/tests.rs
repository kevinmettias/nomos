//! Every reader `Discover_Workspace` composes, exercised against synthetic and one real,
//! captured `cargo clippy --message-format=json` diagnostic. `Discover_Workspace` itself is
//! tested inline, in `local_tests` beside it in `clippy_error.rs` — this file's own tests need
//! Unit-scoped coverage attribution to reach a function declared in a different file, and this
//! codebase's coverage rule keys attribution to the file a test physically lives in.

use super::*;

/// The primary span line/column `Real_Compiler_Message`'s fixture reproduces from the
/// real capture, and the value `Test_A_Real_Captured_Diagnostic_Should_Parse` checks
/// its parsed `line` against — one constant, so the two can never independently drift.
const CAPTURED_LINE_START: u32 = 113;
const CAPTURED_LINE_END: u32 = CAPTURED_LINE_START;
const CAPTURED_COLUMN_START: u32 = 9;
const CAPTURED_COLUMN_END: u32 = 12;

#[test]
fn Test_A_Registry_Package_Id_Should_Not_Resolve()
{
    let root = Path::new("F:/repos/nomos");
    let id = "registry+https://github.com/rust-lang/crates.io-index#serde@1.0.229";

    assert_eq!(First_Party_Relative_Root(id, root), None);
}

#[test]
fn Test_A_First_Party_Package_Id_Should_Resolve_Relative_To_Root()
{
    for (root, id, expected) in Windows_First_Party_Package_Id_Cases()
    {
        assert_eq!(First_Party_Relative_Root(id, Path::new(root)), Some(expected.to_owned()));
    }
}

/// (root, `path+file://` package id, expected relative root) for a Windows-style first
/// party package id — a second case beside this one would extend the table rather than
/// duplicate the test.
fn Windows_First_Party_Package_Id_Cases() -> Vec<(&'static str, &'static str, &'static str)>
{
    return vec![(
        "F:/repos/nomos",
        "path+file:///F:/repos/nomos/crates/substrate/nomos-ledger#0.1.0",
        "crates/substrate/nomos-ledger",
    )];
}

#[test]
fn Test_A_Posix_Package_Id_Should_Resolve_Without_A_Drive_Letter()
{
    for (root, id, expected) in Posix_First_Party_Package_Id_Cases()
    {
        assert_eq!(First_Party_Relative_Root(id, Path::new(root)), Some(expected.to_owned()));
    }
}

/// (root, `path+file://` package id, expected relative root) for a POSIX-style first
/// party package id, which carries no drive letter to strip.
fn Posix_First_Party_Package_Id_Cases() -> Vec<(&'static str, &'static str, &'static str)>
{
    return vec![(
        "/home/build/nomos",
        "path+file:///home/build/nomos/crates/substrate/nomos-ledger#0.1.0",
        "crates/substrate/nomos-ledger",
    )];
}

/// `P68-SUBPROCESS-PROVIDERS-ESCAPE-A-NESTED-ROOT`: a package `root` cannot relativize
/// against must be excluded, not folded in under its own absolute path -- the fallback
/// this test used to assert the *opposite* of, back when `root` not being an ancestor of
/// `id` was read as "an unrelated root, so report the package anyway" rather than as the
/// exact shape of the escape this reader now exists to catch. `Discover_Workspace`'s own
/// `Absolutized` step is what makes a genuinely relative root (`.`) resolve to a real
/// ancestor in practice; this unit test calls `First_Party_Relative_Root` directly and so
/// never absolutizes `root` itself, which is exactly why `.` and an unrelated absolute `id`
/// must disagree here.
#[test]
fn Test_A_First_Party_Package_Id_Outside_Root_Should_Be_Excluded_Rather_Than_Included_Under_Its_Absolute_Path()
{
    let root = Path::new(".");
    let id = "path+file:///F:/repos/nomos/crates/substrate/nomos-ledger#0.1.0";

    assert_eq!(
        First_Party_Relative_Root(id, root),
        None,
        "a package root cannot relativize -- the escape this item measured directly -- must \
         be excluded, not read as first-party under its own absolute path"
    );
}

#[test]
fn Test_Package_Name_Reads_The_Last_Path_Segment()
{
    assert_eq!(Package_Name("crates/substrate/nomos-ledger"), "nomos-ledger");
}

#[test]
fn Test_A_Real_Captured_Diagnostic_Should_Parse()
{
    let message = Real_Compiler_Message();
    let diagnostic = Diagnostic_Of(message.get("message").expect("captured fixture has a message"))
        .expect("a real compiler-message with a primary span must parse");

    assert_eq!(diagnostic.level, LintLevel::Warning);
    assert_eq!(diagnostic.lint.as_deref(), Some("clippy::similar_names"));
    assert_eq!(diagnostic.message, "binding's name is too similar to existing binding");
    assert_eq!(diagnostic.file, "crates/substrate/nomos-ledger/src/finish.rs");
    assert_eq!(diagnostic.line, CAPTURED_LINE_START);
}

/// A real, captured `cargo clippy --message-format=json` diagnostic, from this
/// workspace's own output over `nomos-ledger` before this reader existed to parse it —
/// not invented, so a change to `rustc`'s own JSON shape is caught here rather than
/// only against a fixture written to already agree with this code.
///
/// The primary span's bounds are the exact ones that capture recorded; naming them
/// keeps `Test_A_Real_Captured_Diagnostic_Should_Parse`'s own assertion honest about
/// comparing against the same fixture value rather than a second, independently typed
/// `113`.
fn Real_Compiler_Message() -> serde_json::Value
{
    return serde_json::json!({
        "reason": "compiler-message",
        "package_id": "path+file:///F:/repos/nomos/crates/substrate/nomos-ledger#0.1.0",
        "message": {
            "rendered": "warning: binding's name is too similar to existing binding\n",
            "$message_type": "diagnostic",
            "children": [],
            "code": { "code": "clippy::similar_names", "explanation": null },
            "level": "warning",
            "message": "binding's name is too similar to existing binding",
            "spans": [
                {
                    "file_name": "crates\\substrate\\nomos-ledger\\src\\finish.rs",
                    "line_start": CAPTURED_LINE_START,
                    "line_end": CAPTURED_LINE_END,
                    "column_start": CAPTURED_COLUMN_START,
                    "column_end": CAPTURED_COLUMN_END,
                    "is_primary": true
                }
            ]
        }
    });
}

#[test]
fn Test_A_Message_With_No_Primary_Span_Should_Not_Parse()
{
    let message = serde_json::json!({
        "level": "help",
        "message": "for further information visit https://...",
        "code": null,
        "spans": []
    });

    assert_eq!(Diagnostic_Of(&message), None);
}

#[test]
fn Test_A_Message_With_No_Lint_Code_Should_Still_Parse()
{
    let message = serde_json::json!({
        "level": "error",
        "message": "mismatched types",
        "code": null,
        "spans": [{ "file_name": "a.rs", "line_start": 1, "is_primary": true }]
    });

    let diagnostic = Diagnostic_Of(&message).expect("a code-less diagnostic still names a level, message and span");

    assert_eq!(diagnostic.lint, None);
}

#[test]
fn Test_Grouped_By_Package_Reports_A_Clean_Member_With_No_Diagnostics()
{
    let root = Path::new("F:/repos/nomos");
    let stdout = serde_json::json!({
        "reason": "compiler-artifact",
        "package_id": "path+file:///F:/repos/nomos/crates/contracts/nomos-contracts#0.1.0",
        "target": { "kind": ["lib"] }
    })
    .to_string();

    let discovered = Grouped_By_Package(&stdout, root);

    assert_eq!(discovered.len(), 1, "{discovered:?}");
    let member = discovered.first().expect("asserted len 1 above");
    assert_eq!(member.payload.package, "nomos-contracts");
    assert!(member.payload.diagnostics.is_empty(), "an artifact with no message is a clean report, not an absent one");
}

#[test]
fn Test_A_Malformed_Json_Line_Should_Be_Skipped_Rather_Than_Fail_The_Whole_Stream()
{
    let root = Path::new("F:/repos/nomos");
    let stdout = "not json at all\n";

    assert!(Grouped_By_Package(stdout, root).is_empty());
}

const DUPLICATE_MESSAGE_LINE: u32 = 5;

#[test]
fn Test_Duplicate_Diagnostics_From_Two_Target_Compiles_Should_Collapse_To_One()
{
    let root = Path::new("F:/repos/nomos");
    let id = "path+file:///F:/repos/nomos/crates/rules/nomos-rules#0.1.0";
    let stdout = format!("{}\n{}\n", Duplicate_Message_Line(id), Duplicate_Message_Line(id));

    let discovered = Grouped_By_Package(&stdout, root);

    assert_eq!(discovered.len(), 1);
    assert_eq!(
        discovered.first().expect("asserted len 1 above").payload.diagnostics.len(),
        1,
        "the same diagnostic reported by two target compiles must collapse to one"
    );
}

/// One `compiler-message` line naming `package_id`'s own warning at
/// [`DUPLICATE_MESSAGE_LINE`] -- `--all-targets` emits this shape once per compilation that
/// reaches the same source line, which is the duplicate the test above collapses.
fn Duplicate_Message_Line(package_id: &str) -> String
{
    return serde_json::json!({
        "reason": "compiler-message",
        "package_id": package_id,
        "message": {
            "level": "warning",
            "message": "unneeded return statement",
            "code": { "code": "clippy::needless_return" },
            "spans": [{ "file_name": "src/lib.rs", "line_start": DUPLICATE_MESSAGE_LINE, "is_primary": true }]
        }
    })
    .to_string();
}
