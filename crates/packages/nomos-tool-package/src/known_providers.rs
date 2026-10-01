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

/// Every `ProviderId` a manifest read by this crate may register: every language provider in
/// this workspace that recognizes no language.
///
/// `OD-PACKAGE-018` is the rule this list is checked against. A language package declares
/// the providers that offer `nomos.cap.syntax.items` for its language, and a `ToolProvider`
/// package declares every other `nomos.lang.*` provider, under its `OD-CAPABILITY-013` family and
/// whatever its delivery. So this list holds cargo, clippy, deny, the compiler and complexity for
/// Rust, go-modules, `go vet` and the `go/types` helper for Go, and `MSBuild` for C#. `packages/nomos.tool.rust.json`, `.go.json` and
/// `.csharp.json` declare them.
///
/// Pulled from each provider crate's own `PROVIDER` constant rather than retyped as a
/// literal here, for the reason `nomos-lang-rust-package`'s own `known_providers.rs` gives:
/// two crates that each retype one string are two places that string can drift apart,
/// unnoticed until something compares them.
pub const KNOWN_PROVIDERS: &[&str] = KnownProviders::New(&[
    nomos_lang_rust_cargo::PROVIDER,
    nomos_lang_rust_clippy::PROVIDER,
    nomos_lang_rust_deny::PROVIDER,
    nomos_lang_rust_compiler::PROVIDER,
    nomos_lang_rust_complexity::PROVIDER,
    nomos_lang_go_modules::PROVIDER,
    nomos_lang_go_lint::PROVIDER,
    nomos_lang_go_types::PROVIDER,
    nomos_lang_csharp_compiler::PROVIDER,
])
.As_Slice();

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
    fn Test_Every_Non_Recognizing_Language_Provider_Is_Known()
    {
        for provider in [
            nomos_lang_rust_cargo::PROVIDER,
            nomos_lang_rust_clippy::PROVIDER,
            nomos_lang_rust_deny::PROVIDER,
            nomos_lang_rust_compiler::PROVIDER,
            nomos_lang_rust_complexity::PROVIDER,
            nomos_lang_go_modules::PROVIDER,
            nomos_lang_go_lint::PROVIDER,
            nomos_lang_go_types::PROVIDER,
            nomos_lang_csharp_compiler::PROVIDER,
        ]
        {
            assert!(Is_Known(provider), "{provider}");
        }
    }

    /// A provider that recognizes a language belongs to that language's package, never to this
    /// one -- `OD-PACKAGE-018`'s other half, and the control that keeps the test above from passing
    /// over an allowlist that admits everything.
    #[test]
    fn Test_A_Recognizing_Provider_Is_Not_Known_Here()
    {
        assert!(!Is_Known("nomos.lang.rust.syn"));
        assert!(!Is_Known("nomos.lang.go.tree-sitter"));
        assert!(!Is_Known("nomos.lang.csharp.tree-sitter"));
    }

    #[test]
    fn Test_An_Unregistered_Provider_Is_Not_Known()
    {
        assert!(!Is_Known("nomos.lang.python.ast"));
    }
}
