//! `Discover_Workspace` against real files, written to a fresh, unique temp directory per
//! test and removed afterward — a real filesystem read, not a stubbed one, the same
//! standard `nomos_lang_rust_cargo::metadata`'s own tests hold themselves to against a
//! real `cargo metadata`.

use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A fresh, uniquely-named directory under the system temp directory, so concurrently
/// running tests in this same process never collide.
struct TemporaryWorkspace
{
    root: PathBuf,
}

impl TemporaryWorkspace
{
    fn New() -> Self
    {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "nomos-lang-go-modules-test-{}-{n}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).expect("a fresh temp directory can be created");

        return Self { root };
    }

    /// `relative` is a `&Path` rather than a `&str` so the two positions have distinct
    /// types: a caller that swapped the fixture's path for its content would be writing
    /// a file named after the module source, and only the type system would say so.
    fn Write(&self, relative: &Path, content: &str)
    {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent()
        {
            std::fs::create_dir_all(parent).expect("the fixture's own parent directory can be created");
        }
        std::fs::write(&path, content).expect("the fixture file can be written");
    }
}

impl Drop for TemporaryWorkspace
{
    /// Drop cannot propagate a failure, so a failed removal is reported rather than
    /// silently discarded; each root is uniquely named, so the worst case is one leaked
    /// directory, not a sibling test's fixture disappearing out from under it.
    fn drop(&mut self)
    {
        if let Err(error) = std::fs::remove_dir_all(&self.root)
        {
            eprintln!("failed to remove temporary workspace {}: {error}", self.root.display());
        }
    }
}

fn Payload_Of<'a>(discovered: &'a [DiscoveredModule], package: &str) -> &'a DependencyPayload
{
    return &discovered
        .iter()
        .find(|module| module.payload.package == package)
        .unwrap_or_else(|| panic!("expected a module named {package} among {discovered:?}"))
        .payload;
}

#[test]
fn Test_Discover_Workspace_Should_Treat_A_Single_Module_With_No_Go_Work_As_Its_Own_Whole_Workspace()
{
    let workspace = TemporaryWorkspace::New();
    workspace.Write(
        Path::new("go.mod"),
        "module example.com/solo\n\ngo 1.21\n\nrequire github.com/external/thing v1.0.0\n",
    );

    let discovered = Discover_Workspace(&workspace.root).expect("a real single-module workspace");

    assert_eq!(discovered.len(), 1);
    let payload = Payload_Of(&discovered, "example.com/solo");
    assert!(
        payload.edges.is_empty(),
        "an external requirement is not a workspace member: {:?}",
        payload.edges
    );
}

/// A two-member workspace whose `a` requires both `b` and an external module, and whose
/// `b` requires nothing — the graph the edge test below reads.
fn Two_Member_Workspace_With_An_External_Requirement() -> TemporaryWorkspace
{
    let workspace = TemporaryWorkspace::New();
    workspace.Write(Path::new("go.work"), "go 1.21\n\nuse (\n\t./a\n\t./b\n)\n");
    workspace.Write(
        Path::new("a/go.mod"),
        "module example.com/a\n\ngo 1.21\n\nrequire (\n\texample.com/b v0.0.0\n\tgithub.com/external/thing v1.0.0\n)\n",
    );
    workspace.Write(Path::new("b/go.mod"), "module example.com/b\n\ngo 1.21\n");

    return workspace;
}

/// This fixture declares two members, `a` and `b`, so a discovery over it finds two.
const TWO_WORKSPACE_MEMBERS: usize = 2;

#[test]
fn Test_A_Workspace_Member_Requiring_Another_Should_Carry_The_Edge()
{
    let workspace = Two_Member_Workspace_With_An_External_Requirement();

    let discovered = Discover_Workspace(&workspace.root).expect("a real two-module workspace");

    assert_eq!(discovered.len(), TWO_WORKSPACE_MEMBERS);
    let a = Payload_Of(&discovered, "example.com/a");
    assert_eq!(
        a.edges,
        vec![DependencyEdge {
            target: "example.com/b".to_owned(),
            kind: DependencyKind::Normal,
            optional: false,
        }],
        "the external requirement must not appear: {:?}",
        a.edges
    );
    let b = Payload_Of(&discovered, "example.com/b");
    assert!(b.edges.is_empty());
}

#[test]
fn Test_An_Indirect_Requirement_Of_A_Member_Should_Still_Be_An_Edge()
{
    let workspace = TemporaryWorkspace::New();
    workspace.Write(Path::new("go.work"), "use ./a\nuse ./b\n");
    workspace.Write(
        Path::new("a/go.mod"),
        "module example.com/a\n\nrequire example.com/b v0.0.0 // indirect\n",
    );
    workspace.Write(Path::new("b/go.mod"), "module example.com/b\n");

    let discovered = Discover_Workspace(&workspace.root).expect("a real two-module workspace");

    let a = Payload_Of(&discovered, "example.com/a");
    assert_eq!(a.edges.len(), 1, "a trailing // indirect comment must not hide the edge");
}

#[test]
fn Test_Manifest_Relative_Root_Should_Be_Repository_Relative_And_Forward_Sloshed()
{
    let workspace = TemporaryWorkspace::New();
    workspace.Write(Path::new("go.work"), "use (\n\t./nested/a\n)\n");
    workspace.Write(Path::new("nested/a/go.mod"), "module example.com/a\n");

    let discovered = Discover_Workspace(&workspace.root).expect("a real workspace");

    let module = discovered
        .iter()
        .find(|module| module.payload.package == "example.com/a")
        .expect("the nested module was discovered");
    assert_eq!(module.manifest_relative_root, "nested/a");
}

#[test]
fn Test_Edges_Should_Be_In_Canonical_Order()
{
    let workspace = TemporaryWorkspace::New();
    workspace.Write(Path::new("go.work"), "use (\n\t./a\n\t./b\n\t./c\n)\n");
    workspace.Write(
        Path::new("a/go.mod"),
        "module example.com/a\n\nrequire (\n\texample.com/c v0.0.0\n\texample.com/b v0.0.0\n)\n",
    );
    workspace.Write(Path::new("b/go.mod"), "module example.com/b\n");
    workspace.Write(Path::new("c/go.mod"), "module example.com/c\n");

    let discovered = Discover_Workspace(&workspace.root).expect("a real workspace");

    let a = Payload_Of(&discovered, "example.com/a");
    let mut sorted = a.edges.clone();
    sorted.sort_by(|left, right| left.target.cmp(&right.target));
    assert_eq!(a.edges, sorted);
}

#[test]
fn Test_A_Missing_Go_Mod_Should_Be_Refused()
{
    let workspace = TemporaryWorkspace::New();
    std::fs::create_dir_all(&workspace.root).expect("the empty root itself exists");

    let error = Discover_Workspace(&workspace.root).expect_err("no go.mod anywhere under root");
    assert!(error.reason.contains("go.mod"), "{}", error.reason);
}

/// (`go.mod` content, the fragment its own refusal reason must contain) for a manifest that
/// declares no `module` line at all — a second case beside this one would extend the table
/// rather than duplicate the test.
fn Go_Mod_With_No_Module_Line_Cases() -> Vec<(&'static str, &'static str)>
{
    return vec![("go 1.21\n", "module")];
}

#[test]
fn Test_A_Go_Mod_With_No_Module_Line_Should_Be_Refused()
{
    for (content, expected_fragment) in Go_Mod_With_No_Module_Line_Cases()
    {
        let workspace = TemporaryWorkspace::New();
        workspace.Write(Path::new("go.mod"), content);

        let error = Discover_Workspace(&workspace.root).expect_err("a go.mod with no module line");
        assert!(error.reason.contains(expected_fragment), "{}", error.reason);
    }
}

/// (`go.work` content, the fragment its own refusal reason must contain) for a workspace
/// file that declares no `use` directive at all.
fn Go_Work_With_No_Use_Directive_Cases() -> Vec<(&'static str, &'static str)>
{
    return vec![("go 1.21\n", "use")];
}

#[test]
fn Test_A_Go_Work_With_No_Use_Directive_Should_Be_Refused()
{
    for (content, expected_fragment) in Go_Work_With_No_Use_Directive_Cases()
    {
        let workspace = TemporaryWorkspace::New();
        workspace.Write(Path::new("go.work"), content);

        let error = Discover_Workspace(&workspace.root).expect_err("a go.work naming no member");
        assert!(error.reason.contains(expected_fragment), "{}", error.reason);
    }
}
