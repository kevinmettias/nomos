//! The providers this workspace's C# `LanguagePackage` may register.
//!
//! The identical shape `nomos-lang-package::known_providers` and
//! `nomos-lang-go-package::known_providers` state for their own languages: a short, closed list
//! built through `nomos_package::KnownProviders`, not a bespoke array or an open string. A
//! manifest naming a provider this package does not carry would be a registration nothing
//! resolves.

use nomos_package::KnownProviders;

/// Every `ProviderId` a manifest read by this crate may register.
///
/// One provider today: `nomos-lang-csharp`'s own syntax provider. Pulled from its own `PROVIDER`
/// constant rather than retyped as a literal here, the identical reason
/// `nomos-lang-package::KNOWN_PROVIDERS` gives for its own two.
///
/// A `LanguagePackage` declares the providers that recognize its language -- those offering
/// `nomos.cap.syntax.items` for it -- which is `OD-PACKAGE-018`'s rule, and for C# that is one
/// entry. `nomos-lang-csharp-compiler` (`nomos.cap.csharp.conditional_compilation`) is
/// deliberately not a second: it answers which branches a build compiles, asking `MSBuild`, and
/// recognizes nothing. `packages/nomos.tool.csharp.json` declares it, under `SEMANTIC_MODEL`, and
/// `nomos-tool-package` admits it.
pub const KNOWN_PROVIDERS: &[&str] = KnownProviders::New(&[nomos_lang_csharp::PROVIDER]).As_Slice();

/// Whether a provider identifier is one this package may register.
#[must_use]
pub fn Is_Known(provider: &str) -> bool
{
    return KNOWN_PROVIDERS.contains(&provider);
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_The_Csharp_Provider_Is_Known()
    {
        assert!(Is_Known(nomos_lang_csharp::PROVIDER));
    }

    #[test]
    fn Test_An_Unregistered_Provider_Is_Not_Known()
    {
        assert!(!Is_Known("nomos.lang.python.ast"));
    }

    /// A sibling language's provider is not this package's to register, and the negative control
    /// that keeps the assertion above from passing over an allowlist that admits everything.
    #[test]
    fn Test_A_Sibling_Languages_Provider_Is_Not_Known_Here()
    {
        assert!(!Is_Known("nomos.lang.go.tree-sitter"));
    }
}
