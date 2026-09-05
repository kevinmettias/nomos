//! A signature that borrows an owning container, ported from code-standards'
//! `check-borrowing` (`rules/language-specific/rust/borrowing-and-ownership/borrowing`).
//!
//! `&String`, `&Vec<T>`, `&PathBuf` and `&Box<T>` in a parameter position each ask the
//! caller for a borrow of a *container* when what the body can actually use is the view
//! inside it. The cost lands on the caller: somebody holding a `&str` has to allocate a
//! `String` purely to satisfy the signature, and the allocation exists for the call and
//! nothing else. `&str`, `&[T]`, `&Path` and `&T` accept everything the owning form accepts
//! and more, so the narrower parameter type is strictly the more permissive contract.
//!
//! The rule's own documents name two ideas and this judges the first: a parameter borrows
//! unless it takes ownership, and a signature borrows to read, borrows mutably to modify,
//! and takes by value to consume. The second is a judgment about what a body *does* with
//! its argument, which no line scanner reaches; the pattern below is the part of it that is
//! decidable from a signature alone. The second document,
//! `borrow-when-reading-mutable-borrow-when-modifying-value-when-consuming`, is named here
//! and deliberately given no rule id of its own: an identifier exported for a judgment
//! nothing makes is a rule that reports clean because it does not exist, which is the
//! reading `P43-EMPTY-POPULATION-IS-NOT-CLEAN` is about.
//!
//! # Why the colon is part of every pattern
//!
//! Each pattern requires a preceding `:`, so it matches a *typed position* — a parameter, a
//! struct field, a binding annotation — and not a value. `let held = &owner.name;` borrows a
//! `String` and is not what this rule is about: the borrow there is a use, not a contract
//! offered to a caller. Dropping the colon turned this into a rule against the `&` operator.

use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// The code-standards borrowing rule id this judges.
pub const PARAMETERS_BORROW_UNLESS_OWNERSHIP_IS_TAKEN: &str = "parameters-borrow-unless-ownership-is-taken";

/// One owning container that should have been borrowed as its own unsized view, and the
/// view to name instead.
struct BorrowedContainer
{
    /// The container's type name as it appears after the `&`.
    container: &'static str,
    /// Whether the container carries type arguments, so `Vec<u8>` is matched by its opening
    /// angle bracket and `String` by a word boundary. Getting this wrong made `Stringly` a
    /// `String` and `Vector` a `Vec`.
    opens_type_arguments: bool,
    /// The view a caller can supply without allocating.
    view: &'static str,
}

/// The four containers the rule names. Each is a type whose borrowed form has a strictly
/// more permissive unsized counterpart in the standard library.
const BORROWED_CONTAINERS: [BorrowedContainer; 4] = [
    BorrowedContainer { container: "String", opens_type_arguments: false, view: "&str" },
    BorrowedContainer { container: "Vec", opens_type_arguments: true, view: "&[T]" },
    BorrowedContainer { container: "PathBuf", opens_type_arguments: false, view: "&Path" },
    BorrowedContainer { container: "Box", opens_type_arguments: true, view: "&T" },
];

/// Reports every Rust line that borrows an owning container in a typed position.
#[must_use]
pub fn Check_Parameters_Borrow_Unless_Ownership_Is_Taken(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources.iter().filter(|source| return source.Is_Written_In(RUST_LANGUAGE))
    {
        findings.extend(Borrowed_Container_Findings_In(source));
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// At most one finding per line, so a signature naming two of these reports the first
/// rather than turning one repair into two rows.
fn Borrowed_Container_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for (index, line) in source.text.lines().enumerate()
    {
        let code = Code_Prefix(line);

        if let Some(found) = BORROWED_CONTAINERS.iter().find(|container| return Borrows(code, container))
        {
            findings.push(Borrowed_Container_Finding(source, index.saturating_add(1), found));
        }
    }

    return findings;
}

/// Whether `code` borrows `container` in a typed position: a colon, optional space, an
/// ampersand, optional space, the container's name, and then whatever ends it.
fn Borrows(code: &str, container: &BorrowedContainer) -> bool
{
    let mut rest = code;

    while let Some(offset) = rest.find(':')
    {
        let after_colon = rest.get(offset.saturating_add(1)..).unwrap_or("");

        if Names_Container_After_A_Borrow(after_colon, container)
        {
            return true;
        }

        rest = after_colon;
    }

    return false;
}

/// Whether the text right after a colon is a borrow of `container` and not of some other
/// type whose name merely opens the same way.
fn Names_Container_After_A_Borrow(after_colon: &str, container: &BorrowedContainer) -> bool
{
    let Some(after_ampersand) = after_colon.trim_start().strip_prefix('&')
    else
    {
        return false;
    };
    let Some(after_name) = after_ampersand.trim_start().strip_prefix(container.container)
    else
    {
        return false;
    };
    let trailing = after_name.trim_start();

    if container.opens_type_arguments
    {
        return trailing.starts_with('<');
    }

    return !after_name.starts_with(|character: char| return character.is_ascii_alphanumeric() || character == '_');
}

/// The one finding a borrowed owning container produces.
fn Borrowed_Container_Finding(source: &SourceFile, line_number: usize, container: &BorrowedContainer) -> Finding
{
    let location = format!("{}:{line_number}", source.path);

    return Finding {
        rule: RuleId::New(PARAMETERS_BORROW_UNLESS_OWNERSHIP_IS_TAKEN),
        subject: source.subject,
        subject_name: location.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!(
            "{location} borrows the owning container &{} where {} says the same thing and accepts more; a caller holding the view has to allocate one just to call this",
            container.container, container.view
        ),
        locations: vec![location],
    };
}

/// The code before any line comment. This crate's established per-file convention, which
/// `P45-CODE-PREFIX-KNOWS-STRINGS` will replace with one shared helper.
///
/// String literals are *not* blanked, where the Go original blanks them, so a borrowed
/// container spelled inside a string is judged here and skipped there. Measured before the
/// divergence was accepted: the whole workspace contains no instance of any of these four
/// patterns at all, in code or in a string, so nothing changes verdict for it today. The
/// fixtures below are written so this file does not become the first — see [`Signature`].
fn Code_Prefix(line: &str) -> &str
{
    return line.split("//").next().unwrap_or(line);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_Parameters_Borrow_Unless_Ownership_Is_Taken_Should_Report_Each_Owning_Container()
    {
        let sources = vec![
            Source("demo/src/a.rs", &Signature("String")),
            Source("demo/src/b.rs", &Signature("Vec<u8>")),
            Source("demo/src/c.rs", &Signature("PathBuf")),
            Source("demo/src/d.rs", &Signature("Box<u8>")),
        ];

        let findings = Check_Parameters_Borrow_Unless_Ownership_Is_Taken(&sources);

        assert_eq!(findings.len(), 4, "{findings:?}");
        let found = findings.first().expect("asserted len 4 above");
        assert_eq!(found.rule, RuleId::New(PARAMETERS_BORROW_UNLESS_OWNERSHIP_IS_TAKEN));
        assert_eq!(found.gate, GateCategory::Blocking);
    }

    #[test]
    fn Test_Check_Parameters_Borrow_Unless_Ownership_Is_Taken_Should_Accept_The_Unsized_View()
    {
        let sources = vec![
            Source("demo/src/a.rs", &Signature("str")),
            Source("demo/src/b.rs", &Signature("[u8]")),
            Source("demo/src/c.rs", &Signature("Path")),
            Source("demo/src/d.rs", &Signature("u8")),
        ];

        let findings = Check_Parameters_Borrow_Unless_Ownership_Is_Taken(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// Ownership taken is the exception the rule is named for, so an owned parameter is the
    /// shape it exists to permit rather than one it forgot about.
    #[test]
    fn Test_Check_Parameters_Borrow_Unless_Ownership_Is_Taken_Should_Accept_An_Owned_Parameter()
    {
        let sources = vec![Source("demo/src/a.rs", "fn Judged(name: String, items: Vec<u8>, root: PathBuf) -> usize")];

        let findings = Check_Parameters_Borrow_Unless_Ownership_Is_Taken(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// A type whose name merely opens with a container's is not that container.
    #[test]
    fn Test_Check_Parameters_Borrow_Unless_Ownership_Is_Taken_Should_Not_Match_A_Longer_Type_Name()
    {
        let sources = vec![
            Source("demo/src/a.rs", &Signature("Stringly")),
            Source("demo/src/b.rs", &Signature("Vector")),
            Source("demo/src/c.rs", &Signature("PathBuffer")),
            Source("demo/src/d.rs", &Signature("Boxed")),
        ];

        let findings = Check_Parameters_Borrow_Unless_Ownership_Is_Taken(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// A borrowed container in value position is a use, not a contract offered to a caller.
    /// Only a typed position — which is what the colon marks — is judged.
    #[test]
    fn Test_Check_Parameters_Borrow_Unless_Ownership_Is_Taken_Should_Ignore_A_Borrow_That_Is_Not_A_Type()
    {
        let sources = vec![Source("demo/src/a.rs", "    let held = &owner.name;")];

        let findings = Check_Parameters_Borrow_Unless_Ownership_Is_Taken(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Parameters_Borrow_Unless_Ownership_Is_Taken_Should_Find_A_Container_Past_An_Earlier_Colon()
    {
        let signature = format!("fn Judged(count: usize, name: &{}) -> usize", "String");
        let sources = vec![Source("demo/src/a.rs", &signature)];

        let findings = Check_Parameters_Borrow_Unless_Ownership_Is_Taken(&sources);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Parameters_Borrow_Unless_Ownership_Is_Taken_Should_Report_One_Row_Per_Line()
    {
        let signature = format!("fn Judged(name: &{}, items: &{}) -> usize", "String", "Vec<u8>");
        let sources = vec![Source("demo/src/a.rs", &signature)];

        let findings = Check_Parameters_Borrow_Unless_Ownership_Is_Taken(&sources);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Parameters_Borrow_Unless_Ownership_Is_Taken_Should_Ignore_A_Commented_Signature()
    {
        let commented = format!("// {}", Signature("String"));
        let sources = vec![Source("demo/src/a.rs", &commented)];

        let findings = Check_Parameters_Borrow_Unless_Ownership_Is_Taken(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Parameters_Borrow_Unless_Ownership_Is_Taken_Should_Ignore_A_Language_It_Does_Not_Judge()
    {
        let sources = vec![Source("demo/src/a.go", &Signature("String"))];

        let findings = Check_Parameters_Borrow_Unless_Ownership_Is_Taken(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Parameters_Borrow_Unless_Ownership_Is_Taken_Should_Name_The_Line_It_Found()
    {
        let text = format!("{}\n{{\n}}\n\n{}", Signature("str"), Signature("String"));
        let sources = vec![Source("demo/src/a.rs", &text)];

        let findings = Check_Parameters_Borrow_Unless_Ownership_Is_Taken(&sources);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "demo/src/a.rs:5");
    }

    /// Every fixture is built rather than spelled, so no line of this file's own text ever
    /// carries a colon followed by a borrowed container. Nine modules in this crate answer
    /// the same self-match hazard with a path-suffix exemption instead; that mechanism
    /// exempts any file in any repository whose path ends the same way, and
    /// `P45-CODE-PREFIX-KNOWS-STRINGS` is retiring it, so this file does not add a tenth.
    fn Signature(parameter_type: &str) -> String
    {
        return format!("fn Judged(name: &{parameter_type}) -> usize");
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);
        return source;
    }
}

