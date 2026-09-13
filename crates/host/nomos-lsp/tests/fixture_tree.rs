//! `Diagnostics_For` over a real `nomos_check_orchestration::Run` result -- not a
//! hand-built `Finding`, so this proves the whole translation reaches a real judgment
//! rather than only the shape this crate's own unit tests assert `Finding` fields into.
//!
//! `Walked` is a small, local, top-level-only walk -- the same shape
//! `nomos-correction-orchestration::run`'s own test module carries for the identical
//! reason: `nomos_lsp::sources` is `pub(crate)`, by design, since nothing outside this
//! crate's own provider should walk a tree the way this crate does.

use nomos_contracts::RuleId;
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;
use std::path::{Path, PathBuf};

fn Test_Variant() -> BuildVariant
{
    return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
}

fn Walked(root: &Path) -> Vec<SourceFile>
{
    let mut sources = Vec::new();
    for entry in std::fs::read_dir(root).expect("the fixture root is readable").flatten()
    {
        let path = entry.path();
        if path.extension().is_some_and(|extension| return extension == "rs")
        {
            let text = std::fs::read_to_string(&path).expect("readable");
            let relative = path.strip_prefix(root).expect("under root").display().to_string();
            sources.push(SourceFile::New(relative.clone(), nomos_model::Subject_Of_Path(&relative), text));
        }
    }
    return sources;
}

fn Fresh_Root(name: &str) -> PathBuf
{
    let root = std::env::temp_dir().join(name);
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the temporary root is creatable");
    return root;
}

/// A real trailing-whitespace violation, judged by the real rule and translated by this
/// crate's own public `Diagnostics_For` -- the fixture-tree proof this crate's own report
/// names: the translation function reaches a real `Run` result, not only hand-built
/// `Finding`s.
#[test]
fn Test_Diagnostics_For_Should_Translate_A_Real_Run_Result()
{
    let root = Fresh_Root("nomos-lsp-fixture-tree-trailing-whitespace");
    std::fs::write(root.join("a.rs"), "pub fn Something() -> u32 \n{\n    return 1;\n}\n").expect("writable");

    let sources = Walked(&root);
    let selected = [RuleId::New(nomos_rules::NO_TRAILING_WHITESPACE)];
    let mut workspace = None;
    let mut store = nomos_analysis::MemoryFactStore::New();

    let outcome = nomos_check_orchestration::Run(
        &sources,
        nomos_check_orchestration::RunContext {
            variant: Test_Variant(),
            root: &root,
            launcher: &nomos_composer_std::LAUNCHER,
            filesystem: &nomos_composer_std::FILE_SYSTEM,
            environment: &nomos_composer_std::ENVIRONMENT,
            workspace: &mut workspace,
            store: &mut store,
        },
        &selected,
    );

    let _ignored = std::fs::remove_dir_all(&root);

    let nomos_check_orchestration::CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        panic!("expected a judged outcome: {outcome:?}");
    };
    assert_eq!(findings.len(), 1, "one line carries trailing whitespace: {findings:?}");

    let only_finding = findings.first().expect("asserted len 1 above");
    let diagnostics = nomos_lsp::Diagnostics_For(only_finding);
    let only = diagnostics.first().expect("one location, one diagnostic");

    assert_eq!(only.path, "a.rs");
    // One-based, as every rule in this workspace reports it. Turning that into a zero-based
    // editor position is the engine's, at projection time, and is asserted there.
    assert_eq!(only.line, Some(1), "the trailing whitespace is on the fixture's first line");
    assert_eq!(only.severity, xvpe_diagnostics::DiagnosticSeverity::Error, "no-trailing-whitespace is a real blocking rule");
    assert_eq!(only.code, nomos_rules::NO_TRAILING_WHITESPACE);

    let carried = only.detail.as_ref().expect("walk-outward data is attached");
    let data: serde_json::Value = serde_json::from_str(carried).expect("walk-outward data is a document");
    assert_eq!(At(&data, "governing_rule", "rule"), Some(nomos_rules::NO_TRAILING_WHITESPACE));
    assert_eq!(At(&data, "available_correction", "family"), Some(nomos_rules::NO_TRAILING_WHITESPACE));
}

/// `data.first.second`, read through `Value::get` rather than `Value`'s own `Index` --
/// this workspace's `indexing_slicing` lint policy denies the bracket form everywhere, not
/// only on a slice.
fn At<'a>(data: &'a serde_json::Value, first: &str, second: &str) -> Option<&'a str>
{
    return data.get(first)?.get(second)?.as_str();
}
