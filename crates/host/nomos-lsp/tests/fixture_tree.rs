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

/// A real trailing-whitespace violation, judged by the real rule and translated by this
/// crate's own public `Diagnostics_For` -- the fixture-tree proof this crate's own report
/// names: the translation function reaches a real `Run` result, not only hand-built
/// `Finding`s.
#[test]
fn Test_Diagnostics_For_Should_Translate_A_Real_Run_Result()
{
    let root = Fresh_Root("nomos-lsp-fixture-tree-trailing-whitespace");
    std::fs::write(root.join("a.rs"), "pub fn Something() -> u32 \n{\n    return 1;\n}\n").expect("the fresh root above was just created");

    let sources = Walked(&root);
    let outcome = Judged_Over(&root, &sources);

    let _ignored = std::fs::remove_dir_all(&root);

    let nomos_check_orchestration::CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        panic!("expected a judged outcome: {outcome:?}");
    };
    assert_eq!(findings.len(), 1, "one line carries trailing whitespace: {findings:?}");

    let only_finding = findings.first().expect("asserted len 1 above");
    let diagnostics = nomos_lsp::Diagnostics_For(&nomos_cap_architecture::ArchitecturePayload::default(), only_finding);
    let only = diagnostics.first().expect("one location, one diagnostic");

    Assert_One_Translation(only);
}

fn Fresh_Root(name: &str) -> PathBuf
{
    let root = std::env::temp_dir().join(name);
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the temporary root is creatable");
    return root;
}

fn Walked(root: &Path) -> Vec<SourceFile>
{
    let mut sources = Vec::new();
    for entry in std::fs::read_dir(root).expect("the fixture root is readable").flatten()
    {
        let path = entry.path();
        if path.extension().is_some_and(|extension| return extension == "rs")
        {
            let text = std::fs::read_to_string(&path).expect("read_dir returned this entry, so the file it names is readable");
            let relative = path.strip_prefix(root).expect("the entry was read from this root, so its path begins with it").display().to_string();
            let subject = nomos_model::Subject_Of_Path(&relative);
            let source = SourceFile::New(relative, subject, text);
            sources.push(source);
        }
    }
    return sources;
}

/// The outcome [`nomos_check_orchestration::Run`] reaches over `sources` under `root` with only
/// `no-trailing-whitespace` selected: a workspace and a fact store that live for this one call,
/// which is what the unreassessed seam itself does.
fn Judged_Over(root: &Path, sources: &[SourceFile]) -> nomos_check_orchestration::CheckOutcome
{
    let mut workspace = None;
    let mut store = nomos_analysis::MemoryFactStore::New();

    return nomos_check_orchestration::Run(
        sources,
        nomos_check_orchestration::RunContext {
            variant: Test_Variant(),
            root,
            launcher: &nomos_composer_std::LAUNCHER,
            filesystem: &nomos_composer_std::FILE_SYSTEM,
            environment: &nomos_composer_std::ENVIRONMENT,
            workspace: &mut workspace,
            store: &mut store,
        },
        &[RuleId::New(nomos_rules::NO_TRAILING_WHITESPACE)],
    );
}

fn Test_Variant() -> BuildVariant
{
    return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
}

/// The claims the fixture's one translated diagnostic makes: it sits where the rule reported the
/// violation, speaks at `Error`, carries the rule as its code, and carries a walk-outward
/// document naming the same rule as the governing one and as the correction family.
fn Assert_One_Translation(only: &xvpe_diagnostics::SourceDiagnostic)
{
    assert_eq!(only.path, "a.rs");
    // One-based, as every rule in this workspace reports it. Turning that into a zero-based
    // editor position is the engine's, at projection time, and is asserted there.
    assert_eq!(only.line, Some(1), "the trailing whitespace is on the fixture's first line");
    assert_eq!(only.severity, xvpe_diagnostics::DiagnosticSeverity::Error, "no-trailing-whitespace is a real blocking rule");
    assert_eq!(only.code, nomos_rules::NO_TRAILING_WHITESPACE);

    let carried = only.detail.as_ref().expect("walk-outward data is attached");
    let data: serde_json::Value = serde_json::from_str(carried).expect("walk-outward data is a document");
    assert_eq!(At(&data, &["governing_rule", "rule"]), Some(nomos_rules::NO_TRAILING_WHITESPACE));
    assert_eq!(At(&data, &["available_correction", "family"]), Some(nomos_rules::NO_TRAILING_WHITESPACE));
}

/// The string at the end of `path` in `data`, read through `Value::get` rather than `Value`'s own
/// `Index` -- this workspace's `indexing_slicing` lint policy denies the bracket form everywhere,
/// not only on a slice.
fn At<'a>(data: &'a serde_json::Value, path: &[&str]) -> Option<&'a str>
{
    let mut cursor = data;
    for step in path
    {
        cursor = cursor.get(step)?;
    }

    return cursor.as_str();
}
