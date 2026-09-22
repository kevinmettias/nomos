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
/// The attributes a declaration shares its own line with are read too, and a census is why
/// they had to be. That census said no `#[cfg(test)]`-plus-attribute site in this workspace
/// put the attribute and the item on one line, so the spelling was left deliberately
/// unread — and then `6bb62542` wrote `#[path = "currency/tests.rs"] mod tests;` as one line
/// in `nomos-check-orchestration`'s `facts/currency.rs`. The declaration was invisible, the
/// split-out `facts/currency/tests.rs` stayed in the scan, and the `MaterializedFact` its
/// fixture builds was reported as that crate's own production: the determinism check named
/// a crate that constructs no fact anywhere in its crate proper, and CI's Test step was red
/// for every session in the tree.
///
/// So a line is peeled of the attributes it begins with before what remains is asked
/// whether it declares a module. Where the line breaks fall between a marker, a `#[path]`
/// and the `mod x;` they govern is no longer part of the answer, which is the property that
/// generalizes: a module declared `#[cfg(test)]` in a parent file is test material wherever
/// its own file lives, however its parent chose to lay the declaration out.
fn Test_Only_In(file: &Path) -> Vec<PathBuf>
{
    let Ok(text) = std::fs::read_to_string(file)
    else
    {
        return Vec::new();
    };
    let mut excluded = Vec::new();
    let mut above: Vec<&str> = Vec::new();
    for line in text.lines()
    {
        let item = Extend_Attribute_Run(&mut above, line.trim());
        excluded.extend(Test_Only_Homes(file, item, &above));
        if !Keeps_The_Run_Alive(item)
        {
            above.clear();
        }
    }

    return excluded;
}

/// The paths one line's `mod <name>;` points at, when the attributes above it mark it
/// test-only, and nothing when either half of that is missing.
fn Test_Only_Homes(file: &Path, item: &str, above: &[&str]) -> Vec<PathBuf>
{
    let Some(name) = Declared_Module(item).filter(|_| return Marks_Test_Only(above))
    else
    {
        return Vec::new();
    };

    return Homes_Of(file, name, Path_Attribute(above));
}

/// Whether a run of attribute lines marks the item below it as test-only.
///
/// Anywhere in the run and not merely at its start, because the marker and the item may have
/// attributes between them and the order among them is not part of what is being asked.
fn Marks_Test_Only(above: &[&str]) -> bool
{
    return above.iter().any(|line| return *line == MARKER);
}

/// Peels the `#[…]` attributes a line begins with into the run above the item below them,
/// and returns whatever is left of that line.
///
/// Peeling rather than asking whether the whole line *is* an attribute is what lets a
/// declaration be read when it shares its line with the attributes governing it. What the
/// run means is unchanged: it is still every attribute standing between the last item and
/// the next one, in source order, and [`Marks_Test_Only`] still asks only whether the
/// marker is among them.
fn Extend_Attribute_Run<'line>(above: &mut Vec<&'line str>, line: &'line str) -> &'line str
{
    let mut rest = line;
    while let Some((attribute, after)) = Leading_Attribute(rest)
    {
        above.push(attribute);
        rest = after;
    }

    return rest;
}

/// The `#[…]` attribute a line begins with and what follows it, or `None` for a line that
/// does not begin with one.
///
/// A line beginning `#[` whose bracket never closes on it is not peeled at all, which is the
/// under-excluding direction and deliberately so: an exclusion assembled out of an attribute
/// nobody attached would drop a production module from the scan, and a module missing from
/// the crate proper is a claim about the crate that nothing else would contradict.
fn Leading_Attribute(line: &str) -> Option<(&str, &str)>
{
    if !line.starts_with("#[")
    {
        return None;
    }
    let close = Closing_Bracket(line)?;
    let attribute = line.get(..=close)?;
    let after = line.get(close.saturating_add(1)..)?;

    return Some((attribute, after.trim_start()));
}

/// The index of the `]` that closes the `[` a line's leading attribute opens.
///
/// Counted rather than searched for, because an attribute may carry brackets of its own —
/// `#[expect(clippy::indexing_slicing, reason = "…")]` carries none but nothing stops the
/// next one — and the first `]` in the line would then end the attribute early, handing the
/// caller a remainder that begins mid-attribute. `None` is an attribute this line does not
/// close, which a multi-line attribute is and which this scan does not read.
fn Closing_Bracket(line: &str) -> Option<usize>
{
    let mut depth = 0_u32;
    for (index, character) in line.char_indices()
    {
        depth = match character
        {
            '[' => depth.saturating_add(1),
            ']' => depth.saturating_sub(1),
            _ => depth,
        };
        if character == ']' && depth == 0
        {
            return Some(index);
        }
    }

    return None;
}

/// Whether what is left of a line, once its attributes are peeled off it, leaves the run
/// above the next item intact.
///
/// A line that held nothing but attributes, a blank line and a comment all do, because Rust
/// attaches an attribute to the item below it regardless of any of the three — the gap is
/// not a separator in the grammar, and treating it as one here would read a real declaration
/// as production. Anything else is an item's own text, and one item's text is never the next
/// item's attributes.
fn Keeps_The_Run_Alive(item: &str) -> bool
{
    return item.is_empty() || item.starts_with("//");
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
        let root = Temporary_Root("path-attribute");
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

    /// The shape `6bb62542` wrote and this scan could not read: the marker alone on its line,
    /// and the `#[path]` sharing a line with the declaration it governs.
    ///
    /// The declaring file is `currency.rs` rather than a crate root, because a `#[path]` is
    /// resolved from the declaring file's own directory and that is the resolution the real
    /// defect ran through.
    const SHARED_LINE_DECLARING_FILE: &str = r#"
pub fn Materialized_Or_Already_Current() {}

#[cfg(test)]
#[path = "currency/tests.rs"] mod tests;
"#;

    /// Where that declaration's own file sits, and where the module it names sits.
    const SHARED_LINE_DECLARING_MODULE: &str = "currency.rs";
    const SHARED_LINE_TEST_MODULE: &str = "currency/tests.rs";

    /// What the split-out test module builds. A construction rather than a mention, because
    /// construction is the predicate `fact_domain.rs` applies to whatever survives this scan,
    /// and it is the fixture's construction that was read as a crate's production.
    const FIXTURE_CONSTRUCTION: &str = "return Fact_Built_By_Hand {";

    /// The production text that must survive, so that a scan excluding everything cannot pass
    /// this test by being wrong in the other direction.
    const PRODUCTION_TEXT: &str = "pub fn Real() {}\n";

    /// The defect this file's own documentation now names, exercised end to end rather than
    /// against the crates this workspace happens to hold today.
    ///
    /// The fixture lives in a file of its own and the only thing saying so is a `#[cfg(test)]`
    /// in the parent — with the `#[path]` and the `mod tests;` on one line, which is the part
    /// the scan could not read. What it therefore handed `fact_domain.rs` was a fixture's
    /// construction presented as the crate's production, and that is what this asserts against:
    /// the text a caller reads the crate by, not the list of files, because the list is one
    /// step short of the thing that went wrong.
    ///
    /// Reverting `Extend_Attribute_Run` to a whole-line test fails this test's first assertion.
    #[test]
    fn Test_A_Fixture_In_A_Test_Module_Sharing_Its_Attributes_Line_Should_Not_Reach_A_Caller()
    {
        let root = Temporary_Root("shared-line");
        Write_File(&root, &format!("src/{SHARED_LINE_DECLARING_MODULE}"), SHARED_LINE_DECLARING_FILE);
        Write_File(&root, &format!("src/{SHARED_LINE_TEST_MODULE}"), &Fixture_Module());
        Write_File(&root, &format!("src/{PRODUCTION_MODULE}"), PRODUCTION_TEXT);

        let crate_proper = Crate_Proper_Text(&root.join("src"));

        assert!(
            !crate_proper.contains(FIXTURE_CONSTRUCTION),
            "a fixture in a test module whose `#[path]` shares a line with the declaration was \
             handed to a caller as the crate proper: {crate_proper}"
        );
        assert!(
            crate_proper.contains(PRODUCTION_TEXT.trim_end()),
            "the scan excluded the crate's production too, so its verdict above says nothing: \
             {crate_proper}"
        );
    }

    /// A test module whose only content is a fixture that builds something.
    fn Fixture_Module() -> String
    {
        return format!("fn Fixture()\n{{\n    {FIXTURE_CONSTRUCTION} }};\n}}\n");
    }

    /// The text a caller reads a crate by: every file this scan keeps, with every inline test
    /// module in it blanked — the two steps `fact_domain.rs` runs before it asks whether a
    /// crate constructs a fact.
    fn Crate_Proper_Text(root: &Path) -> String
    {
        return Source_Files(root)
            .iter()
            .filter_map(|file| return std::fs::read_to_string(file).ok())
            .map(|text| return Without_Test_Modules(&text))
            .collect();
    }

    /// A directory this test owns, named so that no other test and no earlier run of the suite
    /// can share it.
    ///
    /// The purpose is part of the name because two tests in one process share a process id, and
    /// a tree one of them wrote would otherwise be inside the scan the other measures. Cleared
    /// on the way out for the same reason a run cannot inherit: a stale file left by an earlier
    /// run at this process id would be in the scan too.
    fn Temporary_Root(purpose: &str) -> PathBuf
    {
        let root = std::env::temp_dir()
            .join(format!("nomos-test-only-modules-{purpose}-{}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&root);

        return root;
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
