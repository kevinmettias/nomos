//! Whether this provider will read a subject at all.
//!
//! The same split `nomos-lang-rust`'s own module doc states the reason for: a file this
//! provider skipped is not coverage debt, a file it could not parse is.

/// The one extension this provider claims.
///
/// One, not several. A C# project carries `.csproj`, `.sln`, `.props`, `.targets` and
/// `.razor` files too, and none of them is C# source this grammar parses — a `.csproj` is
/// `MSBuild` XML and a `.razor` is a template with C# embedded in it. Claiming any of them here
/// would make this provider refuse files it had said it would read, which is the coverage debt
/// the split above exists to keep visible.
pub const CSHARP_EXTENSION: &str = "cs";

/// Whether a subject is one this provider will read.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Recognition
{
    /// C# source, as far as the path claims.
    Recognized,
    /// Not something this provider reads, and what it was instead.
    Unrecognized
    {
        extension: Option<String>,
    },
}

impl Recognition
{
    /// Decides recognition from a path, textually — no filesystem access, for the same reason
    /// `nomos-lang-rust::Recognition::Of_Path` gives.
    #[must_use]
    pub fn Of_Path(path: &str) -> Self
    {
        let extension = Extension_Of(path);

        return match extension
        {
            Some(found) if found.eq_ignore_ascii_case(CSHARP_EXTENSION) => Self::Recognized,
            Some(found) => Self::Unrecognized {
                extension: Some(found.to_lowercase()),
            },
            None => Self::Unrecognized { extension: None },
        };
    }
}

/// The extension a path's file name carries, if it carries one.
fn Extension_Of(path: &str) -> Option<&str>
{
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);

    return name
        .rfind('.')
        .filter(|at| return *at > 0)
        .and_then(|at| return name.get(at.saturating_add(1)..))
        .filter(|extension| return !extension.is_empty());
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// Paths a C# provider must recognize regardless of directory separator or case.
    const RECOGNIZED_CSHARP_PATHS: &[&str] =
        &["Program.cs", "src/Widget.cs", "crates\\a\\B.cs", "F:/repos/x/Y.cs", "SRC/PROGRAM.CS"];

    #[test]
    fn Test_Of_Path_Should_Recognize_Csharp_Sources()
    {
        for path in RECOGNIZED_CSHARP_PATHS
        {
            assert_eq!(Recognition::Of_Path(path), Recognition::Recognized, "`{path}` is C# source");
        }
    }

    #[test]
    fn Test_A_Project_File_Should_Be_Unrecognized_By_Name()
    {
        assert_eq!(
            Recognition::Of_Path("Acme.Widgets.csproj"),
            Recognition::Unrecognized { extension: Some("csproj".to_owned()) }
        );
    }

    #[test]
    fn Test_A_Longer_Extension_Should_Not_Be_Csharp()
    {
        assert_eq!(
            Recognition::Of_Path("Program.cs.bak"),
            Recognition::Unrecognized { extension: Some("bak".to_owned()) }
        );
    }

    #[test]
    fn Test_A_Missing_Extension_Should_Say_So_Rather_Than_Guess()
    {
        assert_eq!(Recognition::Of_Path("Makefile"), Recognition::Unrecognized { extension: None });
    }
}
