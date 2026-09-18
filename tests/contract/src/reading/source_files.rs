//! Which files are the crate proper, and which text in them is not.
//!
//! Two questions with one answer between them: a unit test is not the thing it tests, and a
//! scan that reads one as though it were reports examples as declarations. A `#[cfg(test)]`
//! module is blanked where it is inline and its file is left out where it is declared, so
//! that both spellings of "this is a test" are invisible to every caller here.

use crate::reading::masks::{Is_Code, Matching_Brace, Scan};
use std::path::{Path, PathBuf};

/// A file's text with every `#[cfg(test)]` item body blanked out.
///
/// `pub(crate)` for `strategies.rs`, which asks whether a crate declares a determinism
/// strategy and must not be answered by one written inside a unit-test module.
/// `nomos-contracts` declares two — `AnalysisKernel` and `AgentHost` — as compile-time
/// examples of the trait it defines, and counting those as domains would have credited the
/// contracts crate with occupying rows of a table it exists to describe.
///
/// The bodies are blanked rather than removed so that byte offsets are unchanged and a
/// caller may still line up the result with the original.
///
/// Brace matching runs over the same masks the rest of this module uses, because a
/// `panic!("{} ...")` inside a test module would otherwise desynchronise the scan and
/// blank the remainder of the file — which would hide real declarations and report a clean
/// result, the failure direction that flatters.
///
/// The search for a body is bounded by the end of the item the attribute is attached to,
/// which is the whole of `Item_Shape_After`'s job. An earlier version asked only for the
/// next open brace anywhere in the file, and for `#[cfg(test)] mod registration;` — the
/// ordinary way to declare a test-only module living in another file — that brace belonged
/// to some later, unrelated item, so everything between them was blanked including that
/// item. `nomos-spec-store` lost its `pub use authoring::{…}` that way and the public
/// surface snapshot went red naming the crate, not the scanner.
pub(crate) fn Without_Test_Modules(text: &str) -> String
{
    let bytes = text.as_bytes();
    let masks = Scan(text);
    // A byte buffer rather than an in-place edit of the `String`: this crate forbids
    // unsafe, so there is no mutable view of a `String`'s bytes to reach for, and blanking
    // to spaces keeps every offset and every line break where it was.
    let mut blanked = bytes.to_vec();
    let mut index = 0_usize;
    while index < bytes.len()
    {
        let Some((from, to)) = Marked_Range(bytes, &masks.code, index)
        else
        {
            index = index.saturating_add(1);
            continue;
        };

        Blank(&mut blanked, from, to);
        index = to.saturating_add(1);
    }

    // Blanking replaces whole bytes of what was valid UTF-8 with ASCII spaces, so the
    // result is still valid UTF-8 — but a multi-byte character partially overwritten would
    // not be, and the lossy conversion is what keeps a scanner bug from becoming a panic in
    // a check that is supposed to report.
    return String::from_utf8_lossy(&blanked).into_owned();
}

/// The byte range one `#[cfg(test)]` item occupies, if the marker begins at `index`.
///
/// A body is blanked from its opening brace, leaving the attribute and the item's signature
/// legible, which is what every caller has always seen. A declaration has no body to blank,
/// so what goes is the declaration itself — from the attribute through its `;` — and nothing
/// beyond it.
///
/// `None` for an offset the marker does not begin at, and for an item this scan cannot bound,
/// which means a truncated source.
fn Marked_Range(bytes: &[u8], mask: &[bool], index: usize) -> Option<(usize, usize)>
{
    if !Is_Code(mask, index) || !Starts_Marker(bytes, index)
    {
        return None;
    }

    let after = index.saturating_add(MARKER.len());
    match Item_Shape_After(bytes, mask, after)?
    {
        ItemShape::Body(open) =>
        {
            let close = Matching_Brace(bytes, mask, open)?;

            return Some((open, close));
        }
        ItemShape::Declaration(end) => return Some((index, end)),
    }
}

/// Overwrites a byte range with spaces, leaving line breaks where they were.
///
/// Keeping the newlines is what lets a caller line a blanked buffer up with the original by
/// line as well as by offset.
fn Blank(bytes: &mut [u8], from: usize, to: usize)
{
    for offset in from..=to
    {
        let Some(slot) = bytes.get_mut(offset)
        else
        {
            continue;
        };

        if *slot != b'\n'
        {
            *slot = b' ';
        }
    }
}

/// The attribute that marks an item as test-only.
///
/// A `&str` rather than a byte literal because both readers of it want that form and the one
/// that wants bytes can ask for them: the byte scan compares a window against
/// `MARKER.as_bytes()` and the line scan compares a trimmed line against `MARKER` itself. One
/// spelling is what keeps the two from drifting the way a second literal would.
const MARKER: &str = "#[cfg(test)]";

/// Whether a `#[cfg(test)]` attribute begins at an offset.
fn Starts_Marker(bytes: &[u8], index: usize) -> bool
{
    return bytes
        .get(index..index.saturating_add(MARKER.len()))
        .is_some_and(|window| return window == MARKER.as_bytes());
}

/// How the item carrying a `#[cfg(test)]` attribute ends.
///
/// The distinction the scanner needs is not which keyword follows the attribute but
/// whether the item terminates with a body or with a `;`, because that is the only thing
/// that decides how much text belongs to it. `mod x { … }`, `fn f() { … }` and
/// `impl T for U { … }` are one shape; `mod x;`, `use a::b;` and `struct S;` are the
/// other. Reading the keyword instead would mean listing every item form Rust has and
/// re-listing it whenever one is added — and `mod` alone appears in both shapes anyway.
enum ItemShape
{
    /// The item has a body. The offset is its opening brace.
    Body(usize),
    /// The item is a declaration and has no body. The offset is the `;` that ends it.
    Declaration(usize),
}

/// Whichever of `{` or `;` ends the item beginning at `from`, and which one it was.
///
/// This is the bound that `Without_Test_Modules` was missing. Whichever of the two comes
/// first decides the shape, so a declaration can never borrow the body of the item after
/// it: the `;` is reached before that item's brace is.
///
/// Nesting depth counts only `(`/`)` and `[`/`]`, and both delimiters matter. A `;` can sit
/// inside either without ending anything — `fn f(v: [u8; 4]) { … }` has one in an array
/// type, ahead of the brace that really is the body — and a further `#[…]` attribute
/// stacked under the marker opens and closes a bracket of its own before the item even
/// begins. Angle brackets are deliberately not counted: `<` is ambiguous with comparison
/// in Rust's grammar, and it need not be, because generics and where-clauses put no bare
/// `;` or `{` between the attribute and the body. Anything they could carry that looks
/// like one — an array length, a const-generic argument — is already inside `[]` or `()`.
///
/// Only code bytes are read, so a `;` in a string and a `{` in a comment are both invisible
/// to it. `None` means the file ends mid-item, which is a truncated source rather than a
/// declaration, and the caller stops rather than guessing.
fn Item_Shape_After(bytes: &[u8], mask: &[bool], from: usize) -> Option<ItemShape>
{
    let mut cursor = from;
    let mut depth = 0_u32;

    while cursor < bytes.len()
    {
        if Is_Code(mask, cursor)
        {
            match bytes.get(cursor).copied()
            {
                Some(b'(' | b'[') => depth = depth.saturating_add(1),
                Some(b')' | b']') => depth = depth.saturating_sub(1),
                Some(b'{') if depth == 0 => return Some(ItemShape::Body(cursor)),
                Some(b';') if depth == 0 => return Some(ItemShape::Declaration(cursor)),
                _ =>
                {}
            }
        }
        cursor = cursor.saturating_add(1);
    }

    return None;
}

/// Every `.rs` file under a directory that is compiled into the crate proper.
///
/// `pub(crate)` because `strategies.rs` walks the same trees for a different property,
/// and two walkers would eventually disagree about what counts as a source file.
///
/// A file declared `#[cfg(test)] mod tests;` is left out, for exactly the reason
/// [`Without_Test_Modules`] blanks an inline `#[cfg(test)] mod tests { … }`: it is a unit
/// test, and a scanner that reads it is reading examples as though they were the thing
/// they are examples of. Without this, moving a test module out of the file it tests
/// makes the crate look like it constructs facts and declares no strategy — which is what
/// happened to `nomos-rules` the first time its `mirror` module became a directory.
pub(crate) fn Source_Files(root: &Path) -> Vec<PathBuf>
{
    let mut found = Rust_Files_Under(root);
    let test_only = Test_Only_Modules(&found);

    found.retain(|path| return !test_only.iter().any(|excluded| return path.starts_with(excluded)));

    return found;
}

/// Every `.rs` file under a directory, in whatever order the filesystem answers.
fn Rust_Files_Under(root: &Path) -> Vec<PathBuf>
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
            let path = entry.path();
            if path.is_dir()
            {
                pending.push(path);
            }
            else if path.extension().is_some_and(|extension| return extension == "rs")
            {
                found.push(path);
            }
        }
    }

    return found;
}

/// Every path a `#[cfg(test)] mod <name>;` declaration points at.
///
/// Both spellings, because Rust accepts either: `<name>.rs` beside the declaring file, or
/// `<name>/` beneath it. The declaration is read rather than the file name, so a module
/// that happens to be called `tests` and is compiled into the crate proper is still read.
fn Test_Only_Modules(files: &[PathBuf]) -> Vec<PathBuf>
{
    let mut excluded = Vec::new();
    for file in files
    {
        excluded.extend(Test_Only_In(file));
    }

    return excluded;
}

/// The paths the `#[cfg(test)] mod <name>;` declarations in one file point at.
///
/// The run of attribute lines above a declaration is read rather than only the line
/// immediately above it, because those two are not the same thing the moment a test module
/// says where it lives. Three files in this workspace spell it
/// `#[cfg(test)]`, `#[path = "…"]`, `mod tests;` — a `#[path]` between the marker and the
/// declaration — and a rule that asked about the immediately preceding line alone saw the
/// `#[path]` there and read the declaration as the crate proper. That is the failure
/// direction that flatters: the module is counted as production, and whatever strategy or
/// fact it declares is reported as the crate's own rather than as an example of it.
///
/// One spelling is deliberately outside this: `#[cfg(test)] #[path = "…"] mod tests;` written
/// as a single line, where the attribute and the item share it. Nothing in this workspace
/// uses it — a census of all fifteen `#[cfg(test)]`-plus-attribute declaration sites found the
/// marker alone on its own line in every one — and the path a scan reads is not a pattern it
/// should guess at.
fn Test_Only_In(file: &Path) -> Vec<PathBuf>
{
    let mut excluded = Vec::new();
    let Ok(text) = std::fs::read_to_string(file)
    else
    {
        return excluded;
    };
    let mut above: Vec<&str> = Vec::new();
    for line in text.lines()
    {
        let line = line.trim();
        if let Some(name) = Declared_Module(line).filter(|_| return Marks_Test_Only(&above))
        {
            excluded.extend(Homes_Of(file, name, Path_Attribute(&above)));
        }
        Extend_Attribute_Run(&mut above, line);
    }

    return excluded;
}

/// Whether a run of attribute lines marks the item below it as test-only.
///
/// Anywhere in the run and not merely at its start, because the marker and the item may have
/// attributes between them and the order among them is not part of what is being asked.
fn Marks_Test_Only(above: &[&str]) -> bool
{
    return above.iter().any(|line| return *line == MARKER);
}

/// Extends the run of attribute lines above a declaration, or clears it.
///
/// A blank line, a comment and a bare attribute all keep the run alive; a line of anything
/// else ends it, so one item's own signature is never read as the next item's attributes.
/// Blank lines and comments belong in the run because Rust attaches the attribute to the item
/// regardless of either — the gap is not a separator in the grammar, and treating it as one
/// here would read a real declaration as production.
fn Extend_Attribute_Run<'a>(above: &mut Vec<&'a str>, line: &'a str)
{
    if line.is_empty() || line.starts_with("//") || Is_An_Attribute_Alone(line)
    {
        above.push(line);
        return;
    }

    above.clear();
}

/// Whether a line is one `#[…]` attribute and nothing else.
///
/// The closing `]` is required as well as the opening `#[`, and requiring it is the whole
/// point: the one-line spelling `#[path = "x.rs"] mod tests;` also begins with `#[`, and
/// reading it as a bare attribute would attach its path to whichever declaration came next,
/// excluding a module that attribute never named. Over-exclusion is the direction that
/// flatters, so the guard is written to miss a spelling rather than to invent one.
fn Is_An_Attribute_Alone(line: &str) -> bool
{
    return line.starts_with("#[") && line.ends_with(']');
}

/// The `#[path = "…"]` a run of attribute lines carries, if it carries one.
fn Path_Attribute<'a>(above: &[&'a str]) -> Option<&'a str>
{
    return above.iter().find_map(|line| return Path_Of(line));
}

/// The path an `#[path = "…"]` attribute names, if the line is one.
fn Path_Of(line: &str) -> Option<&str>
{
    let value = line.strip_prefix("#[path = ")?;

    return value.strip_suffix(']')?.trim().strip_prefix('"')?.strip_suffix('"');
}

/// Every path a `mod <name>;` in `file` may resolve to, in the shape the declaration gives it.
///
/// A `#[path]` attribute replaces the resolution rather than adding to it: Rust reads the file
/// it names relative to the declaring file's own directory, and neither of `Module_Homes`'s
/// two spellings is consulted. The distinction is not academic — every `#[path]` in this
/// workspace reaches a path no derived home arrives at. `#[path = "store/tests/file_ledger.rs"]`
/// in `file_ledger.rs` is resolved from `store/tests/`, while the homes derived from that
/// file's own name are `src/tests.rs` and `src/tests/`, so a scan that kept asking the name
/// would exclude nothing and read the test module as production.
///
/// Without an attribute both spellings are returned, unchanged from what they were: `<name>.rs`
/// beside the declaring file and `<name>/` beneath it. The second is a directory, and an
/// exclusion naming a directory is compared by prefix, which is what makes it cover the
/// `mod.rs` inside it.
fn Homes_Of(file: &Path, name: &str, attribute: Option<&str>) -> Vec<PathBuf>
{
    if let Some(path) = attribute
    {
        return file.parent().map(|home| return home.join(path)).into_iter().collect();
    }

    return Module_Homes(file)
        .flat_map(|home| return [home.join(format!("{name}.rs")), home.join(name)])
        .collect();
}

/// The directories a `mod <name>;` in `file` may resolve against.
///
/// Two of them, because Rust 2018 gives a module two spellings and this scan has to know
/// both. A crate root, a `main.rs` and a `mod.rs` own the directory they sit in, so their
/// children are beside them. Any other file owns the directory *named after it*, so
/// `mirror.rs` declaring `mod tests;` means `mirror/tests.rs` and not `tests.rs`.
///
/// Reading only the first spelling is what this scan used to do, and it was invisible while
/// every module root in the workspace was a `mod.rs`. The moment they became `<name>.rs` it
/// stopped excluding a single test file, and `nomos-rules` was reported as constructing
/// facts and promising nothing about them — a claim assembled entirely out of its own unit
/// tests.
fn Module_Homes(file: &Path) -> impl Iterator<Item = PathBuf>
{
    let beside = file.parent().map(Path::to_path_buf);
    let stem = file.file_stem().and_then(std::ffi::OsStr::to_str).unwrap_or_default();
    let owns_its_directory = matches!(stem, "lib" | "main" | "mod");
    let beneath = beside
        .clone()
        .filter(|_| return !owns_its_directory)
        .map(|home| return home.join(stem));

    return beside.into_iter().chain(beneath);
}

/// The module name a `mod <name>;` line declares, if the line is one.
fn Declared_Module(line: &str) -> Option<&str>
{
    let name = line.strip_prefix("mod ")?.strip_suffix(';')?.trim();

    return name
        .chars()
        .all(|character| return character.is_ascii_alphanumeric() || character == '_')
        .then_some(name);
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// A test module with a body still loses the whole body, which is the behaviour every
    /// caller of `Without_Test_Modules` has always depended on.
    #[test]
    fn Test_A_Braced_Test_Item_Should_Still_Lose_Its_Whole_Body()
    {
        let source = r"
#[cfg(test)]
mod tests
{
    pub fn Helper_Inside_Tests() {}
}

pub use authoring::{Claimed};
";

        let blanked = Without_Test_Modules(source);
        assert!(
            !blanked.contains("Helper_Inside_Tests"),
            "the body of a braced #[cfg(test)] item survived: {blanked}"
        );
        assert!(
            blanked.contains("pub use authoring::{Claimed};"),
            "blanking a braced test module reached past it: {blanked}"
        );
        assert_eq!(blanked.len(), source.len());
    }

    /// The exports the `pub use authoring::{…}` line names, in `Test_A_Braceless_Test_Declaration_Should_Not_Eat_The_Item_After_It`'s fixture.
    /// Each must survive the brace-less `#[cfg(test)] mod registration;` above it.
    const AUTHORING_EXPORTS: [&str; 3] = ["BlockChange", "ClaimedRecord", "CommitReport"];

    /// The defect, in the shape it was found in. `#[cfg(test)] mod registration;` carries no
    /// braces of its own, so an unbounded search for a body takes the braces of whatever
    /// item comes next — in `nomos-spec-store` that was the `pub use authoring::{…}` line,
    /// and the public surface snapshot reported eleven exports missing without ever
    /// mentioning the scanner.
    #[test]
    fn Test_A_Braceless_Test_Declaration_Should_Not_Eat_The_Item_After_It()
    {
        let source = r"
mod record;
#[cfg(test)]
mod registration;
mod rows;

pub use authoring::{
    BlockChange, ClaimedRecord, CommitReport,
};
";

        let blanked = Without_Test_Modules(source);
        assert!(
            blanked.contains("pub use authoring::{"),
            "pub use authoring::{{…}} was eaten by the brace-less #[cfg(test)] mod \
             registration; above it; what survived was: {blanked}"
        );
        for export in AUTHORING_EXPORTS
        {
            assert!(
                blanked.contains(export),
                "the export {export} was eaten by the brace-less #[cfg(test)] mod \
                 registration; above it; what survived was: {blanked}"
            );
        }
        assert!(
            blanked.contains("mod rows;"),
            "mod rows; was eaten by the brace-less #[cfg(test)] mod registration; above it; \
             what survived was: {blanked}"
        );
    }

    /// A brace-less declaration blanks itself, so nothing downstream can read the module
    /// name back out and count it as a declared file.
    #[test]
    fn Test_A_Braceless_Test_Declaration_Should_Blank_Itself()
    {
        let source = r"
mod record;
#[cfg(test)]
mod registration;
mod rows;
";

        let blanked = Without_Test_Modules(source);
        assert!(
            !blanked.contains("registration"),
            "a brace-less #[cfg(test)] declaration was left in place: {blanked}"
        );
        assert!(!blanked.contains("#[cfg(test)]"), "the marker was left in place: {blanked}");
        assert!(blanked.contains("mod record;"), "the item above it was blanked: {blanked}");
        assert_eq!(blanked.len(), source.len());
    }

    /// Every brace-less item form the shape check has to treat alike: decided by `;` versus
    /// `{`, never by the keyword.
    const BRACELESS_ITEM_FORMS: [&str; 5] = [
        "mod registration;",
        "use super::Helper;",
        "struct Marker;",
        "type Alias = Vec<u8>;",
        "static LIMIT: [u8; 4] = [0; 4];",
    ];

    /// The shapes are decided by `;` versus `{`, not by the keyword, so every brace-less
    /// item form has to behave the same way. Each of these is followed by a braced item that
    /// must survive.
    #[test]
    fn Test_Every_Braceless_Item_Form_Should_Blank_Only_Itself()
    {
        for declaration in BRACELESS_ITEM_FORMS
        {
            let source = format!("\n#[cfg(test)]\n{declaration}\n\npub fn Survivor() {{}}\n");
            let blanked = Without_Test_Modules(&source);
            assert!(
                blanked.contains("pub fn Survivor()"),
                "`{declaration}` ate the item after it: {blanked}"
            );
            assert!(
                !blanked.contains("#[cfg(test)]"),
                "`{declaration}` was not blanked: {blanked}"
            );
        }
    }

    /// Generics, a where-clause and an array-typed parameter all put punctuation between the
    /// attribute and the body without ending the item. The `;` inside `[u8; 4]` is the one
    /// that would fool a scanner reading for the first `;` anywhere.
    #[test]
    fn Test_A_Body_Behind_Generics_And_An_Array_Type_Should_Still_Be_Blanked()
    {
        let source = r"
#[cfg(test)]
fn Helper<T>(value: [u8; 4]) -> Result<T, ()>
where
    T: Default,
{
    let Eaten_Marker = value;
    return Ok(T::default());
}

pub fn Survivor() {}
";

        let blanked = Without_Test_Modules(source);
        assert!(
            !blanked.contains("Eaten_Marker"),
            "the body was not blanked, so the `;` in [u8; 4] was read as the item's end: {blanked}"
        );
        assert!(blanked.contains("pub fn Survivor()"), "blanking reached past the body: {blanked}");
    }

    /// The shape is decided over code bytes only. A `;` in a comment between the attribute
    /// and the body would otherwise turn a braced item into a declaration and leave the body
    /// in the text — the failure direction that flatters, because it reports more surface
    /// than the crate has.
    #[test]
    fn Test_A_Semicolon_In_A_Comment_Should_Not_End_An_Item()
    {
        let source = r#"
#[cfg(test)]
// a declaration would have ended here;
mod tests
{
    const NOTE: &str = "and here;";
    fn Inside_The_Body() {}
}

pub fn Survivor() {}
"#;

        let blanked = Without_Test_Modules(source);
        assert!(
            !blanked.contains("Inside_The_Body"),
            "a `;` in a comment was read as the end of a braced item: {blanked}"
        );
        assert!(blanked.contains("pub fn Survivor()"), "blanking reached past the body: {blanked}");
    }

    /// A crate root declaring a production module, then a test module that says where it
    /// lives — the shape all three of this workspace's `#[path]`-declared test modules are
    /// written in.
    const DECLARING_FILE: &str = r#"
mod production;

#[cfg(test)]
#[path = "reading/tests.rs"]
mod tests;
"#;

    /// The file that declaration points at, and the path no home derived from `lib.rs`'s own
    /// name could ever reach.
    const TEST_MODULE: &str = "reading/tests.rs";

    /// The module that has to survive the scan, so that a rule excluding everything cannot
    /// pass this test by being wrong in the other direction.
    const PRODUCTION_MODULE: &str = "production.rs";

    /// The defect this file's doc comment names, at the level that mattered: not the scanner
    /// blanking the wrong bytes but the file walk keeping a test module in the scan.
    ///
    /// The line immediately above `mod tests;` here is `#[path = "reading/tests.rs"]` and not
    /// the marker, so the single-line rule this replaces read the declaration as production
    /// and every caller of `Source_Files` was handed a unit test as though it were the crate.
    /// The control is the second assertion: `production.rs` must still be found, which is what
    /// stops the test passing on a scan that returns nothing.
    #[test]
    fn Test_A_Test_Module_Behind_A_Path_Attribute_Should_Still_Be_Excluded()
    {
        let root = Temporary_Root();
        Write_File(&root, "src/lib.rs", DECLARING_FILE);
        Write_File(&root, &format!("src/{TEST_MODULE}"), "pub fn Helper_Inside_Tests() {}\n");
        Write_File(&root, &format!("src/{PRODUCTION_MODULE}"), "pub fn Real() {}\n");

        let found = Source_Files(&root.join("src"));

        assert!(
            !found.iter().any(|path| return path.ends_with(TEST_MODULE)),
            "a #[cfg(test)] #[path = \"…\"] mod tests; declaration was read as production, so \
             the test module it names is still in the scan: {found:?}"
        );
        assert!(
            found.iter().any(|path| return path.ends_with(PRODUCTION_MODULE)),
            "the scan excluded the production module too, so its verdict on the declaration \
             above says nothing: {found:?}"
        );
    }

    /// A directory this test owns, named so that no other test and no earlier run of the suite
    /// can share it.
    fn Temporary_Root() -> PathBuf
    {
        return std::env::temp_dir()
            .join(format!("nomos-test-only-modules-{}", std::process::id()));
    }

    /// One file under `root`, with whatever directories its path names created first.
    fn Write_File(root: &Path, relative: &str, contents: &str)
    {
        let path = root.join(relative);
        if let Some(parent) = path.parent()
        {
            std::fs::create_dir_all(parent)
                .expect("this test owns its temporary root and can create directories in it");
        }

        std::fs::write(&path, contents).expect("a file in a directory this test owns can be written");
    }
}
