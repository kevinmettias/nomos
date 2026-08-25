//! Whether this provider will read a subject at all.
//!
//! The same split `nomos-lang-rust`'s own module doc states the reason for: a file this
//! provider skipped is not coverage debt, a file it could not parse is.

/// The one extension this provider claims.
pub const GO_EXTENSION: &str = "go";

/// Whether a subject is one this provider will read.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Recognition
{
    /// Go source, as far as the path claims.
    Recognized,
    /// Not something this provider reads, and what it was instead.
    Unrecognized
    {
        extension: Option<String>,
    },
}

impl Recognition
{
    /// Decides recognition from a path, textually — no filesystem access, for the same
    /// reason `nomos-lang-rust::Recognition::Of_Path` gives.
    #[must_use]
    pub fn Of_Path(path: &str) -> Self
    {
        let extension = Extension_Of(path);

        return match extension
        {
            Some(found) if found.eq_ignore_ascii_case(GO_EXTENSION) => Self::Recognized,
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

    #[test]
    fn Test_Go_Sources_Should_Be_Recognized()
    {
        for path in ["main.go", "src/lib.go", "crates\\a\\b.go", "F:/repos/x/y.go", "SRC/MAIN.GO"]
        {
            assert_eq!(Recognition::Of_Path(path), Recognition::Recognized, "`{path}` is Go source");
        }
    }

    #[test]
    fn Test_Other_Extensions_Should_Be_Unrecognized_By_Name()
    {
        assert_eq!(
            Recognition::Of_Path("go.mod"),
            Recognition::Unrecognized { extension: Some("mod".to_owned()) }
        );
    }

    #[test]
    fn Test_A_Longer_Extension_Should_Not_Be_Go()
    {
        assert_eq!(
            Recognition::Of_Path("main.go.bak"),
            Recognition::Unrecognized { extension: Some("bak".to_owned()) }
        );
    }

    #[test]
    fn Test_A_Missing_Extension_Should_Say_So_Rather_Than_Guess()
    {
        assert_eq!(Recognition::Of_Path("Makefile"), Recognition::Unrecognized { extension: None });
    }
}
