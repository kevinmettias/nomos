//! Reducing a repository path to the subject it denotes.
//!
//! # Why this is in the kernel
//!
//! Three subsystems reduce a path to a [`SubjectId`] and had done it three times: the work
//! ledger addressing a *claim*, the composition root in `nomos-cli` addressing a *fact*,
//! and the integration corpus addressing a fact again. The three normalizations were
//! byte-for-byte the same rule, arrived at independently, and each carried a doc comment
//! explaining that it was a copy.
//!
//! `OD-MODEL-001` records the decision this file implements. The short form of it is that
//! a claim and a fact ask the *same* question of a path spelling — which file is this? —
//! and only the ledger asks a second question on top of it. So the spelling rule is one
//! rule and belongs where [`SubjectSet`](crate::SubjectSet) already is, and the second
//! question stays with the subsystem that asks it.

use nomos_contracts::SubjectId;

/// The identity of the subject a repository-relative path denotes.
///
/// Two spellings of one file are one subject, because a subject is what a rule, a fact or
/// a claim is *about*, and "about" cannot depend on how somebody typed the name.
#[must_use]
pub fn Subject_Of_Path(path: &str) -> SubjectId
{
    use crate::Content_Digest;

    return SubjectId::From_Digest(Content_Digest(Normalize_Path(path).as_bytes()));
}

/// Reduces a path to the text its identity is computed from.
///
/// Separators are unified, `./` prefixes and repeated or trailing separators are dropped,
/// and the result is lowercased.
///
/// # Why case is folded
///
/// On Windows and macOS, `src/Main.rs` and `src/main.rs` are one file. Not folding means
/// two agents claim the same file, both are told the territory is disjoint, and the second
/// one's edit silently replaces the first — and, on the fact side, that one edit
/// invalidates neither of the two subjects it was split across.
///
/// Folding has a cost, and it is the honest one to pay: on Linux those really are two
/// files, so two agents who could have worked in parallel are serialized instead. That
/// costs throughput. The alternative costs an edit, and an edit does not come back.
///
/// # Why the empty string is the root
///
/// `.` segments are dropped along with empty ones, so `.`, `./`, `a/./b` and `a//b` all
/// reduce the same way. This is also what makes a lone `.` normalize to the empty string,
/// which is how the repository root is represented — the root has to be empty rather than
/// a name, or a containment test would compare it as a sibling of everything it actually
/// contains.
///
/// # What this rule deliberately does not do
///
/// It does not touch the filesystem. `a/b` and `A/B` reduce alike whether or not either
/// exists, which is what makes the answer a decision rather than a guess about the machine
/// it ran on.
///
/// It also does not know what a decision record is. The work ledger folds a record
/// filename onto the identifier it carries, because an item must reserve a record before
/// the file exists and the identifier is the only name two items can both write down in
/// advance. That is a fact about coordinating unwritten work, not about what a path
/// spells, and `nomos_ledger::Normalize_Path` applies it on top of this — see
/// `OD-MODEL-001` for why the composition runs in that direction rather than this file
/// growing a flag.
#[must_use]
pub fn Normalize_Path(path: &str) -> String
{
    return path
        .trim()
        .replace('\\', "/")
        .split('/')
        .filter(|segment| return !segment.is_empty() && *segment != ".")
        .collect::<Vec<&str>>()
        .join("/")
        .to_lowercase();
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Subject_Of_Path_Should_Treat_Different_Spellings_Of_One_Path_As_One_Subject()
    {
        let canonical = Subject_Of_Path("alpha/one.rs");

        for spelling in Equivalent_Spellings_Of_One_Path()
        {
            assert_eq!(Subject_Of_Path(spelling), canonical, "`{spelling}`");
        }
    }

    /// Different spellings of the same file, which `Subject_Of_Path` must fold to one
    /// identity.
    fn Equivalent_Spellings_Of_One_Path() -> [&'static str; 5]
    {
        return ["./alpha/one.rs", "alpha\\one.rs", "alpha//one.rs", "Alpha/One.rs", " alpha/one.rs "];
    }

    /// The negative control. If normalization collapsed everything, every file in a corpus
    /// would be one subject and a single edit would invalidate the world.
    #[test]
    fn Test_Different_Paths_Should_Be_Different_Subjects()
    {
        assert_ne!(Subject_Of_Path("alpha/one.rs"), Subject_Of_Path("alpha/two.rs"));
        assert_ne!(Subject_Of_Path("alpha/one.rs"), Subject_Of_Path("beta/one.rs"));
        assert_ne!(Subject_Of_Path("alpha"), Subject_Of_Path("alpha/one.rs"));
        assert_ne!(Subject_Of_Path("a/b.rs"), Subject_Of_Path("a-b.rs"));
    }

    #[test]
    fn Test_Normalize_Path_Should_Reduce_The_Root_To_The_Empty_String()
    {
        for spelling in Spellings_Of_The_Root()
        {
            assert_eq!(Normalize_Path(spelling), "", "`{spelling}`");
        }
    }

    /// Spellings that all denote the repository root, which `Normalize_Path` must reduce
    /// to the empty string.
    fn Spellings_Of_The_Root() -> [&'static str; 5]
    {
        return [".", "./", "/", "", "  "];
    }

    /// The kernel does not know what a decision record is, and must not start knowing.
    /// `nomos-ledger` composes that fold on top; if this rule ever absorbed it, a *fact*
    /// about a record file would silently become a fact about the identifier instead.
    #[test]
    fn Test_A_Record_Filename_Should_Not_Fold_Onto_Its_Identifier()
    {
        assert_ne!(
            Normalize_Path("docs/records/od-ledger-001-a-slug.md"),
            Normalize_Path("docs/records/od-ledger-001")
        );
    }
}
