//! One `nomos_contracts::Finding::locations` entry, split into what an editor needs: a
//! file, and the line the tool itself reported, if it reported one.
//!
//! `Finding::locations` is not one shape. Most rules that judge source text write
//! `"{path}:{line_number}"` (`concurrency_text.rs`, `formatting.rs`, `error_text.rs`, and
//! a majority of the others under `crates/rules/nomos-rules/src/checks/`); some name a bare
//! path with no line at all (`function_shape.rs`, `orphan_modules.rs`,
//! `reachability.rs`); dependency and policy rules name a package, not a file
//! (`dependency/violations.rs`, `dependency/write_authority.rs`); one workspace-level rule
//! names a fixed declaration file (`goals.rs`). This type reads the one convention that
//! recurs -- an optional trailing `:<digits>` -- and carries a bare location through
//! unparsed rather than guessing a line for it.

/// A location, split into the file (or package, or declaration name) and the one-based
/// line the tool reported, if the location string carried one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Location
{
    pub(crate) path: String,
    pub(crate) line: Option<u32>,
}

impl Location
{
    /// Parses one `Finding::locations` entry.
    ///
    /// Splits on the last `:` only when what follows is entirely digits, so a Windows
    /// drive letter (`C:\...`, never produced by this workspace's own repo-relative
    /// convention, but not this function's job to assume) or a path with no line at all
    /// is carried through whole rather than mis-split.
    #[must_use]
    pub(crate) fn Parse(raw: &str) -> Self
    {
        if let Some((path, line)) = raw.rsplit_once(':')
            && let Ok(line) = line.parse::<u32>()
            && !path.is_empty()
        {
            return Self { path: path.to_owned(), line: Some(line) };
        }

        return Self { path: raw.to_owned(), line: None };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The line number the fixture location string below carries, so the assertion that reads
    /// it back names the line it expects rather than a bare number.
    const REPORTED_LINE: u32 = 42;

    #[test]
    fn Test_Parse_Should_Split_A_Path_And_Line()
    {
        let location = Location::Parse("crates/rules/nomos-rules/src/lib.rs:42");

        assert_eq!(location.path, "crates/rules/nomos-rules/src/lib.rs");
        assert_eq!(location.line, Some(REPORTED_LINE));
    }

    #[test]
    fn Test_Parse_Should_Carry_A_Bare_Path_Through_With_No_Line()
    {
        let location = Location::Parse("crates/rules/nomos-rules/src/lib.rs");

        assert_eq!(location.path, "crates/rules/nomos-rules/src/lib.rs");
        assert_eq!(location.line, None);
    }

    #[test]
    fn Test_Parse_Should_Not_Mistake_A_Package_Name_For_A_Line()
    {
        let location = Location::Parse("nomos-rules");

        assert_eq!(location.path, "nomos-rules");
        assert_eq!(location.line, None);
    }

    #[test]
    fn Test_Parse_Should_Reject_A_Trailing_Colon_With_Nothing_Numeric_After_It()
    {
        let location = Location::Parse("crates/rules/nomos-rules/src/lib.rs:");

        assert_eq!(location.path, "crates/rules/nomos-rules/src/lib.rs:");
        assert_eq!(location.line, None);
    }
}
