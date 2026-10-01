//! The generic shape a package crate's provider allowlist takes.
//!
//! `OD-PACKAGE-006`'s resolution: `nomos-lang-package::KNOWN_PROVIDERS` was a two-element
//! array pulling each Rust provider's own `PROVIDER` constant by reference, checked by its
//! own `Is_Known` before a manifest's [`crate::ProviderRegistration`] was accepted. A
//! second language's package crate needs exactly the same shape -- a handful of `PROVIDER`
//! constants, pulled by reference, and one membership check -- so this crate now owns that
//! shape once, generically, rather than leaving every language-specific package crate to
//! hand-roll its own array and its own `.contains`. `nomos-lang-package` is this type's
//! first real consumer; see its own `known_providers.rs`.

/// The providers a package crate knows how to register, pulled by reference from each
/// provider crate's own `PROVIDER` constant rather than retyped here.
///
/// A thin wrapper over a static slice, not a `BTreeSet`: a package crate's own known
/// providers are unique by construction -- each entry is a distinct provider crate's own
/// constant, never copied twice into one allowlist -- so there is nothing here to
/// deduplicate, and [`Is_Known`](Self::Is_Known) is the same linear scan
/// `nomos-lang-package`'s own `Is_Known` already did before this type existed.
#[derive(Clone, Copy, Debug)]
pub struct KnownProviders(&'static [&'static str]);

impl KnownProviders
{
    /// Builds the allowlist from a package crate's own providers' `PROVIDER` constants,
    /// pulled by reference rather than retyped.
    #[must_use]
    pub const fn New(providers: &'static [&'static str]) -> Self
    {
        return Self(providers);
    }

    /// Whether `provider` is one of the constants this allowlist was built from.
    #[must_use]
    pub fn Is_Known(&self, provider: &str) -> bool
    {
        return self.0.contains(&provider);
    }

    /// The allowlist as a slice, for a caller that needs to pass it somewhere already
    /// expecting `&[&str]` -- [`crate::Parse_Manifest`]'s and [`crate::Read_Manifest`]'s
    /// own `known_providers` parameter, for instance.
    #[must_use]
    pub const fn As_Slice(&self) -> &'static [&'static str]
    {
        return self.0;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    const PROVIDERS: KnownProviders =
        KnownProviders::New(&["nomos.test.alpha", "nomos.test.beta"]);

    #[test]
    fn Test_A_Constant_The_Allowlist_Was_Built_From_Is_Known()
    {
        assert!(PROVIDERS.Is_Known("nomos.test.alpha"));
        assert!(PROVIDERS.Is_Known("nomos.test.beta"));
    }

    #[test]
    fn Test_An_Unlisted_Provider_Is_Not_Known()
    {
        assert!(!PROVIDERS.Is_Known("nomos.test.gamma"));
    }

    #[test]
    fn Test_As_Slice_Round_Trips_The_Constructor_Argument()
    {
        assert_eq!(PROVIDERS.As_Slice(), &["nomos.test.alpha", "nomos.test.beta"]);
    }

    /// An empty allowlist is a real value, not a construction error -- a package crate
    /// with no providers yet is still a `KnownProviders`, it merely knows nothing.
    #[test]
    fn Test_An_Empty_Allowlist_Knows_Nothing()
    {
        let empty = KnownProviders::New(&[]);

        assert!(!empty.Is_Known("nomos.test.alpha"));
    }
}
