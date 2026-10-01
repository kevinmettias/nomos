//! Whether this provider will read a subject at all.
//!
//! Recognition is separate from parsing because the two failures need different
//! responses. A `.md` file this provider skipped is not coverage debt — nothing was
//! expected of it. A `.rs` file it could not parse is, and somebody has to see it.
//! A run that reports both as "no findings" cannot tell a reviewer which of those two
//! worlds they are in.

/// The one extension this provider claims.
///
/// A constant rather than a literal at the comparison site, so that "what does this
/// provider read" is a value a caller can inspect rather than a fact buried in an `if`.
pub const RUST_EXTENSION: &str = "rs";

/// Whether a subject is one this provider will read.
///
/// Deliberately not a `bool`. `Unrecognized` carries the extension it saw, because the
/// useful question after a corpus walk is not "how many were skipped" but "what were
/// they" — and a `bool` throws that away at exactly the moment it becomes interesting.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Recognition
{
    /// Rust source, as far as the path claims.
    Recognized,
    /// Not something this provider reads, and what it was instead.
    ///
    /// `None` means the path had no extension at all, which is a different thing from an
    /// extension this provider does not know.
    Unrecognized
    {
        extension: Option<String>,
    },
}

impl Recognition
{
    /// Decides recognition from a path, textually.
    ///
    /// No filesystem access, on purpose. Recognition must be answerable for a path in a
    /// snapshot manifest that no longer exists on this machine — a snapshot is
    /// interpretable by a process with no access to the tree it came from, and a
    /// recognition that needs to stat the file would be the first thing to break that.
    ///
    /// The comparison folds case because `Main.RS` and `main.rs` are one file on the two
    /// platforms this workspace runs on, and a provider that read one and skipped the
    /// other would report a corpus it had only half walked.
    #[must_use]
    pub fn Of_Path(path: &str) -> Self
    {
        let extension = Extension_Of(path);

        return match extension
        {
            Some(found) if found.eq_ignore_ascii_case(RUST_EXTENSION) => Self::Recognized,
            Some(found) => Self::Unrecognized {
                extension: Some(found.to_lowercase()),
            },
            None => Self::Unrecognized { extension: None },
        };
    }
}

/// The extension a path's file name carries, if it carries one.
///
/// Split on the last dot in the file name rather than in the whole path, so a directory
/// like `crates/nomos.spec/lib` does not lend its dot to a file that has none. A leading
/// dot is a hidden file rather than an extension, which is why the search is over
/// everything after the first character.
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

    /// Paths a Rust provider must recognize regardless of directory separator or case.
    const RECOGNIZED_RUST_PATHS: &[&str] = &[
        "main.rs",
        "src/lib.rs",
        "crates\\kernel\\nomos-model\\src\\digest.rs",
        "F:/repos/xvpe/crates/a/src/b.rs",
        "SRC/MAIN.RS",
    ];

    #[test]
    fn Test_Of_Path_Should_Recognize_Rust_Sources()
    {
        for path in RECOGNIZED_RUST_PATHS
        {
            assert_eq!(
                Recognition::Of_Path(path),
                Recognition::Recognized,
                "`{path}` is Rust source"
            );
        }
    }

    /// The negative control for the test above. If recognition accepted everything, that
    /// test would pass while this provider claimed the whole corpus including its
    /// changelogs.
    #[test]
    fn Test_Other_Extensions_Should_Be_Unrecognized_By_Name()
    {
        assert_eq!(
            Recognition::Of_Path("Cargo.toml"),
            Recognition::Unrecognized {
                extension: Some("toml".to_owned())
            }
        );
        assert_eq!(
            Recognition::Of_Path("docs/README.MD"),
            Recognition::Unrecognized {
                extension: Some("md".to_owned())
            }
        );
    }

    /// `rs` inside a longer extension is not `rs`. A `starts_with` or a `contains` would
    /// let `main.rs.bak` through, and the provider would parse a backup as source.
    #[test]
    fn Test_A_Longer_Extension_Should_Not_Be_Rust()
    {
        assert_eq!(
            Recognition::Of_Path("main.rs.bak"),
            Recognition::Unrecognized {
                extension: Some("bak".to_owned())
            }
        );
        assert_eq!(
            Recognition::Of_Path("notes.rsx"),
            Recognition::Unrecognized {
                extension: Some("rsx".to_owned())
            }
        );
    }

    /// No extension is its own answer, distinct from an unknown one. `Makefile` was
    /// never going to be Rust; `foo.zig` might have been, if somebody added a provider.
    /// Paths that have no extension at all, distinct from an unrecognized one.
    const EXTENSIONLESS_PATHS: &[&str] = &["Makefile", "src/Makefile", ".gitignore", "crates/a.b/LICENSE"];

    #[test]
    fn Test_A_Missing_Extension_Should_Say_So_Rather_Than_Guess()
    {
        for path in EXTENSIONLESS_PATHS
        {
            assert_eq!(
                Recognition::Of_Path(path),
                Recognition::Unrecognized { extension: None },
                "`{path}` has no extension"
            );
        }
    }
}
