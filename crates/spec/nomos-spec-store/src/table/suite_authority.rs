//! Whether a suite is this repository's own specification or one it merely references.

mod authority;
mod title;

pub use authority::SuiteAuthority;
pub use title::SuiteTitle;

/// A suite's own identifier.
///
/// Distinct from `SuiteTitle` even though both are `&str`, because the two sit adjacent
/// at `Put_Suite`'s call site and a position is not a name — a transposed pair used to
/// still compile.
#[derive(Clone, Copy, Debug)]
pub struct SuiteId<'a>(pub &'a str);

impl<'a> From<&'a str> for SuiteId<'a>
{
    fn from(value: &'a str) -> Self
    {
        return Self(value);
    }
}

impl<'a> From<&'a String> for SuiteId<'a>
{
    fn from(value: &'a String) -> Self
    {
        return Self(value.as_str());
    }
}
