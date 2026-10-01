//! How widely a declaration is visible, as written.

/// The accessibility an item declares.
///
/// Nine values, where `nomos-lang-rust` has four and `nomos-lang-go` three, and every one of
/// them is a fact about C# rather than a finer grading of the same question.
///
/// C# states accessibility in a modifier list, so it has two *compound* forms whose meaning is
/// not the intersection of the words they are spelled with: `protected internal` is "this
/// assembly or a derived type", wider than either part, and `private protected` is "a derived
/// type in this assembly", narrower than either. Folding them into [`Visibility::Protected`]
/// or [`Visibility::Internal`] would report an accessibility the file does not declare.
///
/// [`Visibility::Unspecified`] is the value that has no counterpart in either sibling provider,
/// and it is the interesting one. In Rust an absent `pub` *is* private, uniformly; in Go the
/// identifier's own first letter decides and there is nothing to omit. In C# an omitted modifier
/// means `private` for a class member, `internal` for a top-level type and `public` for an
/// interface member — a default that depends on the enclosing form. This provider records that
/// the file stated nothing rather than resolving the default: the default is a language rule,
/// and reporting it as the file's declaration would be reporting a translation.
///
/// [`Visibility::NotApplicable`] is a different absence and keeps the mark the payload schema
/// reserves: a finalizer, an enum member, a `using` directive and a namespace cannot carry an
/// accessibility modifier at all, so for those there is no unstated default either.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Visibility
{
    Public,
    Private,
    Protected,
    Internal,
    /// `protected internal` — this assembly or any derived type.
    ProtectedInternal,
    /// `private protected` — a derived type within this assembly.
    PrivateProtected,
    /// `file` — visible only within the declaring file.
    FileLocal,
    /// A form that could state an accessibility and did not.
    Unspecified,
    /// A form that cannot state an accessibility at all.
    NotApplicable,
}

/// Each accessibility C# can state, with the modifier words that state it, most specific
/// first.
///
/// The order is load-bearing: `protected internal` and `private protected` are each a single
/// accessibility, so they are matched before `protected`, `internal` and `private`, which
/// would otherwise claim them a word at a time.
const STATED_BY: &[(Visibility, &[&str])] = &[
    (Visibility::Public, &["public"]),
    (Visibility::ProtectedInternal, &["protected", "internal"]),
    (Visibility::PrivateProtected, &["private", "protected"]),
    (Visibility::Protected, &["protected"]),
    (Visibility::Internal, &["internal"]),
    (Visibility::Private, &["private"]),
    (Visibility::FileLocal, &["file"]),
];

impl Visibility
{
    /// The stable label used in an encoded payload.
    #[must_use]
    pub fn Label(&self) -> &'static str
    {
        return match self
        {
            Self::Public => nomos_cap_syntax::PUBLIC,
            Self::Private => "Private",
            Self::Protected => "Protected",
            Self::Internal => "Internal",
            Self::ProtectedInternal => "ProtectedInternal",
            Self::PrivateProtected => "PrivateProtected",
            Self::FileLocal => "FileLocal",
            Self::Unspecified => "Unspecified",
            Self::NotApplicable => nomos_cap_syntax::NOT_APPLICABLE,
        };
    }

    /// The accessibility a declaration's own modifier words state.
    ///
    /// Reads the words and nothing else. No filesystem access, no name resolution and no
    /// enclosing-form default — the same syntactic-only promise every other judgment in this
    /// crate makes.
    #[must_use]
    pub fn Of_Modifiers(modifiers: &[&str]) -> Self
    {
        for (accessibility, words) in STATED_BY
        {
            if words.iter().all(|word| return modifiers.contains(word))
            {
                return *accessibility;
            }
        }

        return Self::Unspecified;
    }
}

impl core::fmt::Display for Visibility
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Of_Modifiers_Should_Read_A_Single_Accessibility_Word()
    {
        assert_eq!(Visibility::Of_Modifiers(&["public", "static"]), Visibility::Public);
        assert_eq!(Visibility::Of_Modifiers(&["private", "readonly"]), Visibility::Private);
        assert_eq!(Visibility::Of_Modifiers(&["file"]), Visibility::FileLocal);
    }

    /// The load-bearing one: each compound form is its own accessibility, and reading either
    /// word alone would report a different one.
    #[test]
    fn Test_A_Compound_Accessibility_Should_Not_Be_Read_A_Word_At_A_Time()
    {
        assert_eq!(
            Visibility::Of_Modifiers(&["protected", "internal", "virtual"]),
            Visibility::ProtectedInternal
        );
        assert_eq!(Visibility::Of_Modifiers(&["private", "protected"]), Visibility::PrivateProtected);
    }

    /// The negative control for the test above: the compound arms must not be claiming a
    /// declaration that states only one of the two words.
    #[test]
    fn Test_A_Single_Word_Of_A_Compound_Form_Should_Stay_That_Word()
    {
        assert_eq!(Visibility::Of_Modifiers(&["protected"]), Visibility::Protected);
        assert_eq!(Visibility::Of_Modifiers(&["internal"]), Visibility::Internal);
    }

    #[test]
    fn Test_A_Declaration_With_No_Accessibility_Word_Should_Be_Unspecified()
    {
        assert_eq!(Visibility::Of_Modifiers(&[]), Visibility::Unspecified);
        assert_eq!(Visibility::Of_Modifiers(&["static", "partial"]), Visibility::Unspecified);
    }

    #[test]
    fn Test_Label_Should_Match_The_Shared_Vocabulary()
    {
        assert_eq!(Visibility::Public.Label(), nomos_cap_syntax::PUBLIC);
        assert_eq!(Visibility::NotApplicable.Label(), nomos_cap_syntax::NOT_APPLICABLE);
    }

    /// `Unspecified` and `NotApplicable` are two different absences, and a payload a consumer
    /// reads back must keep them apart.
    #[test]
    fn Test_The_Two_Absences_Should_Not_Share_A_Label()
    {
        assert_ne!(Visibility::Unspecified.Label(), Visibility::NotApplicable.Label());
    }
}
