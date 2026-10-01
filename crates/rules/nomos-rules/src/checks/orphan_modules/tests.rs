use super::*;
use nomos_contracts::SubjectId;
use nomos_model::Content_Digest;

#[test]
fn Test_Check_No_Orphan_Modules_Should_Report_A_File_No_Declaration_Names()
{
    let sources = vec![Source("demo/src/lib.rs", Text(&[])), Source("demo/src/stray.rs", Text(&[]))];

    let findings = Check_No_Orphan_Modules(&sources);

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.rule, RuleId::New(NO_ORPHAN_MODULES));
    assert_eq!(found.gate, GateCategory::Blocking);
    assert_eq!(found.subject_name, "demo/src/stray.rs");
}

#[test]
fn Test_Check_No_Orphan_Modules_Should_Accept_A_Declared_Sibling_File()
{
    let sources = vec![
        Source("demo/src/lib.rs", Text(&["pub mod reached;"])),
        Source("demo/src/reached.rs", Text(&[])),
    ];

    let findings = Check_No_Orphan_Modules(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_No_Orphan_Modules_Should_Accept_A_Directory_Module_File()
{
    let sources = vec![
        Source("demo/src/lib.rs", Text(&["mod legacy;"])),
        Source("demo/src/legacy/mod.rs", Text(&["mod inner;"])),
        Source("demo/src/legacy/inner.rs", Text(&[])),
    ];

    let findings = Check_No_Orphan_Modules(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_No_Orphan_Modules_Should_Accept_Every_Visibility_Spelling()
{
    let sources = vec![
        Source(
            "demo/src/lib.rs",
            Text(&["mod plain;", "pub mod exported;", "pub(crate) mod crate_wide;", "pub (super) mod parental;"]),
        ),
        Source("demo/src/plain.rs", Text(&[])),
        Source("demo/src/exported.rs", Text(&[])),
        Source("demo/src/crate_wide.rs", Text(&[])),
        Source("demo/src/parental.rs", Text(&[])),
    ];

    let findings = Check_No_Orphan_Modules(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

/// The raw-identifier prefix is spelling, not name: the file backing it drops the prefix.
/// Missing this reported a live, compiling file as an orphan in the tool this ports.
#[test]
fn Test_Check_No_Orphan_Modules_Should_Resolve_A_Raw_Identifier_To_The_Unprefixed_File()
{
    let sources = vec![
        Source("demo/src/lib.rs", Text(&["mod r#match;"])),
        Source("demo/src/match.rs", Text(&[])),
    ];

    let findings = Check_No_Orphan_Modules(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

/// A module written with a body is backed by no file, so a same-named file beside it is
/// still an orphan.
#[test]
fn Test_Check_No_Orphan_Modules_Should_Not_Let_An_Inline_Module_Reach_A_File()
{
    let sources = vec![
        Source("demo/src/lib.rs", Text(&["mod inline", "{", "}"])),
        Source("demo/src/inline.rs", Text(&[])),
    ];

    let findings = Check_No_Orphan_Modules(&sources);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "demo/src/inline.rs");
}

#[test]
fn Test_Check_No_Orphan_Modules_Should_Not_Let_A_Commented_Declaration_Reach_A_File()
{
    let sources = vec![
        Source("demo/src/lib.rs", Text(&["// mod disabled;"])),
        Source("demo/src/disabled.rs", Text(&[])),
    ];

    let findings = Check_No_Orphan_Modules(&sources);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "demo/src/disabled.rs");
}

/// An identifier that merely opens with the same three letters is not the keyword.
#[test]
fn Test_Check_No_Orphan_Modules_Should_Not_Read_A_Longer_Word_As_The_Keyword()
{
    let sources = vec![
        Source("demo/src/lib.rs", Text(&["modes;"])),
        Source("demo/src/modes.rs", Text(&[])),
    ];

    let findings = Check_No_Orphan_Modules(&sources);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "demo/src/modes.rs");
}

/// The shape that reported all thirty-nine files of one live crate as orphans: the
/// attribute resolves against the directory the DECLARING file sits in, and the file it
/// names then owns the directory it LANDS in, so that file's own children are its
/// siblings rather than entries in a subdirectory named after it.
#[test]
fn Test_Check_No_Orphan_Modules_Should_Give_A_Path_Attribute_Target_The_Directory_It_Lands_In()
{
    let sources = vec![
        Source(
            "demo/src/lib.rs",
            Text(&["#[path = \"action_model/action_behavior/facade.rs\"]", "pub mod action_behavior;"]),
        ),
        Source("demo/src/action_model/action_behavior/facade.rs", Text(&["mod sibling;"])),
        Source("demo/src/action_model/action_behavior/sibling.rs", Text(&[])),
    ];

    let findings = Check_No_Orphan_Modules(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_No_Orphan_Modules_Should_Still_Report_A_Stray_Beside_A_Path_Attribute_Target()
{
    let sources = vec![
        Source("demo/src/lib.rs", Text(&["#[path = \"nested/facade.rs\"]", "mod facade;"])),
        Source("demo/src/nested/facade.rs", Text(&[])),
        Source("demo/src/nested/stray.rs", Text(&[])),
    ];

    let findings = Check_No_Orphan_Modules(&sources);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "demo/src/nested/stray.rs");
}

#[test]
fn Test_Check_No_Orphan_Modules_Should_Read_A_Binary_Target_As_Its_Own_Root()
{
    let sources = vec![
        Source("demo/src/lib.rs", Text(&[])),
        Source("demo/src/bin/tool.rs", Text(&["mod helper;"])),
        Source("demo/src/bin/helper.rs", Text(&[])),
    ];

    let findings = Check_No_Orphan_Modules(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_No_Orphan_Modules_Should_Read_A_Main_File_As_A_Root()
{
    let sources = vec![
        Source("demo/src/main.rs", Text(&["mod engine;"])),
        Source("demo/src/engine.rs", Text(&[])),
    ];

    let findings = Check_No_Orphan_Modules(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

/// A test target, a build script and a Go file all sit outside every crate's source
/// tree, so none of them has a module tree to be judged against.
#[test]
fn Test_Check_No_Orphan_Modules_Should_Ignore_Anything_Outside_A_Source_Tree()
{
    let sources = vec![
        Source("demo/src/lib.rs", Text(&[])),
        Source("demo/tests/integration.rs", Text(&[])),
        Source("demo/build.rs", Text(&[])),
        Source("demo/src/tool.go", Text(&[])),
    ];

    let findings = Check_No_Orphan_Modules(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

/// One crate's roots must not reach another's files, and each crate is judged from its
/// own source tree.
#[test]
fn Test_Check_No_Orphan_Modules_Should_Judge_Each_Crate_Against_Its_Own_Roots()
{
    let sources = vec![
        Source("first/src/lib.rs", Text(&["mod shared;"])),
        Source("first/src/shared.rs", Text(&[])),
        Source("second/src/lib.rs", Text(&[])),
        Source("second/src/shared.rs", Text(&[])),
    ];

    let findings = Check_No_Orphan_Modules(&sources);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "second/src/shared.rs");
}

/// A source tree with no root at all reaches nothing, which is the same verdict the tool
/// this ports gives a crate whose manifest declares a target it does not carry.
#[test]
fn Test_Check_No_Orphan_Modules_Should_Report_Every_File_Of_A_Rootless_Source_Tree()
{
    let sources = vec![Source("demo/src/one.rs", Text(&[])), Source("demo/src/two.rs", Text(&[]))];

    let findings = Check_No_Orphan_Modules(&sources);

    assert_eq!(findings.len(), sources.len(), "{findings:?}");
}

#[test]
fn Test_Check_No_Orphan_Modules_Should_Report_In_Path_Order()
{
    let sources = vec![
        Source("demo/src/lib.rs", Text(&[])),
        Source("demo/src/zulu.rs", Text(&[])),
        Source("demo/src/alpha.rs", Text(&[])),
    ];

    let findings = Check_No_Orphan_Modules(&sources);

    let reported: Vec<&str> = findings.iter().map(|finding| return finding.subject_name.as_str()).collect();
    assert_eq!(reported, vec!["demo/src/alpha.rs", "demo/src/zulu.rs"]);
}

/// A declaration nothing backs is a compile error rustc reports far better than this
/// would, so it contributes no finding of its own.
#[test]
fn Test_Check_No_Orphan_Modules_Should_Ignore_A_Declaration_Backed_By_No_File()
{
    let sources = vec![Source("demo/src/lib.rs", Text(&["mod absent;"]))];

    let findings = Check_No_Orphan_Modules(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Joined_Path_Should_Refuse_A_Climb_Above_Its_Own_Root()
{
    assert_eq!(Joined_Path(Directory("demo"), "../../escaped.rs"), None);
    assert_eq!(Joined_Path(Directory("demo/src"), "../shared/held.rs"), Some("demo/shared/held.rs".to_owned()));
}

#[test]
fn Test_Source_Directory_Of_Should_Name_Nothing_For_A_Bare_Source_Directory()
{
    assert_eq!(Source_Directory_Of("demo/src"), None);
    assert_eq!(Source_Directory_Of("demo/src/lib.rs"), Some("demo/src"));
    assert_eq!(Source_Directory_Of("demo/tests/main.rs"), None);
}

fn Text(lines: &[&str]) -> String
{
    return lines.join("\n");
}

fn Source(path: &str, text: String) -> SourceFile
{
    let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
    source.language = crate::Recognized_Language_In_Tests(path);
    return source;
}
