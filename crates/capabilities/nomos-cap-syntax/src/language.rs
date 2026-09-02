//! Which language a subject is written in.

/// Which language a subject is written in.
///
/// Authored, not digested: a language crate declares its own name beside its `PROVIDER`
/// identity, and this wraps that name so two authored identifiers cannot be swapped for
/// each other by the compiler. The shape is `nomos-contracts`' own `Named_Identity`, minus
/// the `serde` derives this crate has no dependency for.
///
/// # Why this is not a provider identity
///
/// A provider identity names the implementation that reads a file; this names the language
/// the file is written in, and the two are not one question. `nomos.cap.syntax.items` has
/// three registered offers — `nomos.lang.rust.syn`, `nomos.lang.rust.scan` and
/// `nomos.lang.go.tree-sitter` — so two of the three are the same language, and the
/// identities name reading technology rather than language. A caller asking "is this Rust"
/// against one of those identities asks a narrower question than it means, and gets a wrong
/// answer silently the first time recognition prefers the other Rust provider.
/// `OD-RULES-014` decides this.
///
/// # Why this lives here and not in `nomos-contracts`
///
/// `OD-CONTRACTS-001` admits a type to band 0 when a peer that never compiles that crate
/// would be unable to agree with us without it. Nothing exchanges a language identifier
/// with a peer: `nomos_package::PackageManifest` carries `language_versions` but no
/// language name, so a `LanguagePackage` manifest never declares which language it is for.
/// The parties that must agree are the three language crates and the two consumers that
/// read what they recognize, which is the same constituency `SyntaxPayload` has and the
/// same reason it sits at band 23. `OD-RULES-014`'s amendment records the correction.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Language(String);

impl Language
{
    /// Wraps an authored language name.
    #[must_use]
    pub fn New(value: impl Into<String>) -> Self
    {
        return Self(value.into());
    }

    /// The language name as authored.
    #[must_use]
    pub fn As_Str(&self) -> &str
    {
        return &self.0;
    }
}

impl core::fmt::Display for Language
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(&self.0);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_New_Should_Carry_The_Name_As_Authored()
    {
        let language = Language::New("go");

        assert_eq!(language.As_Str(), "go");
        assert_eq!(language.to_string(), "go");
    }

    /// Two languages are distinguished by their authored name and nothing else, which is
    /// what lets the two Rust syntax providers declare one shared value.
    #[test]
    fn Test_Two_Languages_Should_Compare_By_Authored_Name()
    {
        assert_eq!(Language::New("rust"), Language::New("rust"));
        assert_ne!(Language::New("rust"), Language::New("go"));
    }
}
