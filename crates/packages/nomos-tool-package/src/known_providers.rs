//! The providers this workspace's `ToolProvider` manifest may register.
//!
//! A short, closed list rather than an open string, the same reason
//! `nomos-lang-rust-package`'s own `known_providers.rs` gives: a manifest naming a provider
//! this crate does not actually carry would be a registration nothing resolves.
//!
//! Built through [`nomos_package::KnownProviders`], the same generic provider-registration
//! base type `nomos-lang-rust-package` is the first real consumer of, rather than a bespoke
//! array this crate assembled and checked on its own.

use nomos_package::KnownProviders;

/// Every `ProviderId` a manifest read by this crate may register: this workspace's two
/// real `ToolProvider`s.
///
/// Pulled from each provider crate's own `PROVIDER` constant rather than retyped as a
/// literal here, for the reason `nomos-lang-rust-package`'s own `known_providers.rs` gives:
/// two crates that each retype one string are two places that string can drift apart,
/// unnoticed until something compares them.
pub const KNOWN_PROVIDERS: &[&str] =
    KnownProviders::New(&[nomos_lang_rust_clippy::PROVIDER, nomos_lang_rust_deny::PROVIDER]).As_Slice();

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
    fn Test_Both_Real_ToolProviders_Are_Known()
    {
        assert!(Is_Known(nomos_lang_rust_clippy::PROVIDER));
        assert!(Is_Known(nomos_lang_rust_deny::PROVIDER));
    }

    #[test]
    fn Test_An_Unregistered_Provider_Is_Not_Known()
    {
        assert!(!Is_Known("nomos.lang.python.ast"));
    }
}
