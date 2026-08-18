//! Which Rust editions this package's providers recognize.
//!
//! `PKG-007`'s third version domain: supported language/dialect versions. Neither
//! `nomos-lang-rust` nor `nomos-lang-rust-scan` gates on edition today -- one parses
//! tokens with `syn`, which does not refuse a file for the edition it was written
//! under, and the other reads lines -- so a syntactic answer from either is not a
//! function of which edition produced the source. That is what licenses a package
//! claiming every edition Rust has shipped rather than only the one this workspace
//! happens to compile under.

const EDITION_2015_LABEL: &str = "2015";
const EDITION_2018_LABEL: &str = "2018";
const EDITION_2021_LABEL: &str = "2021";
const EDITION_2024_LABEL: &str = "2024";

/// One Rust edition, spelled the way `Cargo.toml`'s own `edition` field spells it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RustEdition
{
    Edition2015,
    Edition2018,
    Edition2021,
    Edition2024,
}

impl RustEdition
{
    /// The label a manifest file spells this edition with.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Edition2015 => EDITION_2015_LABEL,
            Self::Edition2018 => EDITION_2018_LABEL,
            Self::Edition2021 => EDITION_2021_LABEL,
            Self::Edition2024 => EDITION_2024_LABEL,
        };
    }

    /// The edition a label names, or `None` if it names none of the four Rust has
    /// shipped.
    #[must_use]
    pub fn Of_Label(label: &str) -> Option<Self>
    {
        return match label
        {
            EDITION_2015_LABEL => Some(Self::Edition2015),
            EDITION_2018_LABEL => Some(Self::Edition2018),
            EDITION_2021_LABEL => Some(Self::Edition2021),
            EDITION_2024_LABEL => Some(Self::Edition2024),
            _ => None,
        };
    }
}

impl core::fmt::Display for RustEdition
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

    const EVERY_EDITION: [RustEdition; 4] = [
        RustEdition::Edition2015,
        RustEdition::Edition2018,
        RustEdition::Edition2021,
        RustEdition::Edition2024,
    ];

    #[test]
    fn Test_Every_Edition_Should_Round_Trip_Through_Its_Label()
    {
        for edition in EVERY_EDITION
        {
            assert_eq!(RustEdition::Of_Label(edition.Label()), Some(edition));
        }
    }

    #[test]
    fn Test_An_Edition_Rust_Never_Shipped_Should_Resolve_To_Nothing()
    {
        assert_eq!(RustEdition::Of_Label("2027"), None);
        assert_eq!(RustEdition::Of_Label("edition2024"), None);
        assert_eq!(RustEdition::Of_Label(""), None);
    }

    #[test]
    fn Test_Display_Should_Render_The_Label()
    {
        assert_eq!(RustEdition::Edition2021.to_string(), "2021");
    }
}
