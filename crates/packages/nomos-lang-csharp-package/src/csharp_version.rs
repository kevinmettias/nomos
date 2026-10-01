//! Which C# language versions this package's providers recognize.
//!
//! `PKG-007`'s third version domain, mirrored rather than reused from
//! `nomos_lang_package::RustEdition` or `nomos_lang_go_package::GoVersion`: there is no one
//! typed shape a version label takes across languages, the identical reason
//! `nomos_package::PackageManifest::language_versions` itself gives for staying an unresolved
//! `Vec<String>` until a language-specific crate resolves it.
//!
//! C#'s domain is a genuinely third shape and not a renamed copy of either. Rust's four editions
//! are a small, closed, hand-enumerable set, so a fixed enum is honest there. Go's `go` directive
//! is always `<major>.<minor>`, so a pair of integers is honest there. C#'s `LangVersion` is
//! neither: it was `<major>.<minor>` through `7.3`, and has been a bare `<major>` since `10`, so
//! both spellings are whole, current, well-formed versions and a type that demanded a minor would
//! refuse every version released since 2021.
//!
//! No enumeration of which particular values are "real" C# versions, for the same reason
//! `GoVersion` gives none: nothing about `tree-sitter-c-sharp`'s grammar or `nomos-lang-csharp`'s
//! own recognition varies by which language version produced a `.cs` file, so a package claiming
//! any well-formed version is not a claim this workspace has any way to falsify.

/// One C# language version, as a project's own `LangVersion` spells it.
///
/// # Ordering
///
/// Derived, so it compares the major first and then the minor — and a version stating no minor
/// sorts *before* the same major stating one, because `Option::None` orders before `Some(0)`.
/// That is not a claim that `12` is an earlier version than `12.0`; it is the consequence of
/// treating the two as the two different things a manifest can say, which is what this type is
/// for. `Test_A_Version_Stating_No_Minor_Should_Sort_Before_The_Same_Major_Stating_One` pins it
/// so the edge is declared rather than discovered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CsharpVersion
{
    pub major: u16,
    /// The minor component, when the label states one. `None` for `12`, `Some(3)` for `7.3`.
    pub minor: Option<u16>,
}

impl CsharpVersion
{
    #[must_use]
    pub const fn New(major: u16, minor: Option<u16>) -> Self
    {
        return Self { major, minor };
    }

    /// The label a manifest file spells this version with.
    #[must_use]
    pub fn Label(self) -> String
    {
        return match self.minor
        {
            Some(minor) => format!("{}.{minor}", self.major),
            None => self.major.to_string(),
        };
    }

    /// The version a label names, or `None` if it is neither `<major>` nor `<major>.<minor>` with
    /// every component a plain non-negative integer.
    ///
    /// Rejects a patch component (`7.3.1`) and a suffix (`12preview`) rather than truncating
    /// either silently: a label this reader could not fully account for is not one it should
    /// claim to have understood.
    ///
    /// Rejects `latest`, `preview`, `latestMajor` and `default` too, and those are the
    /// interesting refusals. They are real, valid `LangVersion` values — and each of them names
    /// whatever version the toolchain that reads it happens to be, which is not a version this
    /// manifest can be said to recognize. A package claiming `latest` would claim something that
    /// changes without the package changing.
    #[must_use]
    pub fn Of_Label(label: &str) -> Option<Self>
    {
        let Some((major, minor)) = label.split_once('.')
        else
        {
            return Some(Self::New(Whole_Number(label)?, None));
        };

        let major = Whole_Number(major)?;
        let minor = Whole_Number(minor)?;

        return Some(Self::New(major, Some(minor)));
    }
}

/// One label component as a number, or `None` when it is empty or not wholly numeric.
///
/// `u16::from_str_radix` would take a leading `+`, and `parse` takes a leading `+` too, so the
/// digit check is what refuses `+12` — a label nobody writes, and a label this type must not
/// silently accept as `12` either.
fn Whole_Number(component: &str) -> Option<u16>
{
    if component.is_empty() || !component.chars().all(|character| return character.is_ascii_digit())
    {
        return None;
    }

    return component.parse().ok();
}

impl core::fmt::Display for CsharpVersion
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(&self.Label());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The minor component of `7.3`, C#'s last version to carry one.
    const LAST_MINOR_COMPONENT: u16 = 3;
    /// A major-only version, as every C# release since 2021 is spelled.
    const MAJOR_ONLY: u16 = 12;
    /// `7`, the major of the last version to carry a minor.
    const LAST_MAJOR_WITH_A_MINOR: u16 = 7;

    #[test]
    fn Test_Both_Label_Shapes_Should_Round_Trip()
    {
        let versions = [
            CsharpVersion::New(LAST_MAJOR_WITH_A_MINOR, Some(LAST_MINOR_COMPONENT)),
            CsharpVersion::New(MAJOR_ONLY, None),
        ];

        for version in versions
        {
            assert_eq!(CsharpVersion::Of_Label(&version.Label()), Some(version));
        }
    }

    /// The shape neither sibling domain has: a whole version with no minor at all.
    #[test]
    fn Test_A_Major_Only_Label_Should_Resolve()
    {
        assert_eq!(CsharpVersion::Of_Label("12"), Some(CsharpVersion::New(MAJOR_ONLY, None)));
    }

    #[test]
    fn Test_A_Version_With_A_Patch_Component_Should_Not_Resolve()
    {
        assert_eq!(CsharpVersion::Of_Label("7.3.1"), None);
    }

    #[test]
    fn Test_A_Version_With_A_Non_Numeric_Component_Should_Not_Resolve()
    {
        assert_eq!(CsharpVersion::Of_Label("12preview"), None);
        assert_eq!(CsharpVersion::Of_Label("v12"), None);
        assert_eq!(CsharpVersion::Of_Label("+12"), None);
    }

    /// The refusal that matters: a floating label names whatever the toolchain is, which is not
    /// a version a manifest can claim to recognize.
    #[test]
    fn Test_A_Floating_Label_Should_Not_Resolve()
    {
        for label in ["latest", "preview", "latestMajor", "default"]
        {
            assert_eq!(CsharpVersion::Of_Label(label), None, "`{label}` names no particular version");
        }
    }

    #[test]
    fn Test_An_Empty_Or_Dotted_Label_Should_Not_Resolve()
    {
        assert_eq!(CsharpVersion::Of_Label(""), None);
        assert_eq!(CsharpVersion::Of_Label("."), None);
        assert_eq!(CsharpVersion::Of_Label("12."), None);
        assert_eq!(CsharpVersion::Of_Label(".3"), None);
    }

    #[test]
    fn Test_Display_Should_Render_The_Label()
    {
        assert_eq!(CsharpVersion::New(MAJOR_ONLY, None).to_string(), "12");
        assert_eq!(CsharpVersion::New(LAST_MAJOR_WITH_A_MINOR, Some(LAST_MINOR_COMPONENT)).to_string(), "7.3");
    }

    #[test]
    fn Test_Ordering_Should_Compare_Major_Then_Minor()
    {
        assert!(CsharpVersion::New(LAST_MAJOR_WITH_A_MINOR, Some(0)) < CsharpVersion::New(LAST_MAJOR_WITH_A_MINOR, Some(LAST_MINOR_COMPONENT)));
        assert!(CsharpVersion::New(LAST_MAJOR_WITH_A_MINOR, Some(LAST_MINOR_COMPONENT)) < CsharpVersion::New(MAJOR_ONLY, None));
    }

    /// The edge the derived ordering produces, declared here rather than left to be discovered:
    /// a label stating no minor is a different thing from one stating `.0`, and this type keeps
    /// them apart in comparison as well as in equality.
    #[test]
    fn Test_A_Version_Stating_No_Minor_Should_Sort_Before_The_Same_Major_Stating_One()
    {
        assert!(CsharpVersion::New(MAJOR_ONLY, None) < CsharpVersion::New(MAJOR_ONLY, Some(0)));
        assert_ne!(CsharpVersion::New(MAJOR_ONLY, None), CsharpVersion::New(MAJOR_ONLY, Some(0)));
    }
}
