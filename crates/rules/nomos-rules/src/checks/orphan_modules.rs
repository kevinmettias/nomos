//! A Rust source file no `mod` declaration reaches, ported from code-standards'
//! `check-orphan-modules` (`rules/language-specific/rust/checks/check-orphan-modules`,
//! `main.go` plus `module_reachability.go`).
//!
//! Rust compiles only what the module tree names. A `.rs` file nothing declares is never
//! parsed: it does not compile, its tests do not run, its lints do not fire, and no part of
//! the toolchain says a word. It reads as finished work — every editor opens it, every
//! reviewer reads it, every doc links it — and it does nothing. The cost is not a bug in the
//! file; it is the belief that the file is doing its job.
//!
//! Rust is unusual in needing this at all. Go compiles every file in a package directory and
//! C# compiles every file the project globs, so neither language can carry an orphan. That
//! is why this rule is scoped by path rather than by an [`crate::SourceFile::Is_Written_In`]
//! guard: a file outside a crate's source tree has no module tree to be reachable from, and
//! there is nothing for the rule to say about it.
//!
//! # Three shapes a naive version gets wrong
//!
//! Each is recorded in the Go implementation as a false positive it produced on real code,
//! and a rule that cries wolf on live files is a rule somebody turns off — which is how the
//! orphans get in.
//!
//!   - `mod r#match;` is backed by `match.rs`. The raw-identifier prefix is spelling, not
//!     part of the name, and missing it reports a live, compiling file as an orphan.
//!   - An inline module is backed by no file and declares nothing on disk, so the
//!     terminating semicolon is required rather than incidental.
//!   - `#[path = "…"]` names a file relative to *the directory the declaring file sits in*,
//!     and the file it names then owns *the directory it lands in* — its children are its
//!     siblings, not entries in a subdirectory named after it. Deriving that directory from
//!     the file's own name reported all thirty-nine files of one live crate as orphans.
//!
//! # Resolution is against the sources, not the filesystem
//!
//! The Go original asks the filesystem whether each candidate exists. This one asks the set
//! of files the run collected, which is what `D-134` and `OD-RULES-001` require of every
//! rule here: a rule that reads the filesystem can only ever be judged against the tree it
//! is standing in. The two answers move together — a file the walk did not collect can
//! neither back a declaration nor be reported as an orphan — so no file changes verdict for
//! having been resolved one way rather than the other.
//!
//! That assumption is load-bearing and was briefly false. `nomos gate run --include <file>`
//! used to filter the walked set before this rule saw it, which collects a file while
//! dropping the `lib.rs` that declares it — the one shape the sentence above rules out — and
//! the rule duly reported a declared, compiling file as an orphan, advising a reader to
//! delete or re-declare correct code. `OD-GATE-025` took the scope off the source set: a
//! gate run judges every walked file and narrows only what it reports, so the walk this rule
//! sees is whole again and the assumption holds by construction rather than by nobody having
//! narrowed yet.
//!
//! A crate is likewise identified by a `src` path segment rather than by a `Cargo.toml`
//! beside it, because a manifest is not a source this walk collects. Measured against this
//! workspace when the rule was written: sixty source directories found this way, sixty with
//! a sibling manifest, and none of either without the other — the two agree exactly, and the
//! divergence is in what the rule can see rather than in what it decides.
//!
//! # What this port narrows, deliberately
//!
//! The original reads every direct `.rs` child of `src/bin/` as an auto-discovered binary
//! root but not `src/bin/<name>/main.rs`, which Cargo also compiles as a root. That gap is
//! inherited rather than repaired: widening it here would make this port disagree with the
//! tool it is porting on a shape neither tree exercises, and the disagreement would be
//! invisible until one did.
//!
//! Declarations are read one line at a time. The original's `mod` pattern is line-anchored
//! and this is exactly equivalent for it; its path-attribute pattern is not, so an attribute
//! split across lines, or a second one on the same line, is missed here and found there.
//! Neither is a shape `rustfmt` produces.
//!
//! # Why no self-exemption
//!
//! Nine modules in this crate skip their own file, because their fixtures spell the very
//! shape they judge. This one needs no such guard and deliberately carries none —
//! `P45-CODE-PREFIX-KNOWS-STRINGS` is retiring that mechanism for keying on a path suffix
//! any repository can match. A declaration read out of this file's own text would resolve
//! against `crates/rules/nomos-rules/src/checks/orphan_modules/`, a directory that does not
//! exist, so nothing it could find is a file anything else reaches. The fixtures below still
//! avoid opening a line inside a string literal with the declaration keyword, so the
//! question stays academic.

use super::code_prefix::Code_Prefix;
use crate::SourceFile;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};
use std::collections::{BTreeMap, BTreeSet};

/// The code-standards module-reachability rule id.
pub const NO_ORPHAN_MODULES: &str = "no-orphan-modules";

/// The directory segment that marks a crate's source tree.
const SOURCE_DIRECTORY_NAME: &str = "src";

/// The directory under a crate's source tree whose every direct `.rs` child Cargo compiles
/// as its own crate root, with no declaration naming it.
const BINARY_DIRECTORY_NAME: &str = "bin";

/// The extension every file this rule judges carries.
const RUST_SOURCE_SUFFIX: &str = ".rs";

/// The pre-2018 directory-module file: `foo/mod.rs` backs `mod foo;` where `foo.rs` does not.
const DIRECTORY_MODULE_FILE: &str = "mod.rs";

/// The files Cargo compiles as a crate root without any declaration reaching them.
const CRATE_ROOT_FILES: [&str; 2] = ["lib.rs", "main.rs"];

/// The prefix that lets a keyword be spelled as an identifier. It is spelling, not name:
/// `mod r#match;` is backed by `match.rs`.
const RAW_IDENTIFIER_PREFIX: &str = "r#";

/// The declaration keyword this rule resolves, and the attribute that redirects it.
const MODULE_KEYWORD: &str = "mod";
const PATH_ATTRIBUTE_NAME: &str = "path";

/// Reports every Rust source under a crate's source tree that the crate's module tree does
/// not reach.
#[must_use]
pub fn Check_No_Orphan_Modules(sources: &[SourceFile]) -> Vec<Finding>
{
    let tree = SourceTree::Of(sources);
    let mut findings = Vec::new();

    for source_directory in tree.Source_Directories()
    {
        findings.extend(tree.Orphan_Findings_Under(source_directory));
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// One reached module: the file itself, and the directory *its own* children resolve
/// against.
///
/// The directory cannot be derived from the file's name, which is the subtlety that makes a
/// naive version of this rule wrong. A crate root and a `mod.rs` own the directory they sit
/// in; a plain `foo.rs` owns the sibling `foo/`; and a file a path attribute pulled in owns
/// the directory it landed in, so its children are its siblings.
struct Module
{
    file: String,
    directory: String,
}

/// Every Rust source the run collected, addressed by the path a declaration would resolve to.
struct SourceTree<'a>
{
    by_path: BTreeMap<&'a str, &'a SourceFile>,
}

impl<'a> SourceTree<'a>
{
    /// Indexes the Rust sources among `sources`, ignoring every other language.
    fn Of(sources: &'a [SourceFile]) -> Self
    {
        let mut by_path = BTreeMap::new();

        for source in sources.iter().filter(|source| return source.path.ends_with(RUST_SOURCE_SUFFIX))
        {
            by_path.insert(source.path.as_str(), source);
        }

        return Self { by_path };
    }

    /// Every crate source directory these sources sit under, each named once.
    fn Source_Directories(&self) -> BTreeSet<&'a str>
    {
        return self.by_path.keys().filter_map(|path| return Source_Directory_Of(path)).collect();
    }

    /// The findings for one crate source directory: every Rust source beneath it that the
    /// walk from its roots did not reach.
    fn Orphan_Findings_Under(&self, source_directory: &str) -> Vec<Finding>
    {
        let reached = self.Reachable_Under(source_directory);
        let mut findings = Vec::new();

        for (path, source) in &self.by_path
        {
            if Source_Directory_Of(path) == Some(source_directory) && !reached.contains(*path)
            {
                findings.push(Orphan_Finding(source));
            }
        }

        return findings;
    }

    /// The set of files the module tree under `source_directory` actually reaches, walked
    /// out from every root Cargo compiles without a declaration.
    fn Reachable_Under(&self, source_directory: &str) -> BTreeSet<String>
    {
        let mut reached = BTreeSet::new();
        let mut pending = self.Roots_Under(source_directory);

        while let Some(current) = pending.pop()
        {
            if !reached.insert(current.file.clone())
            {
                continue;
            }

            pending.extend(self.Children_Of(&current));
        }

        return reached;
    }

    /// The crate roots under `source_directory`: `lib.rs`, `main.rs`, and every direct `.rs`
    /// child of `bin/`. A crate root owns the directory it sits in.
    fn Roots_Under(&self, source_directory: &str) -> Vec<Module>
    {
        let mut roots = Vec::new();

        for name in CRATE_ROOT_FILES
        {
            let candidate = format!("{source_directory}/{name}");

            if self.by_path.contains_key(candidate.as_str())
            {
                roots.push(Module { file: candidate, directory: source_directory.to_owned() });
            }
        }

        let binary_directory = format!("{source_directory}/{BINARY_DIRECTORY_NAME}");

        for path in self.by_path.keys()
        {
            if Is_Direct_Child_Of(path, &binary_directory)
            {
                roots.push(Module { file: (*path).to_owned(), directory: binary_directory.clone() });
            }
        }

        return roots;
    }

    /// The modules `parent` declares, each paired with the directory *its* children resolve
    /// against. A declaration backed by no collected file is skipped: that is a compile
    /// error, and rustc reports it far better than this would.
    fn Children_Of(&self, parent: &Module) -> Vec<Module>
    {
        let Some(source) = self.by_path.get(parent.file.as_str())
        else
        {
            return Vec::new();
        };

        let declaring_directory = Parent_Directory_Of(&parent.file);
        let mut children = Vec::new();

        for line in source.text.lines()
        {
            let code = Code_Prefix(line);

            if let Some(relative) = Declared_Path_Attribute(&code)
                && let Some(child) = self.Resolve_Path_Attribute(declaring_directory, relative)
            {
                children.push(child);
            }

            if let Some(name) = Declared_Module_Name(&code)
                && let Some(child) = self.Resolve_Declaration(&parent.directory, name)
            {
                children.push(child);
            }
        }

        return children;
    }

    /// Resolves a path attribute against the directory the declaring file sits in. The file
    /// it names owns the directory it lands in, so its own children are its siblings.
    fn Resolve_Path_Attribute(&self, declaring_directory: &str, relative: &str) -> Option<Module>
    {
        let file = Joined_Path(declaring_directory, relative)?;

        if !self.by_path.contains_key(file.as_str())
        {
            return None;
        }

        let directory = Parent_Directory_Of(&file).to_owned();
        return Some(Module { file, directory });
    }

    /// Resolves `mod name;` against `directory`, trying both layouts Rust allows: the
    /// sibling file `name.rs` first, then the directory module `name/mod.rs`. Either way the
    /// module it finds owns `directory/name`.
    fn Resolve_Declaration(&self, directory: &str, name: &str) -> Option<Module>
    {
        let owned = format!("{directory}/{name}");
        let sibling = format!("{owned}{RUST_SOURCE_SUFFIX}");

        if self.by_path.contains_key(sibling.as_str())
        {
            return Some(Module { file: sibling, directory: owned });
        }

        let nested = format!("{owned}/{DIRECTORY_MODULE_FILE}");

        if self.by_path.contains_key(nested.as_str())
        {
            return Some(Module { file: nested, directory: owned });
        }

        return None;
    }
}

/// The one finding an unreachable source produces.
fn Orphan_Finding(source: &SourceFile) -> Finding
{
    return Finding {
        rule: RuleId::New(NO_ORPHAN_MODULES),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!(
            "{} is not reachable from its crate's module tree: no declaration names it, so rustc never parses it and \
             nothing in it compiles, runs or is linted -- declare it, or delete it",
            source.path
        ),
        locations: vec![source.path.clone()],
    };
}

/// The crate source directory `path` sits under: everything up to and including its first
/// `src` segment, or `None` for a file outside any crate's source tree.
fn Source_Directory_Of(path: &str) -> Option<&str>
{
    let mut offset = 0usize;

    for segment in path.split('/')
    {
        let end = offset.saturating_add(segment.len());

        if segment == SOURCE_DIRECTORY_NAME && end < path.len()
        {
            return path.get(..end);
        }

        offset = end.saturating_add(1);
    }

    return None;
}

/// Whether `path` names a `.rs` file sitting directly in `directory`, with no further
/// directory between the two.
fn Is_Direct_Child_Of(path: &str, directory: &str) -> bool
{
    let Some(remainder) = path.strip_prefix(directory).and_then(|rest| return rest.strip_prefix('/'))
    else
    {
        return false;
    };

    return remainder.ends_with(RUST_SOURCE_SUFFIX) && !remainder.contains('/');
}

/// The directory `path` sits in, empty when it sits at the root.
fn Parent_Directory_Of(path: &str) -> &str
{
    return match path.rfind('/')
    {
        Some(separator) => path.get(..separator).unwrap_or(""),
        None => "",
    };
}

/// `relative` resolved against `directory`, with `.` dropped and `..` applied, or `None`
/// when it climbs above the root it started from.
fn Joined_Path(directory: &str, relative: &str) -> Option<String>
{
    let mut segments: Vec<String> = Vec::new();

    if !directory.is_empty()
    {
        segments.extend(directory.split('/').map(str::to_owned));
    }

    let normalized = relative.replace('\\', "/");
    let named_segments = normalized.split('/').filter(|segment| return !segment.is_empty() && *segment != ".");

    for segment in named_segments
    {
        match segment
        {
            ".." =>
            {
                segments.pop()?;
            },
            named => segments.push(named.to_owned()),
        }
    }

    return Some(segments.join("/"));
}

/// The code before any line comment, so a commented-out declaration does not read as a live
/// one — which would hide the very orphan this rule looks for. This crate's established
/// per-file convention, which `P45-CODE-PREFIX-KNOWS-STRINGS` will replace with one shared
/// helper.
/// The module name a line declares with `mod name;`, or `None` for any other line.
///
/// The terminating semicolon is required, which is what excludes an inline module: one
/// written with a body is backed by no file and so declares nothing on disk.
fn Declared_Module_Name(code: &str) -> Option<&str>
{
    let after_visibility = Without_Visibility(code.trim_start()).trim_start();
    let after_keyword = Without_Module_Keyword(after_visibility)?;
    let after_prefix = after_keyword.strip_prefix(RAW_IDENTIFIER_PREFIX).unwrap_or(after_keyword);
    let (name, remainder) = Leading_Identifier(after_prefix)?;

    if !remainder.trim_start().starts_with(';')
    {
        return None;
    }

    return Some(name);
}

/// `code` with a leading `pub` or `pub(…)` removed, unchanged when it carries neither.
fn Without_Visibility(code: &str) -> &str
{
    let Some(after_visibility) = code.strip_prefix("pub")
    else
    {
        return code;
    };

    if let Some(after_open) = after_visibility.trim_start().strip_prefix('(')
        && let Some(close) = after_open.find(')')
    {
        return after_open.get(close.saturating_add(1)..).unwrap_or("");
    }

    if !after_visibility.starts_with(char::is_whitespace)
    {
        return code;
    }

    return after_visibility;
}

/// `code` past a leading declaration keyword and the whitespace after it, or `None` when it
/// does not start with the keyword — an identifier merely beginning with those letters must
/// not read as one.
fn Without_Module_Keyword(code: &str) -> Option<&str>
{
    let after_keyword = code.strip_prefix(MODULE_KEYWORD)?;

    if !after_keyword.starts_with(char::is_whitespace)
    {
        return None;
    }

    return Some(after_keyword.trim_start());
}

/// The identifier `code` starts with, paired with everything after it.
fn Leading_Identifier(code: &str) -> Option<(&str, &str)>
{
    let first = code.chars().next()?;

    if !first.is_ascii_alphabetic() && first != '_'
    {
        return None;
    }

    let end = code
        .find(|character: char| return !character.is_ascii_alphanumeric() && character != '_')
        .unwrap_or(code.len());

    return Some((code.get(..end)?, code.get(end..)?));
}

/// The file a path attribute on this line names, or `None` when the line carries no complete
/// attribute.
fn Declared_Path_Attribute(code: &str) -> Option<&str>
{
    let opened = code.find("#[").and_then(|start| return code.get(start.saturating_add(2)..))?;
    let after_name = opened.trim_start().strip_prefix(PATH_ATTRIBUTE_NAME)?;
    let after_equals = after_name.trim_start().strip_prefix('=')?;
    let quoted = after_equals.trim_start().strip_prefix('"')?;
    let end = quoted.find('"')?;
    let after_quote = quoted.get(end.saturating_add(1)..)?;

    if !after_quote.trim_start().starts_with(']')
    {
        return None;
    }

    return quoted.get(..end);
}

#[cfg(test)]
mod tests
{
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

        assert_eq!(findings.len(), 2, "{findings:?}");
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
        assert_eq!(Joined_Path("demo", "../../escaped.rs"), None);
        assert_eq!(Joined_Path("demo/src", "../shared/held.rs"), Some("demo/shared/held.rs".to_owned()));
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
}
