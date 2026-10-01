//! What this binary is actually being built as, passed through to the crate.
//!
//! A deliberate twin of `nomos-cli`'s own `build.rs` and `tests/integration`'s own —
//! captured identically here for the identical reason: a build script may not depend on a
//! crate, so a `BuildVariant` computed by one cannot be shared with another. This crate
//! needs its own copy because it calls `nomos_check_orchestration::Run` directly, the same
//! caller-supplies-the-variant contract `nomos-cli::check` and `nomos-correction-
//! orchestration`'s own tests already carry.

fn main() -> Result<(), String>
{
    // Cargo sets both for every build script. A missing one means the contract with cargo
    // has changed, and guessing would produce a variant identity describing a build that
    // never happened.
    let target = Cargo_Variable("TARGET")?;
    let profile = Cargo_Variable("PROFILE")?;
    let toolchain = Toolchain();
    let features = Enabled_Features();

    println!("cargo::rustc-env=NOMOS_TARGET={target}");
    println!("cargo::rustc-env=NOMOS_PROFILE={profile}");
    println!("cargo::rustc-env=NOMOS_TOOLCHAIN={toolchain}");
    println!("cargo::rustc-env=NOMOS_FEATURES={features}");

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=RUSTUP_TOOLCHAIN");

    return Ok(());
}

/// One variable cargo sets for every build script.
///
/// Returned rather than unwound. A build script's caller is cargo, which prints the `Err`
/// and fails the build — the same stop, with the same sentence, reached by a path the
/// caller acknowledged.
fn Cargo_Variable(name: &str) -> Result<String, String>
{
    return std::env::var(name).map_err(|cause| {
        return format!("cargo sets {name} for every build script, and did not: {cause}");
    });
}

/// Absent outside rustup — a direct rustc invocation, a distribution toolchain, a vendored
/// compiler. Named as unstated rather than defaulted to a version, because a variant claiming
/// `1.85` on a toolchain nobody identified is a false statement about which compiler produced
/// the facts.
fn Toolchain() -> String
{
    return std::env::var("RUSTUP_TOOLCHAIN").unwrap_or_else(|_| return "unstated".to_owned());
}

/// The enabled features as one comma-separated field.
///
/// Cargo exports one variable per enabled feature and no list. Collected into a set, so the
/// order they happen to be read in cannot reach the variant's identity.
fn Enabled_Features() -> String
{
    use std::collections::BTreeSet;

    let features: BTreeSet<String> = std::env::vars()
        .filter_map(|(name, _)| {
            return name
                .strip_prefix("CARGO_FEATURE_")
                .map(|feature| return feature.to_lowercase().replace('_', "-"));
        })
        .collect();

    return features.into_iter().collect::<Vec<String>>().join(",");
}
