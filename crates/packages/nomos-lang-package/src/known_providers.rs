//! The providers this workspace's Rust `LanguagePackage` may register.
//!
//! A short, closed list rather than an open string. A manifest naming a provider this
//! package does not actually carry would be exactly the "manifest with no reader"
//! `OD-PACKAGE-001` warns against, one level down: a registration nothing resolves.

/// Every `ProviderId` a manifest read by this crate may register.
///
/// Pulled from each provider crate's own `PROVIDER` constant rather than retyped as a
/// literal here, for the reason `nomos-lang-rust-scan`'s own `guarantee.rs` gives about
/// its capability constant: two crates that each retype one string are two places that
/// string can drift apart, unnoticed until something compares them. Depending on both
/// provider crates directly is what a manifest crate above them in the band order is
/// for.
pub const KNOWN_PROVIDERS: [&str; 2] = [nomos_lang_rust::PROVIDER, nomos_lang_rust_scan::PROVIDER];

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
    fn Test_Both_Rust_Providers_Are_Known()
    {
        assert!(Is_Known(nomos_lang_rust::PROVIDER));
        assert!(Is_Known(nomos_lang_rust_scan::PROVIDER));
    }

    #[test]
    fn Test_An_Unregistered_Provider_Is_Not_Known()
    {
        assert!(!Is_Known("nomos.lang.python.ast"));
    }
}
