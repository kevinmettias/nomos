//! Which Go language versions this package's providers recognize.
//!
//! `PKG-007`'s third version domain, mirrored rather than reused from
//! `nomos_lang_package::RustEdition`: there is no one typed shape a version label takes
//! across languages, the identical reason `nomos_package::PackageManifest::language_versions`
//! itself gives for staying an unresolved `Vec<String>` until a language-specific crate
//! resolves it. Go's own domain is a genuinely different shape from Rust's, not merely a
//! renamed copy of it — Go ships a new minor version roughly twice a year and has done so
//! since 1.0, so unlike Rust's four editions (a small, closed, hand-enumerable set this
//! workspace can simply list), a fixed enum of every Go version to date would already be
//! stale by the time it shipped and would need editing on a cadence this crate has no way
//! to keep up with. This domain is instead a parsed `{major, minor}` pair, validated
//! against the grammar `go.mod`'s own `go` directive uses, with no enumeration of which
//! particular values are "real" Go versions at all — the same reason `nomos-lang-go`
//! itself does not gate on Go version: nothing about `tree-sitter-go`'s grammar or this
//! provider's own recognition varies by which minor version produced a `.go` file, so a
//! package claiming any well-formed `{major, minor}` pair is not a claim this workspace
//! has any way to falsify, the identical non-gating argument `RustEdition`'s own module
//! doc gives for Rust.

/// One Go language version, as `go.mod`'s own `go` directive spells it: `<major>.<minor>`,
/// with no patch component — Go's `go` directive has never carried one, unlike a Go
/// module's own semantic-versioned release.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GoVersion
{
    pub major: u16,
    pub minor: u16,
}

impl GoVersion
{
    #[must_use]
    pub const fn New(major: u16, minor: u16) -> Self
    {
        return Self { major, minor };
    }

    /// The label a manifest file spells this version with: `<major>.<minor>`.
    #[must_use]
    pub fn Label(self) -> String
    {
        return format!("{}.{}", self.major, self.minor);
    }

    /// The version a label names, or `None` if it is not `<major>.<minor>` with both
    /// components a plain non-negative integer.
    ///
    /// Rejects a patch component (`1.21.0`) and a pre-release/build suffix (`1.21rc1`)
    /// rather than truncating either silently: a label this reader could not fully
    /// account for is not one it should claim to have understood.
    #[must_use]
    pub fn Of_Label(label: &str) -> Option<Self>
    {
        let (major, minor) = label.split_once('.')?;
        if major.is_empty() || minor.is_empty()
        {
            return None;
        }

        let major: u16 = major.parse().ok()?;
        let minor: u16 = minor.parse().ok()?;

        return Some(Self::New(major, minor));
    }
}

impl core::fmt::Display for GoVersion
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}.{}", self.major, self.minor);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Real_Go_Version_Should_Round_Trip_Through_Its_Label()
    {
        for version in [GoVersion::New(1, 18), GoVersion::New(1, 21), GoVersion::New(1, 23)]
        {
            assert_eq!(GoVersion::Of_Label(&version.Label()), Some(version));
        }
    }

    #[test]
    fn Test_A_Version_With_A_Patch_Component_Should_Not_Resolve()
    {
        assert_eq!(GoVersion::Of_Label("1.21.0"), None);
    }

    #[test]
    fn Test_A_Version_With_A_Non_Numeric_Component_Should_Not_Resolve()
    {
        assert_eq!(GoVersion::Of_Label("1.21rc1"), None);
        assert_eq!(GoVersion::Of_Label("go1.21"), None);
    }

    #[test]
    fn Test_A_Label_With_No_Dot_Should_Not_Resolve()
    {
        assert_eq!(GoVersion::Of_Label("121"), None);
        assert_eq!(GoVersion::Of_Label(""), None);
    }

    #[test]
    fn Test_Display_Should_Render_The_Label()
    {
        assert_eq!(GoVersion::New(1, 21).to_string(), "1.21");
    }

    #[test]
    fn Test_Ordering_Should_Compare_Major_Then_Minor()
    {
        assert!(GoVersion::New(1, 18) < GoVersion::New(1, 21));
        assert!(GoVersion::New(1, 21) < GoVersion::New(2, 0));
    }
}
