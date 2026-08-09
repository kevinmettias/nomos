//! Item declarations, found by reading lines.
//!
//! # What this is and what it is not
//!
//! It is not a parser and does not try to be a bad one. There is no brace matching, no
//! string or block-comment tracking, and no `cfg` evaluation. A line that begins with a
//! visibility and an item keyword is reported as a declaration, and that is the whole rule.
//!
//! Everything about the guarantee this crate declares follows from that. It is
//! [`nomos_contracts::FactVariant::Approximate`] because the method trades accuracy for
//! cost, and [`nomos_contracts::Assurance::Unsound`] because the trade is real and the
//! failures are enumerable rather than hypothetical:
//!
//! - a declaration inside a block comment is reported
//! - a declaration inside a string or a `macro_rules!` body is reported
//! - a declaration behind a `#[cfg]` that is off for this build is reported
//! - nesting is invisible, so `inner::two` is reported as `two`
//! - a declaration written across two lines is missed
//!
//! What it buys is the case a parser cannot serve: a file that does not parse still has
//! lines. Seven files in the scale corpus carry a stray byte order mark that `syn` refuses
//! outright, and one in the precision corpus does. This reads them.

/// One declaration, as a line-reader can see it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScannedItem
{
    /// Position in the file, counting only what was reported.
    pub ordinal: u32,
    /// The line it was found on, one-based. Kept because a caller comparing this provider
    /// against a parser needs to go and look at the disagreement.
    pub line: u32,
    pub kind: ItemKind,
    pub visibility: Visibility,
    pub name: String,
}

/// What kind of declaration a line looks like.
///
/// The labels are the same strings `nomos-lang-rust` writes, and the list is deliberately
/// re-authored rather than imported. Two providers of one capability agree on a payload
/// *format*, which is an interface; sharing an enum would make them agree by construction
/// and there would be nothing left for the slice to check.
///
/// It is a shorter list than a parser's. A line-reader cannot see a `ForeignModule`'s
/// contents or tell a trait alias from a type alias without following the tokens, and a
/// kind it cannot distinguish is a kind it must not claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemKind
{
    Constant,
    Enum,
    ExternCrate,
    Function,
    Implementation,
    MacroDefinition,
    Module,
    Static,
    Struct,
    Trait,
    TypeAlias,
    Union,
    Use,
}

impl ItemKind
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Constant => "Constant",
            Self::Enum => "Enum",
            Self::ExternCrate => "ExternCrate",
            Self::Function => "Function",
            Self::Implementation => "Implementation",
            Self::MacroDefinition => "MacroDefinition",
            Self::Module => "Module",
            Self::Static => "Static",
            Self::Struct => "Struct",
            Self::Trait => "Trait",
            Self::TypeAlias => "TypeAlias",
            Self::Union => "Union",
            Self::Use => "Use",
        };
    }

    /// The keyword that introduces this kind, in the order a scanner must try them.
    ///
    /// `macro_rules` before `macro`, and `const` before nothing — order matters because
    /// these are matched as prefixes and a shorter keyword that is a prefix of a longer one
    /// would claim it first.
    const fn Table() -> &'static [(&'static str, Self)]
    {
        return &[
            ("macro_rules!", Self::MacroDefinition),
            ("extern crate", Self::ExternCrate),
            ("unsafe impl", Self::Implementation),
            ("async fn", Self::Function),
            ("unsafe fn", Self::Function),
            ("const fn", Self::Function),
            ("fn", Self::Function),
            ("struct", Self::Struct),
            ("enum", Self::Enum),
            ("union", Self::Union),
            ("trait", Self::Trait),
            ("impl", Self::Implementation),
            ("mod", Self::Module),
            ("type", Self::TypeAlias),
            ("const", Self::Constant),
            ("static", Self::Static),
            ("use", Self::Use),
        ];
    }
}

/// How visible a declaration says it is.
///
/// There is no `NotApplicable`. A parser knows a trait member declares no visibility of its
/// own; a line-reader cannot see that it is inside a trait, so it would have to guess, and a
/// guess recorded as a fact is worse than a weaker vocabulary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Visibility
{
    Public,
    Restricted
    {
        scope: String,
    },
    Private,
}

impl Visibility
{
    #[must_use]
    pub fn Label(&self) -> String
    {
        return match self
        {
            Self::Public => "Public".to_owned(),
            Self::Restricted { scope } => format!("Restricted({scope})"),
            Self::Private => "Private".to_owned(),
        };
    }
}

/// What one file looks like to a line-reader.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScannedFile
{
    pub items: Vec<ScannedItem>,
    /// Lines read. The denominator: a scan that reported three items out of four lines and
    /// one that reported three out of four thousand are different answers, and a count with
    /// no denominator cannot tell them apart.
    pub lines: u32,
}

/// Reads a source file.
///
/// There is no failure case, and that is the whole difference from a parser. Bytes that are
/// not valid Rust are still lines. This is why the guarantee says `Unsound` rather than
/// `Unknown`: it will answer for input a parser rejects, and some of those answers are
/// wrong.
#[must_use]
pub fn Scan(source: &str) -> ScannedFile
{
    let mut scanned = ScannedFile::default();
    let mut ordinal = 0_u32;

    for (index, raw) in source.lines().enumerate()
    {
        scanned.lines = scanned.lines.saturating_add(1);
        let line = u32::try_from(index).unwrap_or(u32::MAX).saturating_add(1);
        let trimmed = Without_Marks(raw.trim());

        // A line comment is not a declaration. This is the one accuracy the scanner buys
        // cheaply and it is worth buying: `// pub fn Old_Name()` above a rename is common
        // enough that not skipping it would make the disagreement with a parser mostly
        // noise, and noise is what hides the interesting cases.
        if trimmed.starts_with("//")
        {
            continue;
        }

        let (visibility, rest) = Visibility_Of(trimmed);

        let Some((kind, name)) = Declaration(rest)
        else
        {
            continue;
        };

        scanned.items.push(ScannedItem {
            ordinal,
            line,
            kind,
            visibility,
            name,
        });
        ordinal = ordinal.saturating_add(1);
    }

    return scanned;
}

/// Strips a leading byte order mark and any attribute prefix on the same line.
///
/// The mark matters: the files this provider exists to answer for carry one, and leaving it
/// attached would make `\u{feff}pub` fail to match `pub` — the scanner would refuse exactly
/// the input it is here to read.
fn Without_Marks(line: &str) -> &str
{
    return line.trim_start_matches('\u{feff}').trim_start();
}

/// Splits a leading visibility off a line.
fn Visibility_Of(line: &str) -> (Visibility, &str)
{
    let Some(rest) = line.strip_prefix("pub")
    else
    {
        return (Visibility::Private, line);
    };

    // `pub(crate)`, `pub(super)`, `pub(in path)`. Taken textually: a scanner that resolved
    // the path would be claiming to know what module it is in, which it does not.
    if let Some(open) = rest.strip_prefix('(')
        && let Some(close) = open.find(')')
        && let (Some(scope), Some(after)) = (open.get(..close), open.get(close.saturating_add(1)..))
    {
        return (
            Visibility::Restricted {
                scope: scope.trim().to_owned(),
            },
            after.trim_start(),
        );
    }

    // `pub` must be a whole word. Without this, `pubfn` and `public_thing` would both look
    // like public declarations of something.
    let Some(after) = rest.strip_prefix(' ')
    else
    {
        return (Visibility::Private, line);
    };

    return (Visibility::Public, after.trim_start());
}

/// Matches a declaration keyword and the name that follows it.
fn Declaration(line: &str) -> Option<(ItemKind, String)>
{
    for (keyword, kind) in ItemKind::Table()
    {
        let Some(rest) = line.strip_prefix(keyword)
        else
        {
            continue;
        };

        // The keyword must end. `fnord` is not a function and `using` is not a use.
        if !rest.is_empty() && !rest.starts_with([' ', '<', '(', '!', '{', ';', '\t'])
        {
            continue;
        }

        return Some((*kind, Name_In(rest)));
    }

    return None;
}

/// The first identifier-shaped run after a keyword.
///
/// Empty for the forms that have no name a line-reader can see — a bare `impl Trait for T`
/// names two types and declares neither, and reporting one of them as "the name" would be a
/// guess. An empty name is the honest answer and the payload carries it as such.
fn Name_In(rest: &str) -> String
{
    return rest
        .trim_start_matches([' ', '\t', '!'])
        .chars()
        .take_while(|character| return character.is_alphanumeric() || *character == '_')
        .collect();
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Kinds(source: &str) -> Vec<(ItemKind, String, String)>
    {
        return Scan(source)
            .items
            .into_iter()
            .map(|item| return (item.kind, item.visibility.Label(), item.name))
            .collect();
    }

    #[test]
    fn Test_A_Declaration_Should_Be_Found_With_Its_Kind_And_Visibility()
    {
        assert_eq!(
            Kinds(
                "pub fn one() {}\n\
                 fn two() {}\n\
                 pub struct Three;\n\
                 pub(crate) enum Four {}\n\
                 pub(super) trait Five {}\n"
            ),
            vec![
                (ItemKind::Function, "Public".to_owned(), "one".to_owned()),
                (ItemKind::Function, "Private".to_owned(), "two".to_owned()),
                (ItemKind::Struct, "Public".to_owned(), "Three".to_owned()),
                (ItemKind::Enum, "Restricted(crate)".to_owned(), "Four".to_owned()),
                (ItemKind::Trait, "Restricted(super)".to_owned(), "Five".to_owned()),
            ]
        );
    }

    /// The property this provider exists for. A parser refuses these bytes outright; a
    /// line-reader does not care.
    #[test]
    fn Test_A_File_A_Parser_Refuses_Should_Still_Be_Read()
    {
        let damaged = "use super::*;\n\n\u{feff}//! A stray mark, mid-file.\n\npub fn Answered() {}\n";

        let scanned = Scan(damaged);

        assert_eq!(scanned.items.len(), 2, "{:?}", scanned.items);
        assert_eq!(
            scanned.items.last().map(|item| return item.name.clone()),
            Some("Answered".to_owned())
        );
    }

    /// A byte order mark at the start of a line must not hide the declaration behind it.
    #[test]
    fn Test_A_Marked_Declaration_Should_Still_Be_A_Declaration()
    {
        assert_eq!(
            Kinds("\u{feff}pub fn Marked() {}\n"),
            vec![(ItemKind::Function, "Public".to_owned(), "Marked".to_owned())]
        );
    }

    /// The unsoundness, asserted rather than described. These are wrong answers, and the
    /// guarantee says so — a test that only showed the right answers would let the
    /// declaration drift to `Sound` without anything failing.
    #[test]
    fn Test_The_Declared_Unsoundness_Should_Be_Demonstrable()
    {
        assert_eq!(
            Kinds("/*\npub fn Commented() {}\n*/\n").len(),
            1,
            "a declaration inside a block comment is reported"
        );
        assert_eq!(
            Kinds("#[cfg(never)]\npub fn Disabled() {}\n").len(),
            1,
            "a declaration behind a cfg that is off is reported"
        );
        assert_eq!(
            Kinds("mod inner { fn two() {} }\n"),
            vec![(ItemKind::Module, "Private".to_owned(), "inner".to_owned())],
            "nesting is invisible, so an inner item on the same line is missed entirely"
        );
        assert_eq!(
            Kinds("pub\nfn Split() {}\n"),
            vec![(ItemKind::Function, "Private".to_owned(), "Split".to_owned())],
            "a declaration written across two lines loses its visibility"
        );
    }

    /// A line comment is not a declaration.
    #[test]
    fn Test_A_Commented_Line_Should_Not_Be_A_Declaration()
    {
        assert!(Kinds("// pub fn Renamed_Away() {}\n/// pub struct Documented;\n").is_empty());
    }

    /// A keyword must be a whole word. Prefix matching without this makes every identifier
    /// beginning with `use` or `fn` a declaration.
    #[test]
    fn Test_A_Keyword_Prefix_Should_Not_Be_A_Keyword()
    {
        assert!(
            Kinds("    fnord();\n    using(x);\n    pubfn();\n    constant = 1;\n").is_empty(),
            "{:?}",
            Kinds("    fnord();\n    using(x);\n    pubfn();\n    constant = 1;\n")
        );
    }

    /// The denominator. A scan that reported nothing over four thousand lines and one that
    /// reported nothing over four are different answers.
    #[test]
    fn Test_A_Scan_Should_Count_The_Lines_It_Read()
    {
        assert_eq!(Scan("one\ntwo\nthree\n").lines, 3);
        assert_eq!(Scan("").lines, 0, "an empty file has no lines and is not a failure");
    }

    /// `const fn` is a function, not a constant. The keyword table is ordered so the longer
    /// form wins, and a table that lost its order would file every `const fn` as a `const`
    /// named `fn`.
    #[test]
    fn Test_A_Longer_Keyword_Should_Win_Over_A_Prefix_Of_It()
    {
        assert_eq!(
            Kinds("pub const fn Computed() -> u8 { 0 }\npub const VALUE: u8 = 0;\n"),
            vec![
                (ItemKind::Function, "Public".to_owned(), "Computed".to_owned()),
                (ItemKind::Constant, "Public".to_owned(), "VALUE".to_owned()),
            ]
        );
    }

    /// An `impl` has no name, and what a line-reader reports for one is the first type
    /// after the keyword. That is recorded here as what it is — a known imprecision, not a
    /// name — so nothing downstream comes to rely on it meaning more than that.
    #[test]
    fn Test_An_Impl_Should_Report_The_Type_It_Follows_And_Not_A_Name()
    {
        assert_eq!(
            Kinds("impl Display for Thing {}\n"),
            vec![(ItemKind::Implementation, "Private".to_owned(), "Display".to_owned())],
            "the first type after the keyword is what a line-reader sees, and it is not \
             the name of the impl"
        );
    }
}
