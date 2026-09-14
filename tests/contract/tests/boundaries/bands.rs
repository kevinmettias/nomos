//! This repository's own architecture declaration, and the walk two of the assertions need.
//!
//! The declaration is not written here and never has been. `OD-RULES-020`'s migration made
//! `nomos-rules` the one place it lived; `OD-RULES-003`'s third prerequisite moved it out of
//! any crate entirely, into `nomos-architecture.json` at the repository root, and these
//! assertions read it through the one provider that reads it — never by parsing the file a
//! second way. A second copy is exactly what let the README and the dependency graph drift
//! against each other before, and a second *parser* would be the same defect wearing the
//! shape of independence. What these tests are independent of is `cargo metadata`, which is
//! the half the declaration cannot state about itself.

use nomos_cap_architecture::ArchitecturePayload;
use nomos_platform_std::StdFileSystem;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// This repository's own declared architecture, read from the root.
///
/// Panics rather than returning an error: every assertion below is about a declaration this
/// repository is asserted elsewhere to have, so a declaration that cannot be read is a broken
/// checkout and not a finding.
pub(crate) fn Declared_Architecture() -> ArchitecturePayload
{
    return nomos_repo_policy::architecture::Discover_Workspace(&Repository_Root(), &StdFileSystem)
        .expect("this repository's own nomos-architecture.json is committed and readable");
}

/// The workspace root, from this crate's manifest directory.
pub(crate) fn Repository_Root() -> PathBuf
{
    return Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
}

/// Every `.rs` file under a directory, recursively.
pub(crate) fn Source_Files(root: &Path) -> Vec<PathBuf>
{
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop()
    {
        let Ok(entries) = std::fs::read_dir(&directory)
        else
        {
            continue;
        };
        for entry in entries.flatten()
        {
            Sort_One_Entry(&entry.path(), &mut pending, &mut found);
        }
    }

    return found;
}

/// A directory to descend into later, a Rust file to keep, or neither.
fn Sort_One_Entry(path: &Path, pending: &mut Vec<PathBuf>, found: &mut Vec<PathBuf>)
{
    if path.is_dir()
    {
        pending.push(path.to_path_buf());
    }
    else if path.extension().is_some_and(|extension| extension == "rs")
    {
        found.push(path.to_path_buf());
    }
}

/// Every `mod` name declared anywhere under a source root.
///
/// Deliberately a flat set rather than a resolved tree. The precise version would walk
/// declarations from each root, and would need to handle `#[cfg]` and inline modules to
/// avoid false positives — `#[path]` is handled below. This approximation cannot report a
/// false orphan — it only misses the case where a module is declared in one place and the
/// file lives in an unrelated one, which is a naming problem rather than an invisibility
/// problem.
pub(crate) fn Declared_Modules(source_root: &Path) -> BTreeSet<String>
{
    let mut declared = BTreeSet::new();
    for file in Source_Files(source_root)
    {
        let Ok(text) = std::fs::read_to_string(&file)
        else
        {
            continue;
        };
        for line in text.lines()
        {
            declared.extend(Module_Declared_By(line));
            declared.extend(Path_Attribute_Stem(line));
        }
    }

    return declared;
}

/// The file stem a `#[path = "..."]` attribute names, when this line carries one.
///
/// `#[path]` is legal only on a `mod` item, so finding the attribute is enough on its own
/// to know the file it names is reachable — rustc has already enforced which item it
/// attaches to, whether the two share a line (`#[path = "x.rs"] mod y;`) or not.
fn Path_Attribute_Stem(line: &str) -> Option<String>
{
    let trimmed = line.trim();
    if trimmed.starts_with("//")
    {
        return None;
    }

    let (_, after_marker) = trimmed.split_once("#[path")?;
    let (_, after_open_quote) = after_marker.split_once('"')?;
    let (path, _) = after_open_quote.split_once('"')?;

    return Path::new(path).file_stem()?.to_str().map(str::to_owned);
}

/// The file name one `mod` line declares.
///
/// A commented-out declaration is not a declaration. The sibling workspace names this
/// specifically as a decoy that made an orphan look declared. And `mod foo;` is a declaration
/// of a file where `mod foo {` is an inline module, which declares nothing on disk.
fn Module_Declared_By(line: &str) -> Option<String>
{
    let trimmed = line.trim();
    if trimmed.starts_with("//")
    {
        return None;
    }

    let rest = Without_Visibility(trimmed).strip_prefix("mod ")?;
    let name = rest.strip_suffix(';')?.trim();

    return Some(name.strip_prefix("r#").unwrap_or(name).to_owned());
}

/// A line with any leading visibility modifier removed, whatever spelling it used.
///
/// `pub`, `pub(crate)`, `pub(super)` and `pub(in some::path)` are all legal ahead of `mod`,
/// and a fixed list of exact prefixes missed `pub(super)` the first time this was written —
/// stripping the shape generically (an optional `pub`, then an optional parenthesized
/// group) covers every spelling rustc accepts instead of enumerating them one at a time.
fn Without_Visibility(trimmed: &str) -> &str
{
    let Some(after_pub) = trimmed.strip_prefix("pub") else { return trimmed };
    let after_pub = after_pub.trim_start();
    if let Some(after_open_paren) = after_pub.strip_prefix('(')
        && let Some((_, after_close_paren)) = after_open_paren.split_once(')')
    {
        return after_close_paren.trim_start();
    }

    return after_pub;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Path_Attribute_On_Its_Own_Line_Should_Declare_Its_File_Stem()
    {
        assert_eq!(Path_Attribute_Stem(r#"#[path = "build_variant.rs"]"#), Some("build_variant".to_owned()));
    }

    #[test]
    fn Test_A_Path_Attribute_Sharing_A_Line_With_Its_Mod_Should_Still_Declare_Its_File_Stem()
    {
        assert_eq!(
            Path_Attribute_Stem(r#"#[path = "fact_reuse.rs"] mod determinism;"#),
            Some("fact_reuse".to_owned())
        );
    }

    #[test]
    fn Test_A_Commented_Out_Path_Attribute_Should_Declare_Nothing()
    {
        assert_eq!(Path_Attribute_Stem(r#"// #[path = "build_variant.rs"]"#), None);
    }

    #[test]
    fn Test_A_Line_With_No_Path_Attribute_Should_Declare_Nothing()
    {
        assert_eq!(Path_Attribute_Stem("mod variant;"), None);
    }

    #[test]
    fn Test_A_Raw_Identifier_Module_Should_Declare_Its_Unprefixed_Name()
    {
        assert_eq!(Module_Declared_By("mod r#type;"), Some("type".to_owned()));
        assert_eq!(Module_Declared_By("pub(crate) mod r#ref;"), Some("ref".to_owned()));
    }

    /// Every visibility spelling a module declaration can carry, all naming the same module.
    const VISIBILITY_SPELLINGS: [&str; 5] = [
        "mod finish;",
        "pub mod finish;",
        "pub(crate) mod finish;",
        "pub(super) mod finish;",
        "pub(in crate::store) mod finish;",
    ];

    #[test]
    fn Test_Every_Visibility_Spelling_Should_Declare_Its_Module()
    {
        for line in VISIBILITY_SPELLINGS
        {
            assert_eq!(Module_Declared_By(line), Some("finish".to_owned()), "{line} should declare finish");
        }
    }
}
