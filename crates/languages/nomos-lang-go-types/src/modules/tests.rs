//! Which module holds a source, over a filesystem that holds exactly the `go.mod` files a test names.

use super::*;
use nomos_platform::{DeterminismStrength, FileSystemError, ReproducibilityScope, Strategy, TraceEquivalence};
use std::collections::BTreeSet;
use std::path::PathBuf;

/// A filesystem holding exactly the `go.mod` files under [`Root`] a test names.
struct FakeFileSystem
{
    go_mods: BTreeSet<PathBuf>,
}

/// Answers from fixed data, so every read is the same.
impl Strategy for FakeFileSystem
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl FileSystem for FakeFileSystem
{
    fn Read_To_String(&self, path: &Path) -> Result<String, FileSystemError>
    {
        return Err(FileSystemError::NotFound { path: path.to_string_lossy().into_owned() });
    }

    fn Replace_Atomically(&self, _path: &Path, _contents: &str) -> Result<(), FileSystemError>
    {
        unimplemented!("module discovery never writes")
    }

    fn Exists(&self, path: &Path) -> bool
    {
        return self.go_mods.contains(path);
    }
}

fn Root() -> PathBuf
{
    return PathBuf::from("repo");
}

fn Holding(directories: &[&str]) -> FakeFileSystem
{
    return FakeFileSystem { go_mods: directories.iter().map(|directory| return Root().join(directory).join("go.mod")).collect() };
}

/// A root module holds everything beneath it except what a nested module holds, and a source above
/// every `go.mod` is held by none -- the nearest module wins, as `go` itself resolves it.
#[test]
fn Test_Each_Source_Should_Belong_To_Its_Nearest_Module_Or_To_None()
{
    let filesystem = Holding(&["", "tools/gen"]);
    let lone = Holding(&["svc"]);

    let modules = Group_By_Module(&Root(), &["main.go", "pkg/a/a.go", "tools/gen/gen.go", "tools/gen/sub/s.go"], &filesystem);
    let partly = Group_By_Module(&Root(), &["svc/main.go", "scripts/hack.go"], &lone);

    assert_eq!(modules.holding.get(""), Some(&vec!["main.go".to_owned(), "pkg/a/a.go".to_owned()]));
    assert_eq!(modules.holding.get("tools/gen"), Some(&vec!["tools/gen/gen.go".to_owned(), "tools/gen/sub/s.go".to_owned()]));
    assert!(modules.outside.is_empty(), "{modules:?}");
    assert_eq!(partly.outside, ["scripts/hack.go"]);
}
