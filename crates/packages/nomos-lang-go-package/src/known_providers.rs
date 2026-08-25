//! The providers this workspace's Go `LanguagePackage` may register.
//!
//! The identical shape `nomos-lang-package::known_providers` states for Rust: a short,
//! closed list built through `nomos_package::KnownProviders`, not a bespoke array or an
//! open string. A manifest naming a provider this package does not carry would be a
//! registration nothing resolves.

use nomos_package::KnownProviders;

/// Every `ProviderId` a manifest read by this crate may register.
///
/// One provider today: `nomos-lang-go`'s own syntax provider. Pulled from its own
/// `PROVIDER` constant rather than retyped as a literal here, the identical reason
/// `nomos-lang-package::KNOWN_PROVIDERS` gives for its own two.
///
/// `nomos-lang-go-modules` (`nomos.cap.dependency.edges`) is deliberately not a second
/// entry here: `nomos-lang-package`'s own allowlist carries only `nomos-lang-rust` and
/// `nomos-lang-rust-scan` — the syntax-capability providers a `LanguagePackage` registers
/// — and not `nomos-lang-rust-cargo`, its own dependency-edges provider. This crate keeps
/// the identical scope for the identical reason: a `LanguagePackage` registers what
/// recognizes the language, not every capability provider that happens to read its
/// ecosystem's files.
pub const KNOWN_PROVIDERS: &[&str] = KnownProviders::New(&[nomos_lang_go::PROVIDER]).As_Slice();

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
    fn Test_The_Go_Provider_Is_Known()
    {
        assert!(Is_Known(nomos_lang_go::PROVIDER));
    }

    #[test]
    fn Test_An_Unregistered_Provider_Is_Not_Known()
    {
        assert!(!Is_Known("nomos.lang.python.ast"));
    }

    #[test]
    fn Test_The_Sibling_Dependency_Edges_Provider_Is_Not_Known_Here()
    {
        assert!(!Is_Known("nomos.lang.go.modules"));
    }
}
